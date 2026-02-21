//! v-kernel — Jupyter kernel for the V programming language
//!
//! Implements the Jupyter messaging protocol (v5.3) over ZeroMQ.
//! Zed's REPL uses this kernel when you press Ctrl+Shift+Enter on a .v file.
//!
//! Architecture:
//!   - Shell socket:   receives execute_request, kernel_info_request, etc.
//!   - IOPub socket:   broadcasts status, stream output, errors to all clients
//!   - Stdin socket:   (input_request — not used by V, kept for protocol compliance)
//!   - Control socket: handles shutdown_request / interrupt_request
//!   - Heartbeat:      echoes back raw bytes to signal liveness
//!
//! Stateful execution:
//!   Each session accumulates top-level declarations (fn, struct, enum, const,
//!   import, type, interface) across cells. Bare statements and expressions are
//!   wrapped in fn main(). On each execute_request the kernel synthesises a
//!   complete .v source file and runs it via `v run <tmpfile>`.

use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::Sha256;
use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};
use uuid::Uuid;
use zmq::{Context, Socket, SocketType};

// ── Jupyter wire-protocol types ──────────────────────────────────────────────

/// A Jupyter message as decoded from the wire.
#[derive(Debug, Clone)]
struct JupyterMessage {
    identities: Vec<Vec<u8>>,
    header: Value,
    parent_header: Value,
    metadata: Value,
    content: Value,
    buffers: Vec<Vec<u8>>,
}

impl JupyterMessage {
    /// Decode a multipart ZMQ message into a JupyterMessage.
    fn from_frames(frames: Vec<Vec<u8>>, key: &[u8]) -> Option<Self> {
        // Find the delimiter frame "<IDS|MSG>"
        let delim = b"<IDS|MSG>";
        let delim_pos = frames.iter().position(|f| f == delim)?;

        let identities = frames[..delim_pos].to_vec();
        // frames after delimiter: hmac, header, parent_header, metadata, content, [buffers…]
        let rest = &frames[delim_pos + 1..];
        if rest.len() < 5 {
            return None;
        }

        let hmac_sig = std::str::from_utf8(&rest[0]).ok()?;
        let header_raw = &rest[1];
        let parent_raw = &rest[2];
        let metadata_raw = &rest[3];
        let content_raw = &rest[4];

        // Verify HMAC-SHA256 signature
        if !key.is_empty() {
            let expected = compute_hmac(key, &[header_raw, parent_raw, metadata_raw, content_raw]);
            if expected != hmac_sig {
                eprintln!("[v-kernel] HMAC mismatch — dropping message");
                return None;
            }
        }

        let buffers = rest[5..].to_vec();

        Some(JupyterMessage {
            identities,
            header: serde_json::from_slice(header_raw).unwrap_or(json!({})),
            parent_header: serde_json::from_slice(parent_raw).unwrap_or(json!({})),
            metadata: serde_json::from_slice(metadata_raw).unwrap_or(json!({})),
            content: serde_json::from_slice(content_raw).unwrap_or(json!({})),
            buffers,
        })
    }

    /// Encode a reply message to multipart ZMQ frames.
    fn to_frames(&self, key: &[u8]) -> Vec<Vec<u8>> {
        let header_raw = serde_json::to_vec(&self.header).unwrap();
        let parent_raw = serde_json::to_vec(&self.parent_header).unwrap();
        let metadata_raw = serde_json::to_vec(&self.metadata).unwrap();
        let content_raw = serde_json::to_vec(&self.content).unwrap();

        let sig = compute_hmac(key, &[&header_raw, &parent_raw, &metadata_raw, &content_raw]);

        let mut frames: Vec<Vec<u8>> = self.identities.clone();
        frames.push(b"<IDS|MSG>".to_vec());
        frames.push(sig.into_bytes());
        frames.push(header_raw);
        frames.push(parent_raw);
        frames.push(metadata_raw);
        frames.push(content_raw);
        for buf in &self.buffers {
            frames.push(buf.clone());
        }
        frames
    }
}

fn compute_hmac(key: &[u8], parts: &[&[u8]]) -> String {
    if key.is_empty() {
        return String::new();
    }
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts any key size");
    for part in parts {
        mac.update(part);
    }
    hex::encode(mac.finalize().into_bytes())
}

/// Build a reply header for a given message type.
fn make_header(msg_type: &str, session: &str) -> Value {
    json!({
        "msg_id": Uuid::new_v4().to_string(),
        "session": session,
        "username": "v-kernel",
        "date": Utc::now().to_rfc3339(),
        "msg_type": msg_type,
        "version": "5.3"
    })
}

/// Send a message on a socket.
fn send_message(socket: &Socket, msg: &JupyterMessage, key: &[u8]) {
    let frames = msg.to_frames(key);
    for (i, frame) in frames.iter().enumerate() {
        let is_last = i == frames.len() - 1;
        if is_last {
            socket.send(frame, 0).ok();
        } else {
            socket.send(frame, zmq::SNDMORE).ok();
        }
    }
}

/// Receive a multipart message from a socket.
fn recv_message(socket: &Socket, key: &[u8]) -> Option<JupyterMessage> {
    let mut frames = Vec::new();
    loop {
        let frame = socket.recv_bytes(0).ok()?;
        frames.push(frame);
        if !socket.get_rcvmore().unwrap_or(false) {
            break;
        }
    }
    JupyterMessage::from_frames(frames, key)
}

// ── Connection file ───────────────────────────────────────────────────────────

/// The JSON connection file Jupyter writes and passes to us via argv.
#[derive(Debug, Deserialize)]
struct ConnectionInfo {
    ip: String,
    transport: String,
    shell_port: u16,
    iopub_port: u16,
    stdin_port: u16,
    control_port: u16,
    hb_port: u16,
    key: String,
    #[allow(dead_code)]
    signature_scheme: String,
    #[allow(dead_code)]
    kernel_name: Option<String>,
}

impl ConnectionInfo {
    fn endpoint(&self, port: u16) -> String {
        format!("{}://{}:{}", self.transport, self.ip, port)
    }
}

// ── Session state ─────────────────────────────────────────────────────────────

/// Accumulated kernel state across cells.
#[derive(Debug, Default)]
struct KernelState {
    /// Top-level declarations seen so far (fn, struct, enum, …)
    declarations: Vec<String>,
    /// Statements seen so far (executed inside fn main each time)
    statements: Vec<String>,
    /// Execution counter (shown in Zed as [1], [2], …)
    execution_count: u32,
    /// Temporary directory for compiled artefacts
    tmp_dir: PathBuf,
}

impl KernelState {
    fn new() -> Self {
        let tmp_dir = env::temp_dir().join(format!("v-kernel-{}", Uuid::new_v4()));
        fs::create_dir_all(&tmp_dir).ok();
        KernelState {
            declarations: Vec::new(),
            statements: Vec::new(),
            execution_count: 0,
            tmp_dir,
        }
    }

    /// Classify and accumulate a cell, then run it.
    /// Returns (stdout, stderr, is_error).
    fn execute(&mut self, code: &str) -> (String, String, bool) {
        self.execution_count += 1;

        let (new_decls, new_stmts) = classify(code);

        // Add new declarations and statements to the accumulator
        self.declarations.extend(new_decls);
        self.statements.extend(new_stmts);

        // Build the full source file
        let source = self.build_source();

        // Write to a temp file
        let src_path = self.tmp_dir.join(format!("cell_{}.v", self.execution_count));
        if let Err(e) = fs::write(&src_path, &source) {
            return (String::new(), format!("Failed to write source: {e}"), true);
        }

        // Run with `v run <file>`
        run_v(&src_path)
    }

    /// Synthesise a complete runnable V source from accumulated state.
    fn build_source(&self) -> String {
        let mut out = String::new();

        // Collect imports from declarations so they appear at the top
        let imports: Vec<&str> = self
            .declarations
            .iter()
            .filter(|d| d.trim_start().starts_with("import "))
            .map(|s| s.as_str())
            .collect();

        let non_imports: Vec<&str> = self
            .declarations
            .iter()
            .filter(|d| !d.trim_start().starts_with("import "))
            .map(|s| s.as_str())
            .collect();

        // module main is implicit in v run, but being explicit avoids warnings
        out.push_str("module main\n\n");

        for imp in &imports {
            out.push_str(imp);
            out.push('\n');
        }
        if !imports.is_empty() {
            out.push('\n');
        }

        for decl in &non_imports {
            out.push_str(decl);
            out.push_str("\n\n");
        }

        // Wrap statements in fn main()
        if !self.statements.is_empty() {
            out.push_str("fn main() {\n");
            for stmt in &self.statements {
                // Indent each line
                for line in stmt.lines() {
                    out.push('\t');
                    out.push_str(line);
                    out.push('\n');
                }
            }
            out.push_str("}\n");
        }

        out
    }
}

impl Drop for KernelState {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.tmp_dir).ok();
    }
}

// ── V code classifier ─────────────────────────────────────────────────────────

/// Determines which lines are top-level declarations vs. statements.
/// Returns (declarations, statements).
///
/// Top-level in V: fn, struct, interface, enum, type, const, import, __global
/// Everything else is a statement to be wrapped in fn main().
///
/// The classifier is line/block-based rather than a full parser; it works
/// well for REPL-style usage where cells tend to contain one conceptual unit.
fn classify(code: &str) -> (Vec<String>, Vec<String>) {
    let mut decls = Vec::new();
    let mut stmts = Vec::new();

    let lines: Vec<&str> = code.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        // Skip blank lines and comments at the top level
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") {
            i += 1;
            continue;
        }

        // Shebang
        if trimmed.starts_with("#!") {
            i += 1;
            continue;
        }

        // module clause — skip (we add it ourselves)
        if trimmed.starts_with("module ") {
            i += 1;
            continue;
        }

        // Detect top-level declaration starters
        let is_decl = is_top_level_decl(trimmed);

        if is_decl {
            // Collect the whole block (up to the matching closing brace, or
            // a single line for import/type statements)
            let (block, consumed) = collect_block(&lines, i);
            decls.push(block);
            i += consumed;
        } else {
            // It's a statement / expression
            let (block, consumed) = collect_statement(&lines, i);
            stmts.push(block);
            i += consumed;
        }
    }

    (decls, stmts)
}

/// Returns true if a line looks like the start of a top-level declaration.
fn is_top_level_decl(line: &str) -> bool {
    // Strip pub/mut/static modifiers before checking
    let stripped = line
        .trim_start_matches("pub ")
        .trim_start_matches("mut ")
        .trim_start_matches("static ");

    // Strip attribute lines — they belong to the declaration that follows,
    // so we treat the attribute itself as "part of a declaration start"
    if stripped.starts_with('[') || stripped.starts_with("@[") {
        return true;
    }

    let keywords = [
        "fn ",
        "struct ",
        "interface ",
        "enum ",
        "type ",
        "const ",
        "const(",
        "import ",
        "__global",
    ];
    keywords.iter().any(|kw| stripped.starts_with(kw))
}

/// Collect a brace-delimited block (or a single-line statement) starting at
/// line index `start`. Returns (source, lines_consumed).
fn collect_block(lines: &[&str], start: usize) -> (String, usize) {
    let first = lines[start];

    // Single-line declarations (import, single-line type alias)
    if !first.contains('{') {
        return (first.to_string(), 1);
    }

    // Multi-line: count braces
    let mut depth = 0i32;
    let mut collected = Vec::new();
    let mut i = start;

    while i < lines.len() {
        let line = lines[i];
        for ch in line.chars() {
            match ch {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }
        collected.push(line);
        i += 1;
        if depth <= 0 {
            break;
        }
    }

    (collected.join("\n"), i - start)
}

/// Collect a statement starting at `start`. For simple one-liners this is
/// just that line. For multi-line expressions (e.g. `if … { … }`) we collect
/// the full block.
fn collect_statement(lines: &[&str], start: usize) -> (String, usize) {
    let first = lines[start];

    if !first.contains('{') {
        return (first.to_string(), 1);
    }

    // Multi-line statement — count braces
    let mut depth = 0i32;
    let mut collected = Vec::new();
    let mut i = start;

    while i < lines.len() {
        let line = lines[i];
        for ch in line.chars() {
            match ch {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }
        collected.push(line);
        i += 1;
        if depth <= 0 {
            break;
        }
    }

    (collected.join("\n"), i - start)
}

// ── V runner ─────────────────────────────────────────────────────────────────

/// Execute a V source file and return (stdout, stderr, is_error).
fn run_v(src: &PathBuf) -> (String, String, bool) {
    let mut cmd = Command::new("v");
    cmd.arg("run")
        .arg(src)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return (
                String::new(),
                format!(
                    "Could not start `v`. Is V installed and in PATH?\nError: {e}"
                ),
                true,
            );
        }
    };

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => return (String::new(), format!("Failed to wait on `v run`: {e}"), true),
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let is_error = !output.status.success();

    (stdout, stderr, is_error)
}

// ── Kernel info ───────────────────────────────────────────────────────────────

fn kernel_info_content() -> Value {
    json!({
        "status": "ok",
        "protocol_version": "5.3",
        "implementation": "v-kernel",
        "implementation_version": "0.1.0",
        "language_info": {
            "name": "v",
            "version": "0.4",
            "mimetype": "text/x-vlang",
            "file_extension": ".v",
            "pygments_lexer": "v",
            "codemirror_mode": "clike"
        },
        "banner": "V kernel for Zed — stateful REPL powered by v-kernel",
        "help_links": [
            {
                "text": "V Documentation",
                "url": "https://docs.vlang.io/"
            }
        ]
    })
}

// ── Main kernel loop ──────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: v-kernel <connection-file>");
        std::process::exit(1);
    }

    let conn_json = fs::read_to_string(&args[1]).expect("Could not read connection file");
    let conn: ConnectionInfo =
        serde_json::from_str(&conn_json).expect("Invalid connection file JSON");

    let key = conn.key.as_bytes().to_vec();
    let session_id = Uuid::new_v4().to_string();

    let ctx = Context::new();

    // ── Bind sockets ─────────────────────────────────────────────────────────

    let shell = ctx.socket(SocketType::ROUTER).unwrap();
    shell.bind(&conn.endpoint(conn.shell_port)).unwrap();

    let iopub = ctx.socket(SocketType::PUB).unwrap();
    iopub.bind(&conn.endpoint(conn.iopub_port)).unwrap();

    let stdin = ctx.socket(SocketType::ROUTER).unwrap();
    stdin.bind(&conn.endpoint(conn.stdin_port)).unwrap();

    let control = ctx.socket(SocketType::ROUTER).unwrap();
    control.bind(&conn.endpoint(conn.control_port)).unwrap();

    let heartbeat = ctx.socket(SocketType::REP).unwrap();
    heartbeat.bind(&conn.endpoint(conn.hb_port)).unwrap();

    eprintln!("[v-kernel] Listening on all sockets. Session: {session_id}");

    // ── Heartbeat thread ──────────────────────────────────────────────────────
    {
        thread::spawn(move || loop {
            if let Ok(msg) = heartbeat.recv_bytes(0) {
                heartbeat.send(&msg, 0).ok();
            }
        });
    }

    // ── Shared state ──────────────────────────────────────────────────────────
    let state = Arc::new(Mutex::new(KernelState::new()));

    // ── IOPub helper (clone Arc for use in closures) ──────────────────────────
    // We share iopub via Arc<Mutex<Socket>> so shell handler can send on it.
    let iopub = Arc::new(Mutex::new(iopub));

    // ── Control thread ────────────────────────────────────────────────────────
    {
        let key = key.clone();
        let session_id = session_id.clone();
        thread::spawn(move || loop {
            if let Some(msg) = recv_message(&control, &key) {
                let msg_type = msg.header["msg_type"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();

                match msg_type.as_str() {
                    "shutdown_request" => {
                        let restart = msg.content["restart"].as_bool().unwrap_or(false);
                        let reply = JupyterMessage {
                            identities: msg.identities.clone(),
                            header: make_header("shutdown_reply", &session_id),
                            parent_header: msg.header.clone(),
                            metadata: json!({}),
                            content: json!({ "status": "ok", "restart": restart }),
                            buffers: vec![],
                        };
                        send_message(&control, &reply, &key);
                        eprintln!("[v-kernel] Shutdown requested. restart={restart}");
                        if !restart {
                            std::process::exit(0);
                        }
                    }
                    _ => {
                        eprintln!("[v-kernel] Unhandled control msg: {msg_type}");
                    }
                }
            }
        });
    }

    // ── Shell loop ────────────────────────────────────────────────────────────
    loop {
        let msg = match recv_message(&shell, &key) {
            Some(m) => m,
            None => continue,
        };

        let msg_type = msg.header["msg_type"]
            .as_str()
            .unwrap_or("")
            .to_string();

        eprintln!("[v-kernel] shell <- {msg_type}");

        match msg_type.as_str() {
            // ── kernel_info_request ──────────────────────────────────────────
            "kernel_info_request" => {
                let reply = JupyterMessage {
                    identities: msg.identities.clone(),
                    header: make_header("kernel_info_reply", &session_id),
                    parent_header: msg.header.clone(),
                    metadata: json!({}),
                    content: kernel_info_content(),
                    buffers: vec![],
                };
                send_message(&shell, &reply, &key);
            }

            // ── execute_request ──────────────────────────────────────────────
            "execute_request" => {
                let code = msg.content["code"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                let silent = msg.content["silent"].as_bool().unwrap_or(false);

                let exec_count = {
                    let s = state.lock().unwrap();
                    s.execution_count + 1
                };

                // Publish "busy" status
                if !silent {
                    publish_status(&iopub, &key, &session_id, &msg, "busy");
                }

                // Publish execute_input so Zed shows [N] next to the cell
                if !silent {
                    let input_msg = JupyterMessage {
                        identities: vec![],
                        header: make_header("execute_input", &session_id),
                        parent_header: msg.header.clone(),
                        metadata: json!({}),
                        content: json!({
                            "code": code,
                            "execution_count": exec_count
                        }),
                        buffers: vec![],
                    };
                    let iopub = iopub.lock().unwrap();
                    send_message(&iopub, &input_msg, &key);
                }

                // Run the code
                let (stdout, stderr, is_error) = {
                    let mut s = state.lock().unwrap();
                    s.execute(&code)
                };

                let final_exec_count = {
                    let s = state.lock().unwrap();
                    s.execution_count
                };

                // Publish stdout stream
                if !stdout.is_empty() && !silent {
                    let stream_msg = JupyterMessage {
                        identities: vec![],
                        header: make_header("stream", &session_id),
                        parent_header: msg.header.clone(),
                        metadata: json!({}),
                        content: json!({
                            "name": "stdout",
                            "text": stdout
                        }),
                        buffers: vec![],
                    };
                    let iopub = iopub.lock().unwrap();
                    send_message(&iopub, &stream_msg, &key);
                }

                // Publish stderr / error
                if is_error && !silent {
                    // Publish to stderr stream
                    let stream_msg = JupyterMessage {
                        identities: vec![],
                        header: make_header("stream", &session_id),
                        parent_header: msg.header.clone(),
                        metadata: json!({}),
                        content: json!({
                            "name": "stderr",
                            "text": stderr
                        }),
                        buffers: vec![],
                    };
                    let iopub_lock = iopub.lock().unwrap();
                    send_message(&iopub_lock, &stream_msg, &key);
                    drop(iopub_lock);

                    // Also publish error message so Zed can show it inline
                    let error_msg = JupyterMessage {
                        identities: vec![],
                        header: make_header("error", &session_id),
                        parent_header: msg.header.clone(),
                        metadata: json!({}),
                        content: json!({
                            "ename": "CompileError",
                            "evalue": "V compilation or runtime error",
                            "traceback": stderr.lines().collect::<Vec<_>>()
                        }),
                        buffers: vec![],
                    };
                    let iopub_lock = iopub.lock().unwrap();
                    send_message(&iopub_lock, &error_msg, &key);
                } else if !stderr.is_empty() && !silent {
                    let stream_msg = JupyterMessage {
                        identities: vec![],
                        header: make_header("stream", &session_id),
                        parent_header: msg.header.clone(),
                        metadata: json!({}),
                        content: json!({
                            "name": "stderr",
                            "text": stderr
                        }),
                        buffers: vec![],
                    };
                    let iopub = iopub.lock().unwrap();
                    send_message(&iopub, &stream_msg, &key);
                }

                // Send execute_reply
                let _status = if is_error { "error" } else { "ok" };
                let reply_content = if is_error {
                    json!({
                        "status": "error",
                        "execution_count": final_exec_count,
                        "ename": "CompileError",
                        "evalue": "V compilation or runtime error",
                        "traceback": stderr.lines().collect::<Vec<_>>()
                    })
                } else {
                    json!({
                        "status": "ok",
                        "execution_count": final_exec_count,
                        "payload": [],
                        "user_expressions": {}
                    })
                };

                let reply = JupyterMessage {
                    identities: msg.identities.clone(),
                    header: make_header("execute_reply", &session_id),
                    parent_header: msg.header.clone(),
                    metadata: json!({}),
                    content: reply_content,
                    buffers: vec![],
                };
                send_message(&shell, &reply, &key);

                // Publish "idle" status
                if !silent {
                    publish_status(&iopub, &key, &session_id, &msg, "idle");
                }
            }

            // ── is_complete_request ──────────────────────────────────────────
            // Zed may send this to check if a cell is syntactically complete.
            "is_complete_request" => {
                let reply = JupyterMessage {
                    identities: msg.identities.clone(),
                    header: make_header("is_complete_reply", &session_id),
                    parent_header: msg.header.clone(),
                    metadata: json!({}),
                    content: json!({ "status": "complete" }),
                    buffers: vec![],
                };
                send_message(&shell, &reply, &key);
            }

            // ── comm_info_request (notebooks use this for widgets) ───────────
            "comm_info_request" => {
                let reply = JupyterMessage {
                    identities: msg.identities.clone(),
                    header: make_header("comm_info_reply", &session_id),
                    parent_header: msg.header.clone(),
                    metadata: json!({}),
                    content: json!({ "status": "ok", "comms": {} }),
                    buffers: vec![],
                };
                send_message(&shell, &reply, &key);
            }

            // ── history_request ──────────────────────────────────────────────
            "history_request" => {
                let reply = JupyterMessage {
                    identities: msg.identities.clone(),
                    header: make_header("history_reply", &session_id),
                    parent_header: msg.header.clone(),
                    metadata: json!({}),
                    content: json!({ "status": "ok", "history": [] }),
                    buffers: vec![],
                };
                send_message(&shell, &reply, &key);
            }

            other => {
                eprintln!("[v-kernel] Unhandled shell msg type: {other}");
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn publish_status(
    iopub: &Arc<Mutex<Socket>>,
    key: &[u8],
    session_id: &str,
    parent: &JupyterMessage,
    execution_state: &str,
) {
    let status_msg = JupyterMessage {
        identities: vec![],
        header: make_header("status", session_id),
        parent_header: parent.header.clone(),
        metadata: json!({}),
        content: json!({ "execution_state": execution_state }),
        buffers: vec![],
    };
    let iopub = iopub.lock().unwrap();
    send_message(&iopub, &status_msg, key);
}

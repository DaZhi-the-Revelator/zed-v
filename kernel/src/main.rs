//! v-kernel — Jupyter kernel for the V programming language
//!
//! Implements the Jupyter messaging protocol (v5.3) over ZeroMQ.
//! Zed's REPL uses this kernel when you press Ctrl+Shift+Enter on a .v file.
//!
//! Architecture:
//!   - Shell socket:   receives execute_request, kernel_info_request, etc.
//!   - IOPub socket:   broadcasts status, stream output, errors to all clients
//!   - Stdin socket:   (input_request — not used by V, kept for protocol compliance)
//!   - Control socket: handles shutdown_request, interrupt_request
//!   - Heartbeat:      echoes back raw bytes to signal liveness
//!
//! Stateful execution:
//!   Top-level declarations (fn, struct, enum, const, import, type, interface)
//!   accumulate across cells — later cells can reference structs and functions
//!   defined in earlier cells.
//!
//!   Bare statements and expressions are wrapped in fn main() for the current
//!   cell only and are NOT accumulated.  This means re-running or editing a
//!   cell never causes "already defined" / redeclaration errors from stale
//!   earlier runs.  On each execute_request the kernel synthesises a complete
//!   .v source file and runs it via `v run <tmpfile>`.
//!
//! Rich dump() output:
//!   The kernel post-processes stdout to detect V's dump() output format:
//!     [module.v:line] name = type(value)
//!   Matching lines are collected and emitted as a styled HTML table via the
//!   Jupyter display_data message (MIME type text/html). Non-matching lines
//!   are forwarded as plain stream output as before.

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
    /// Top-level declarations seen so far (fn, struct, enum, …).
    /// These accumulate across cells — later cells can use earlier structs/fns.
    declarations: Vec<String>,
    /// Execution counter (shown in Zed as [1], [2], …)
    execution_count: u32,
    /// Temporary directory for compiled artefacts
    tmp_dir: PathBuf,
    /// PID of the currently running `v run` child process, if any.
    running_pid: Option<u32>,
}

impl KernelState {
    fn new() -> Self {
        let tmp_dir = env::temp_dir().join(format!("v-kernel-{}", Uuid::new_v4()));
        fs::create_dir_all(&tmp_dir).ok();
        KernelState {
            declarations: Vec::new(),
            execution_count: 0,
            tmp_dir,
            running_pid: None,
        }
    }

    /// Classify and accumulate a cell, then run it.
    ///
    /// Magic commands:
    ///   %reset  — clear all accumulated declarations and reset the execution
    ///             counter to 0. Returns a confirmation message and does NOT
    ///             invoke the V compiler.
    ///   %show   — print the complete synthesised V source file that would be
    ///             prepended to the next cell. Useful for inspecting accumulated
    ///             state. Returns the source as plain stream output.
    ///
    /// Declarations (fn, struct, enum, …) are accumulated across cells so
    /// later cells can reference earlier definitions.
    ///
    /// Statements are NOT accumulated — each cell's statements are run once,
    /// in the context of all prior declarations, and then discarded.  This
    /// means re-running or editing a cell never causes "already defined"
    /// errors from stale earlier runs.
    ///
    /// Returns (stdout, stderr, is_error).
    fn execute(&mut self, code: &str) -> (String, String, bool) {
        let trimmed = code.trim();

        // ── %reset ────────────────────────────────────────────────────────────
        if trimmed == "%reset" {
            let prev_count = self.execution_count;
            let prev_decls = self.declarations.len();
            self.declarations.clear();
            self.execution_count = 0;
            let msg = format!(
                "[v-kernel] Session reset.\n\
                 Cleared {prev_decls} accumulated declaration(s). \
                 Execution counter was {prev_count}, now reset to 0.\n"
            );
            return (msg, String::new(), false);
        }

        // ── %show ─────────────────────────────────────────────────────────────
        if trimmed == "%show" {
            let source = self.build_source(&[]);
            let out = if self.declarations.is_empty() {
                "[v-kernel] No declarations accumulated yet.\n".to_string()
            } else {
                format!("[v-kernel] Accumulated source ({} declaration(s)):\n\n{source}",
                    self.declarations.len())
            };
            return (out, String::new(), false);
        }

        self.execution_count += 1;

        let (new_decls, cell_stmts) = classify(code);

        // Accumulate only declarations.
        self.declarations.extend(new_decls);

        // Build the full source file for this cell.
        let source = self.build_source(&cell_stmts);

        // Write to a temp file.
        let src_path = self.tmp_dir.join(format!("cell_{}.v", self.execution_count));
        if let Err(e) = fs::write(&src_path, &source) {
            return (String::new(), format!("Failed to write source: {e}"), true);
        }

        // Run with `v run <file>`
        run_v(&src_path, self)
    }

    /// Synthesise a complete runnable V source.
    ///
    /// `cell_stmts` are the statements from the current cell only — they are
    /// NOT stored on `self` and will not appear in future cells.
    fn build_source(&self, cell_stmts: &[String]) -> String {
        let mut out = String::new();

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

        if !cell_stmts.is_empty() {
            out.push_str("fn main() {\n");
            for stmt in cell_stmts {
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

// ── dump() rich output ────────────────────────────────────────────────────────

/// A single parsed dump() entry.
struct DumpEntry {
    location: String, // e.g. "main.v:12"
    name: String,     // variable / expression name
    typ: String,      // V type string
    value: String,    // printed value
}

/// Try to parse a line as V dump() output.
///
/// V has used two different dump() output formats across versions.
///
/// Old format (pre-0.4 or so):
///   [/path/to/file.v:NN] name = TypeName(value)
///
/// Current format (0.4+):
///   [/path/to/file.v:NN] name: value
///
/// We accept both.  The distinguishing heuristic: if the rest-after-bracket
/// contains " = " before any ":" it's the old format; otherwise it's the
/// new colon format.  Type information is not included in the new format, so
/// we leave the type column blank in that case.
fn parse_dump_line(line: &str) -> Option<DumpEntry> {
    // Must start with '['
    let line = line.trim();
    if !line.starts_with('[') {
        return None;
    }

    // Find closing ']'
    let bracket_end = line.find(']')?;
    let location_raw = &line[1..bracket_end]; // e.g. "C:\\...\\cell_1.v:6"

    // The location must end with ":N" where N is a decimal line number.
    // We use rfind so that Windows drive-letter colons ("C:") are skipped.
    // The last ':' in the bracket content must be followed only by digits.
    let last_colon = location_raw.rfind(':')?;
    let line_num_part = &location_raw[last_colon + 1..];
    if line_num_part.is_empty() || !line_num_part.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    // Shorten path to basename:line for display
    let location = if let Some(slash) = location_raw.rfind(|c| c == '/' || c == '\\') {
        location_raw[slash + 1..].to_string()
    } else {
        location_raw.to_string()
    };

    // Rest after "] " (trim leading whitespace)
    let rest = line[bracket_end + 1..].trim();

    // ── Old format: "name = TypeName(value)" ─────────────────────────────────
    if let Some(eq_pos) = rest.find(" = ") {
        let name = rest[..eq_pos].trim().to_string();
        let type_value = rest[eq_pos + 3..].trim();

        let (typ, value) = if let Some(paren) = type_value.find('(') {
            let t = type_value[..paren].trim().to_string();
            let inner = &type_value[paren + 1..];
            let v = if inner.ends_with(')') {
                inner[..inner.len() - 1].to_string()
            } else {
                inner.to_string()
            };
            (t, v)
        } else {
            (String::new(), type_value.to_string())
        };

        return Some(DumpEntry { location, name, typ, value });
    }

    // ── New format: "name: value" ─────────────────────────────────────────────
    // Split on the FIRST ": " (with space) to avoid splitting on ":" inside
    // values like struct displays or Windows paths.
    if let Some(colon_pos) = rest.find(": ") {
        let name = rest[..colon_pos].trim().to_string();
        // name must be a valid identifier (non-empty, no spaces)
        if !name.is_empty() && !name.contains(' ') {
            let value = rest[colon_pos + 2..].trim().to_string();
            return Some(DumpEntry {
                location,
                name,
                typ: String::new(), // current V dump() omits the type
                value,
            });
        }
    }

    None
}

/// Escape a string for safe inclusion in HTML.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Render a list of DumpEntry values as a styled HTML table.
/// If none of the entries have a type, the type column is omitted entirely.
fn render_dump_table(entries: &[DumpEntry]) -> String {
    let show_type = entries.iter().any(|e| !e.typ.is_empty());

    let type_th = if show_type { "<th>type</th>" } else { "" };

    let mut html = format!(
        r#"<style>
.v-dump{{border-collapse:collapse;font-family:monospace;font-size:13px;margin:4px 0}}
.v-dump th{{background:#1e1e2e;color:#cdd6f4;padding:4px 10px;text-align:left;font-weight:600;border-bottom:2px solid #45475a}}
.v-dump td{{padding:3px 10px;border-bottom:1px solid #313244;vertical-align:top}}
.v-dump tr:last-child td{{border-bottom:none}}
.v-dump .loc{{color:#6c7086;font-size:11px}}
.v-dump .name{{color:#89b4fa;font-weight:600}}
.v-dump .type{{color:#a6e3a1}}
.v-dump .val{{color:#f5c2e7}}
</style>
<table class="v-dump">
<thead><tr><th>location</th><th>name</th>{type_th}<th>value</th></tr></thead>
<tbody>
"#
    );

    for e in entries {
        let type_td = if show_type {
            format!("<td class=\"type\">{}</td>", html_escape(&e.typ))
        } else {
            String::new()
        };
        html.push_str(&format!(
            "<tr><td class=\"loc\">{}</td><td class=\"name\">{}</td>{type_td}<td class=\"val\">{}</td></tr>\n",
            html_escape(&e.location),
            html_escape(&e.name),
            html_escape(&e.value),
        ));
    }

    html.push_str("</tbody></table>");
    html
}

/// Split stdout into (plain_lines, dump_entries).
/// dump() lines are removed from the plain output and returned separately.
fn split_dump_output(stdout: &str) -> (String, Vec<DumpEntry>) {
    let mut plain_lines: Vec<&str> = Vec::new();
    let mut dump_entries: Vec<DumpEntry> = Vec::new();

    for line in stdout.lines() {
        if let Some(entry) = parse_dump_line(line) {
            dump_entries.push(entry);
        } else {
            plain_lines.push(line);
        }
    }

    // Rebuild plain output, adding back the trailing newline only if the
    // original had one (to avoid spurious blank lines in Zed).
    let mut plain = plain_lines.join("\n");
    if !plain.is_empty() {
        plain.push('\n');
    }

    (plain, dump_entries)
}

// ── V code classifier ─────────────────────────────────────────────────────────

fn classify(code: &str) -> (Vec<String>, Vec<String>) {
    let mut decls = Vec::new();
    let mut stmts = Vec::new();

    let lines: Vec<&str> = code.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") {
            i += 1;
            continue;
        }

        if trimmed.starts_with("#!") {
            i += 1;
            continue;
        }

        if trimmed.starts_with("module ") {
            i += 1;
            continue;
        }

        let is_decl = is_top_level_decl(trimmed);

        if is_decl {
            let (block, consumed) = collect_block(&lines, i);
            decls.push(block);
            i += consumed;
        } else {
            let (block, consumed) = collect_statement(&lines, i);
            stmts.push(block);
            i += consumed;
        }
    }

    (decls, stmts)
}

fn is_top_level_decl(line: &str) -> bool {
    let stripped = line
        .trim_start_matches("pub ")
        .trim_start_matches("mut ")
        .trim_start_matches("static ");

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

fn collect_block(lines: &[&str], start: usize) -> (String, usize) {
    let first = lines[start];

    if !first.contains('{') {
        return (first.to_string(), 1);
    }

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

fn collect_statement(lines: &[&str], start: usize) -> (String, usize) {
    let first = lines[start];

    if !first.contains('{') {
        return (first.to_string(), 1);
    }

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

fn run_v(src: &PathBuf, state: &mut KernelState) -> (String, String, bool) {
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

    state.running_pid = Some(child.id());

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => {
            state.running_pid = None;
            return (String::new(), format!("Failed to wait on `v run`: {e}"), true);
        }
    };

    state.running_pid = None;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let raw_stderr = String::from_utf8_lossy(&output.stderr).to_string();
    // Base is_error purely on exit status. Do NOT check stdout.is_empty() —
    // dump() writes to stderr on success, so stderr is non-empty on normal runs.
    let is_error = !output.status.success() && !raw_stderr.contains("Killed");

    // Rewrite cell_N.v:LINE:COL: references in error messages so they point to
    // the line number within the cell rather than a meaningless temp filename.
    // e.g. "/tmp/v-kernel-abc/cell_3.v:7:5: error: ..." → "line 7:5: error: ..."
    let stderr = rewrite_cell_paths(&raw_stderr, src);

    (stdout, stderr, is_error)
}

/// Replace occurrences of the temp cell filename in `text` with `line N`.
///
/// The V compiler emits paths in one of two forms:
///   /full/path/to/cell_3.v:7:5: error: …      (absolute path)
///   cell_3.v:7:5: error: …                    (basename only)
///
/// Both are replaced with `line 7:5: error: …` so error messages make
/// sense in the context of the cell the user just executed.
fn rewrite_cell_paths(text: &str, src: &PathBuf) -> String {
    // Build the two patterns to replace: full path and basename.
    let full = src.to_string_lossy().to_string();
    let basename = src
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    // Replace full path first (it subsumes the basename on most systems),
    // then any remaining basename-only occurrences.
    let step1 = if !full.is_empty() {
        text.replace(&full, "cell")
    } else {
        text.to_string()
    };
    let step2 = if !basename.is_empty() && basename != full {
        step1.replace(&basename, "cell")
    } else {
        step1
    };

    // Now rewrite "cell:LINE:COL:" → "line LINE:COL:" and "cell:LINE:" → "line LINE:"
    // The V compiler always separates the location with `:` so a simple
    // prefix replacement on `cell:` is sufficient.
    step2.replace("cell:", "line ")
}

// ── Process interrupt ───────────────────────────────────────────────────────

fn interrupt_process(pid: u32) {
    #[cfg(unix)]
    {
        unsafe {
            libc::kill(pid as libc::pid_t, libc::SIGINT);
        }
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{
            OpenProcess, TerminateProcess, PROCESS_TERMINATE,
        };
        unsafe {
            let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
            if handle != 0 {
                TerminateProcess(handle, 1);
                CloseHandle(handle);
            }
        }
    }
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

    let iopub = Arc::new(Mutex::new(iopub));

    // ── Control thread ────────────────────────────────────────────────────────
    {
        let key = key.clone();
        let session_id = session_id.clone();
        let state = Arc::clone(&state);
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
                    "interrupt_request" => {
                        let pid = state.lock().unwrap().running_pid;
                        if let Some(pid) = pid {
                            interrupt_process(pid);
                            eprintln!("[v-kernel] Interrupted pid={pid}");
                        } else {
                            eprintln!("[v-kernel] interrupt_request but no child running");
                        }
                        let reply = JupyterMessage {
                            identities: msg.identities.clone(),
                            header: make_header("interrupt_reply", &session_id),
                            parent_header: msg.header.clone(),
                            metadata: json!({}),
                            content: json!({ "status": "ok" }),
                            buffers: vec![],
                        };
                        send_message(&control, &reply, &key);
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

                if !silent {
                    publish_status(&iopub, &key, &session_id, &msg, "busy");
                }

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

                let (raw_stdout, stderr, is_error) = {
                    let mut s = state.lock().unwrap();
                    s.execute(&code)
                };

                let final_exec_count = {
                    let s = state.lock().unwrap();
                    s.execution_count
                };

                // ── Split dump() lines from stdout AND stderr ─────────────────
                // V writes dump() output to stderr (not stdout). We intercept
                // dump lines from both streams and merge them into a single
                // HTML table, emitted before the plain text output.
                let (plain_stdout, mut dump_entries) = split_dump_output(&raw_stdout);
                let (plain_stderr, stderr_dump_entries) = if !is_error {
                    split_dump_output(&stderr)
                } else {
                    // Don't strip dump lines from a genuine compiler error —
                    // the whole stderr is the error message.
                    (stderr.clone(), vec![])
                };
                dump_entries.extend(stderr_dump_entries);

                // Publish plain stdout stream (non-dump lines)
                if !plain_stdout.is_empty() && !silent {
                    let stream_msg = JupyterMessage {
                        identities: vec![],
                        header: make_header("stream", &session_id),
                        parent_header: msg.header.clone(),
                        metadata: json!({}),
                        content: json!({
                            "name": "stdout",
                            "text": plain_stdout
                        }),
                        buffers: vec![],
                    };
                    let iopub = iopub.lock().unwrap();
                    send_message(&iopub, &stream_msg, &key);
                }

                // Publish dump() entries as rich HTML display_data
                if !dump_entries.is_empty() && !silent {
                    let html = render_dump_table(&dump_entries);
                    // Plain-text fallback for non-HTML frontends.
                    let plain_fallback = dump_entries
                        .iter()
                        .map(|e| {
                            if e.typ.is_empty() {
                                format!("[{}] {}: {}", e.location, e.name, e.value)
                            } else {
                                format!("[{}] {} = {}({})", e.location, e.name, e.typ, e.value)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n");

                    let display_msg = JupyterMessage {
                        identities: vec![],
                        header: make_header("display_data", &session_id),
                        parent_header: msg.header.clone(),
                        metadata: json!({}),
                        content: json!({
                            "data": {
                                "text/html": html,
                                "text/plain": plain_fallback
                            },
                            "metadata": {}
                        }),
                        buffers: vec![],
                    };
                    let iopub = iopub.lock().unwrap();
                    send_message(&iopub, &display_msg, &key);
                }

                // Publish stderr / error
                // Use plain_stderr (dump lines already extracted above).
                if is_error && !silent {
                    let stream_msg = JupyterMessage {
                        identities: vec![],
                        header: make_header("stream", &session_id),
                        parent_header: msg.header.clone(),
                        metadata: json!({}),
                        content: json!({
                            "name": "stderr",
                            "text": stderr  // full stderr for error messages
                        }),
                        buffers: vec![],
                    };
                    let iopub_lock = iopub.lock().unwrap();
                    send_message(&iopub_lock, &stream_msg, &key);
                    drop(iopub_lock);

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
                } else if !plain_stderr.is_empty() && !silent {
                    let stream_msg = JupyterMessage {
                        identities: vec![],
                        header: make_header("stream", &session_id),
                        parent_header: msg.header.clone(),
                        metadata: json!({}),
                        content: json!({
                            "name": "stderr",
                            "text": plain_stderr  // dump lines stripped
                        }),
                        buffers: vec![],
                    };
                    let iopub = iopub.lock().unwrap();
                    send_message(&iopub, &stream_msg, &key);
                }

                // Send execute_reply
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

                if !silent {
                    publish_status(&iopub, &key, &session_id, &msg, "idle");
                }
            }

            // ── is_complete_request ──────────────────────────────────────────
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

            // ── comm_info_request ────────────────────────────────────────────
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

use zed_extension_api::{
    self as zed,
    LanguageServerId, Result,
};

// GitHub API endpoint for the latest commit on the fork's added-features branch.
// We hit the commits endpoint and compare the returned SHA against what
// `v-analyzer --version` reports as its build commit.
const GITHUB_COMMITS_URL: &str =
    "https://api.github.com/repos/DaZhi-the-Revelator/velvet/commits/main";

// --- Extension state ---------------------------------------------------------

struct VEnhancedExtension {
    cached_binary_path: Option<String>,
    /// True once we have shown the update notification in this session so we
    /// don't spam the user every time a new language server is resolved.
    update_check_done: bool,
}

// --- zed::Extension impl -----------------------------------------------------

impl zed::Extension for VEnhancedExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
            update_check_done: false,
        }
    }

    // -- LSP ------------------------------------------------------------------

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary_path = self.velvet_binary_path(language_server_id, worktree)?;

        // Run the update check once per session, after we have located the binary.
        if !self.update_check_done {
            self.update_check_done = true;
            self.check_velvet_update(language_server_id, &binary_path);
        }

        Ok(zed::Command {
            command: binary_path,
            args: vec![],
            env: Default::default(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        Ok(Some(zed::serde_json::json!({
            "inlay_hints": {
                "enable": true,
                "enable_type_hints": true,
                "enable_parameter_name_hints": true,
                "enable_range_hints": true,
                "enable_implicit_err_hints": true,
                "enable_constant_type_hints": true,
                "enable_enum_field_value_hints": true
            },
            "enable_semantic_tokens": "full"
        })))
    }
}

// --- LSP helper methods ------------------------------------------------------

impl VEnhancedExtension {
    fn velvet_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        if let Some(path) = &self.cached_binary_path {
            if std::fs::metadata(path).is_ok() {
                return Ok(path.clone());
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let binary_name = if cfg!(target_os = "windows") {
            "velvet.exe"
        } else {
            "velvet"
        };

        let path = worktree.which(binary_name).ok_or_else(|| {
            "velvet not found in PATH.\n\n\
             Please install velvet:\n\
             git clone --recursive https://github.com/DaZhi-the-Revelator/velvet\n\
             cd velvet && v run build.vsh release\n\n\
             Then restart Zed."
                .to_string()
        })?;

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::None,
        );

        self.cached_binary_path = Some(path.clone());
        Ok(path)
    }

    // --- velvet update check -------------------------------------------------

    /// Compare the locally installed velvet's build commit against the
    /// latest commit SHA on the main branch via the GitHub API.
    /// If they differ, show a Zed notification prompting a rebuild.
    ///
    /// Failures are silent — a network error or malformed response simply
    /// means no notification is shown.
    fn check_velvet_update(&self, language_server_id: &LanguageServerId, binary_path: &str) {
        // 1. Ask the local binary for its version string.
        //    velvet prints a line like:
        //      velvet version 0.0.6 (commit abc1234)
        //    We extract the 7-character short SHA from that line.
        let local_commit = match self.get_local_commit(binary_path) {
            Some(c) => c,
            None => {
                // Binary didn't print a recognisable version line — skip check.
                return;
            }
        };

        // 2. Fetch the latest commit SHA from GitHub.
        let remote_sha = match self.fetch_remote_commit_sha() {
            Some(s) => s,
            None => return,
        };

        // 3. Compare.  GitHub returns the full 40-char SHA; the binary prints
        //    a 7-char prefix.  We compare only the first 7 chars.
        let remote_prefix = if remote_sha.len() >= 7 {
            &remote_sha[..7]
        } else {
            &remote_sha
        };

        if local_commit != remote_prefix {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Failed(
                    format!(
                        "velvet is out of date (local: {local_commit}, remote: {remote_prefix}). \
                         Run: cd velvet && git pull && v run build.vsh release, \
                         then copy bin/velvet to your PATH and restart Zed."
                    ),
                ),
            );
        }
    }

    /// Run `velvet --version` and extract the short commit SHA.
    /// Returns `None` if the binary cannot be run or the output is not parseable.
    fn get_local_commit(&self, binary_path: &str) -> Option<String> {
        let output = std::process::Command::new(binary_path)
            .arg("--version")
            .output()
            .ok()?;

        let text = String::from_utf8_lossy(&output.stdout).to_string();
        // Look for "(commit XXXXXXX)" or "commit: XXXXXXX"
        for line in text.lines() {
            if let Some(sha) = extract_commit_sha(line) {
                return Some(sha);
            }
        }
        // Fallback: also check stderr (some versions write there)
        let text2 = String::from_utf8_lossy(&output.stderr).to_string();
        for line in text2.lines() {
            if let Some(sha) = extract_commit_sha(line) {
                return Some(sha);
            }
        }
        None
    }

    /// GET the GitHub commits API and return the top commit SHA.
    /// Uses the zed_extension_api HTTP client so the request runs inside the
    /// WASM sandbox with Zed's proxy/trust settings.
    fn fetch_remote_commit_sha(&self) -> Option<String> {
        let request = zed::http_client::HttpRequest::builder()
            .method(zed::http_client::HttpMethod::Get)
            .url(GITHUB_COMMITS_URL)
            // GitHub API requires a User-Agent header.
            .header("User-Agent", "zed-v-enhanced")
            .header("Accept", "application/vnd.github+json")
            .redirect_policy(zed::http_client::RedirectPolicy::NoFollow)
            .build()
            .ok()?;

        let response = zed::http_client::fetch(&request).ok()?;

        // HttpResponse has no status_code field; a successful fetch() means
        // the request completed — treat an empty body as a signal to abort.
        let body = String::from_utf8(response.body).ok()?;
        // The response is a JSON object; the SHA is at .sha
        let value: zed::serde_json::Value = zed::serde_json::from_str(&body).ok()?;
        let sha = value["sha"].as_str()?.to_string();
        Some(sha)
    }
}

// --- Helpers -----------------------------------------------------------------

/// Extract a 7+ character hex commit SHA from a version string line.
/// Handles patterns like:
///   "velvet version 0.0.6 (commit abc1234)"
///   "build commit: abc1234def"
///   "abc1234"  (bare)
fn extract_commit_sha(line: &str) -> Option<String> {
    // Look for the word "commit" followed (possibly after punctuation/space) by
    // a hex string of at least 7 chars.
    let lower = line.to_lowercase();
    if let Some(pos) = lower.find("commit") {
        let after = &line[pos + "commit".len()..];
        // Skip non-hex chars to find start of SHA
        let sha_start = after.find(|c: char| c.is_ascii_hexdigit())?;
        let hex_part = &after[sha_start..];
        let sha_end = hex_part
            .find(|c: char| !c.is_ascii_hexdigit())
            .unwrap_or(hex_part.len());
        let sha = &hex_part[..sha_end];
        if sha.len() >= 7 {
            return Some(sha[..7].to_string());
        }
    }
    None
}

// --- Registration ------------------------------------------------------------

zed::register_extension!(VEnhancedExtension);

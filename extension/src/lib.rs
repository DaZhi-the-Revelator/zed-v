use zed_extension_api::{
    self as zed,
    LanguageServerId, Result,
};

// GitHub API endpoint for the latest release tag.
// We compare the tag name against the version string from `velvet --version`
// so that only actual releases trigger the update notification, not every
// commit to main.
const GITHUB_RELEASES_URL: &str =
    "https://api.github.com/repos/DaZhi-the-Revelator/velvet/releases/latest";

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
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        // Start with the built-in defaults.
        let mut options = zed::serde_json::json!({
            "inlay_hints": {
                "enable": true,
                "enable_type_hints": true,
                "enable_parameter_name_hints": true,
                "enable_range_hints": true,
                "enable_implicit_err_hints": true,
                "enable_constant_type_hints": true,
                "enable_enum_field_value_hints": true,
                "enable_anon_fn_return_type_hints": true
            },
            "enable_semantic_tokens": "full",
            "code_lens": {
                "enable": true,
                "enable_run_lens": true,
                "enable_inheritors_lens": true,
                "enable_super_interfaces_lens": true,
                "enable_run_tests_lens": true
            },
            "inspections": {
                "enable_unused_parameter_warning": true
            }
        });

        // Merge any user-supplied initialization_options from settings.json on
        // top of the defaults.  This lets users override individual keys (e.g.
        // "inspections.enable_unused_parameter_warning") without having to
        // repeat the whole block.
        if let Ok(lsp_settings) = zed::settings::LspSettings::for_worktree("velvet", worktree) {
            if let Some(user_options) = lsp_settings.initialization_options {
                merge_json(&mut options, user_options);
            }
        }

        Ok(Some(options))
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

        // Check lsp.velvet.binary.path from Zed settings.json first.
        // This lets users point at a non-PATH install without needing to
        // add it to their shell PATH.
        let path = if let Ok(lsp_settings) = zed::settings::LspSettings::for_worktree("velvet", worktree) {
            if let Some(binary) = lsp_settings.binary {
                if let Some(configured_path) = binary.path {
                    if std::fs::metadata(&configured_path).is_ok() {
                        configured_path
                    } else {
                        return Err(format!(
                            "velvet binary not found at configured path: {configured_path}\n\
                             Check lsp.velvet.binary.path in your Zed settings.json."
                        ));
                    }
                } else {
                    self.find_velvet_in_path(worktree)?
                }
            } else {
                self.find_velvet_in_path(worktree)?
            }
        } else {
            self.find_velvet_in_path(worktree)?
        };

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::None,
        );

        self.cached_binary_path = Some(path.clone());
        Ok(path)
    }

    fn find_velvet_in_path(&self, worktree: &zed::Worktree) -> Result<String> {
        let binary_name = if cfg!(target_os = "windows") {
            "velvet.exe"
        } else {
            "velvet"
        };

        worktree.which(binary_name).ok_or_else(|| {
            "velvet not found in PATH.\n\n\
             Please install velvet:\n\
             git clone --recursive https://github.com/DaZhi-the-Revelator/velvet\n\
             cd velvet && v run build.vsh release\n\n\
             Then copy bin/velvet to your PATH, or set lsp.velvet.binary.path in\n\
             your Zed settings.json."
                .to_string()
        })
    }

    // --- velvet update check -------------------------------------------------

    /// Compare the locally installed velvet's version against the latest
    /// GitHub release tag. Only actual releases trigger the notification,
    /// not every commit to main.
    ///
    /// Failures are silent — a network error or malformed response simply
    /// means no notification is shown.
    fn check_velvet_update(&self, language_server_id: &LanguageServerId, binary_path: &str) {
        // 1. Ask the local binary for its version string.
        //    velvet prints a line like: velvet version 0.1.0
        let local_version = match self.get_local_version(binary_path) {
            Some(v) => v,
            None => return,
        };

        // 2. Fetch the latest release tag name from GitHub.
        let remote_version = match self.fetch_remote_release_tag() {
            Some(v) => v,
            None => return,
        };

        // 3. Strip leading 'v' from tag if present (e.g. "v0.1.0" -> "0.1.0")
        let remote_clean = remote_version.trim_start_matches('v');
        let local_clean = local_version.trim_start_matches('v');

        if local_clean != remote_clean {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Failed(
                    format!(
                        "velvet is out of date (local: {local_clean}, latest release: {remote_clean}). \
                         Run: cd velvet && git pull && v run build.vsh release, \
                         then copy bin/velvet to your PATH and restart Zed."
                    ),
                ),
            );
        }
    }

    /// Run `velvet --version` and extract the version string.
    /// Handles output like "velvet version 0.1.0" or "velvet 0.1.0".
    /// Returns `None` if the binary cannot be run or the output is not parseable.
    fn get_local_version(&self, binary_path: &str) -> Option<String> {
        let output = std::process::Command::new(binary_path)
            .arg("--version")
            .output()
            .ok()?;

        let text = String::from_utf8_lossy(&output.stdout).to_string();
        for line in text.lines() {
            if let Some(v) = extract_version(line) {
                return Some(v);
            }
        }
        let text2 = String::from_utf8_lossy(&output.stderr).to_string();
        for line in text2.lines() {
            if let Some(v) = extract_version(line) {
                return Some(v);
            }
        }
        None
    }

    /// GET the GitHub releases API and return the latest release tag name.
    /// Uses the zed_extension_api HTTP client so the request runs inside the
    /// WASM sandbox with Zed's proxy/trust settings.
    fn fetch_remote_release_tag(&self) -> Option<String> {
        let request = zed::http_client::HttpRequest::builder()
            .method(zed::http_client::HttpMethod::Get)
            .url(GITHUB_RELEASES_URL)
            .header("User-Agent", "zed-v-enhanced")
            .header("Accept", "application/vnd.github+json")
            .redirect_policy(zed::http_client::RedirectPolicy::NoFollow)
            .build()
            .ok()?;

        let response = zed::http_client::fetch(&request).ok()?;
        let body = String::from_utf8(response.body).ok()?;
        let value: zed::serde_json::Value = zed::serde_json::from_str(&body).ok()?;
        // The releases/latest response has a "tag_name" field, e.g. "v0.1.0"
        let tag = value["tag_name"].as_str()?.to_string();
        Some(tag)
    }
}

// --- Helpers -----------------------------------------------------------------

/// Recursively merge `src` into `dst`.  Object keys in `src` overwrite keys in
/// `dst`; for nested objects the merge is recursive so individual sub-keys can
/// be overridden without replacing the whole object.
fn merge_json(dst: &mut zed::serde_json::Value, src: zed::serde_json::Value) {
    if let (Some(dst_obj), Some(src_obj)) = (dst.as_object_mut(), src.as_object()) {
        for (key, src_val) in src_obj {
            let dst_entry = dst_obj.entry(key.clone()).or_insert(zed::serde_json::Value::Null);
            if dst_entry.is_object() && src_val.is_object() {
                merge_json(dst_entry, src_val.clone());
            } else {
                *dst_entry = src_val.clone();
            }
        }
    }
}

/// Extract a semver-style version string from a line of `velvet --version` output.
/// Handles patterns like:
///   "velvet version 0.1.0"
///   "velvet 0.1.0"
///   "0.1.0"
fn extract_version(line: &str) -> Option<String> {
    // Find the first token that looks like N.N.N (or N.N)
    for token in line.split_whitespace() {
        let t = token.trim_start_matches('v');
        let parts: Vec<&str> = t.split('.').collect();
        if parts.len() >= 2 && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())) {
            return Some(t.to_string());
        }
    }
    None
}

// --- Registration ------------------------------------------------------------

zed::register_extension!(VEnhancedExtension);

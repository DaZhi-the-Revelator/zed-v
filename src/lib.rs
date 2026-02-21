use zed_extension_api::{self as zed, LanguageServerId, Result};

// --- Extension state ---------------------------------------------------------

struct VEnhancedExtension {
    cached_binary_path: Option<String>,
}

// --- zed::Extension impl -----------------------------------------------------

impl zed::Extension for VEnhancedExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    // -- LSP ------------------------------------------------------------------

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary_path = self.v_analyzer_binary_path(language_server_id, worktree)?;
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
    fn v_analyzer_binary_path(
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
            "v-analyzer.exe"
        } else {
            "v-analyzer"
        };

        let path = worktree.which(binary_name).ok_or_else(|| {
            "v-analyzer not found in PATH.\n\n\
             Please install the forked v-analyzer:\n\
             git clone --branch added-features --recursive \
             https://github.com/DaZhi-the-Revelator/v-analyzer\n\
             cd v-analyzer && v run build.vsh release\n\n\
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
}

// --- Registration ------------------------------------------------------------

zed::register_extension!(VEnhancedExtension);

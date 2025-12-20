//! Configuration management for Shard.
//!
//! Loads and saves configuration from `~/.config/shard/config.toml` (or platform equivalent).

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Editor preset with predefined commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EditorPreset {
    /// Visual Studio Code: `code --wait {file}`
    #[default]
    Vscode,
    /// Helix: `hx {file}`
    Helix,
    /// Neovim: `nvim {file}`
    Neovim,
    /// Vim: `vim {file}`
    Vim,
    /// Custom command specified in `editor_command`
    Custom,
}

impl EditorPreset {
    /// Get the default command for this preset.
    /// Returns None for Custom (uses editor_command instead).
    pub fn default_command(&self) -> Option<&'static str> {
        match self {
            EditorPreset::Vscode => Some("code --wait {file}"),
            EditorPreset::Helix => Some("hx {file}"),
            EditorPreset::Neovim => Some("nvim {file}"),
            EditorPreset::Vim => Some("vim {file}"),
            EditorPreset::Custom => None,
        }
    }

    /// Get display name for the preset.
    pub fn display_name(&self) -> &'static str {
        match self {
            EditorPreset::Vscode => "VS Code",
            EditorPreset::Helix => "Helix",
            EditorPreset::Neovim => "Neovim",
            EditorPreset::Vim => "Vim",
            EditorPreset::Custom => "Custom",
        }
    }
}

/// External editor configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    /// Which preset to use.
    #[serde(default)]
    pub preset: EditorPreset,

    /// Custom command (used when preset is Custom).
    /// Use `{file}` as placeholder for the file path.
    #[serde(default)]
    pub custom_command: String,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            preset: EditorPreset::Helix,
            custom_command: String::new(),
        }
    }
}

impl EditorConfig {
    /// Get the effective command to run.
    /// Returns the preset command or custom command.
    pub fn effective_command(&self) -> &str {
        match self.preset {
            EditorPreset::Custom => &self.custom_command,
            preset => preset.default_command().unwrap_or("code --wait {file}"),
        }
    }

    /// Build the command line for opening a file.
    /// Returns (program, args) tuple.
    pub fn build_command(&self, file_path: &str) -> Option<(String, Vec<String>)> {
        let cmd_template = self.effective_command();
        if cmd_template.is_empty() {
            return None;
        }

        let cmd_with_file = cmd_template.replace("{file}", file_path);
        let parts: Vec<&str> = cmd_with_file.split_whitespace().collect();

        if parts.is_empty() {
            return None;
        }

        let program = parts[0].to_string();
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        Some((program, args))
    }
}

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// External editor settings.
    #[serde(default)]
    pub editor: EditorConfig,
}

impl Config {
    /// Get the config file path.
    pub fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "shard").map(|dirs| dirs.config_dir().join("config.toml"))
    }

    /// Load configuration from file, or return defaults if not found.
    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Self::default();
        };

        match fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save configuration to file.
    pub fn save(&self) -> Result<(), String> {
        let Some(path) = Self::config_path() else {
            return Err("Could not determine config directory".to_string());
        };

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, content).map_err(|e| e.to_string())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_preset_commands() {
        assert_eq!(
            EditorPreset::Vscode.default_command(),
            Some("code --wait {file}")
        );
        assert_eq!(EditorPreset::Helix.default_command(), Some("hx {file}"));
        assert_eq!(EditorPreset::Neovim.default_command(), Some("nvim {file}"));
        assert_eq!(EditorPreset::Custom.default_command(), None);
    }

    #[test]
    fn test_build_command() {
        let config = EditorConfig {
            preset: EditorPreset::Vscode,
            custom_command: String::new(),
        };

        let (prog, args) = config.build_command("/tmp/test.rs").unwrap();
        assert_eq!(prog, "code");
        assert_eq!(args, vec!["--wait", "/tmp/test.rs"]);
    }

    #[test]
    fn test_custom_command() {
        let config = EditorConfig {
            preset: EditorPreset::Custom,
            custom_command: "subl -w {file}".to_string(),
        };

        let (prog, args) = config.build_command("/tmp/test.txt").unwrap();
        assert_eq!(prog, "subl");
        assert_eq!(args, vec!["-w", "/tmp/test.txt"]);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.editor.preset, EditorPreset::Vscode);
    }
}

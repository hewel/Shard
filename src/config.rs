//! Configuration management for Shard.
//!
//! Loads and saves configuration from `~/.config/shard/config.toml` (or platform equivalent).

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::PathBuf;

use crate::view::PickerMode;

// === Keyboard Shortcuts ===

/// Modifier keys for a shortcut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Modifiers {
    pub const fn new(ctrl: bool, alt: bool, shift: bool) -> Self {
        Self { ctrl, alt, shift }
    }

    pub const fn ctrl() -> Self {
        Self::new(true, false, false)
    }

    pub const fn none() -> Self {
        Self::new(false, false, false)
    }

    /// Check if iced modifiers match this config.
    pub fn matches(&self, mods: iced::keyboard::Modifiers) -> bool {
        self.ctrl == mods.command() && self.alt == mods.alt() && self.shift == mods.shift()
    }
}

impl fmt::Display for Modifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        write!(f, "{}", parts.join("+"))
    }
}

/// A keyboard shortcut (key + modifiers).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Shortcut {
    /// The key (e.g., "v", "n", "Delete", "Escape").
    pub key: String,
    /// Modifier keys.
    #[serde(default)]
    pub modifiers: Modifiers,
}

impl Shortcut {
    pub const fn new(key: String, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }

    /// Create from a character key with modifiers.
    pub fn char_key(c: char, modifiers: Modifiers) -> Self {
        Self {
            key: c.to_string(),
            modifiers,
        }
    }

    /// Create from a named key (e.g., "Delete", "Escape").
    pub fn named(name: &str, modifiers: Modifiers) -> Self {
        Self {
            key: name.to_string(),
            modifiers,
        }
    }

    /// Check if this shortcut matches a key press.
    pub fn matches(&self, key: &iced::keyboard::Key, modifiers: iced::keyboard::Modifiers) -> bool {
        if !self.modifiers.matches(modifiers) {
            return false;
        }

        match key {
            iced::keyboard::Key::Character(c) => c.to_lowercase().eq(&self.key.to_lowercase()),
            iced::keyboard::Key::Named(named) => {
                let named_str = format!("{:?}", named);
                named_str.eq_ignore_ascii_case(&self.key)
            }
            _ => false,
        }
    }
}

impl fmt::Display for Shortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mods = self.modifiers.to_string();
        if mods.is_empty() {
            write!(f, "{}", self.key)
        } else {
            write!(f, "{}+{}", mods, self.key)
        }
    }
}

/// Available shortcut actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShortcutAction {
    Paste,
    NewColor,
    Escape,
    Delete,
    CopySnippet,
}

impl ShortcutAction {
    /// Get display name for the action.
    pub fn display_name(&self) -> &'static str {
        match self {
            ShortcutAction::Paste => "Paste / Add Snippet",
            ShortcutAction::NewColor => "New Color Input",
            ShortcutAction::Escape => "Close / Cancel",
            ShortcutAction::Delete => "Delete Selected",
            ShortcutAction::CopySnippet => "Copy Snippet",
        }
    }

    /// All available actions.
    pub const ALL: [ShortcutAction; 5] = [
        ShortcutAction::Paste,
        ShortcutAction::NewColor,
        ShortcutAction::Escape,
        ShortcutAction::Delete,
        ShortcutAction::CopySnippet,
    ];
}

/// Keyboard shortcuts configuration.
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct KeyboardConfig {
    pub paste: Shortcut,
    pub new_color: Shortcut,
    pub escape: Shortcut,
    pub delete: Shortcut,
    #[serde(default = "default_copy_snippet_shortcut")]
    pub copy_snippet: Shortcut,
}

fn default_copy_snippet_shortcut() -> Shortcut {
    Shortcut::char_key('c', Modifiers::ctrl())
}

impl Default for KeyboardConfig {
    fn default() -> Self {
        Self {
            paste: Shortcut::char_key('v', Modifiers::ctrl()),
            new_color: Shortcut::char_key('n', Modifiers::ctrl()),
            escape: Shortcut::named("Escape", Modifiers::none()),
            delete: Shortcut::named("Delete", Modifiers::none()),
            copy_snippet: default_copy_snippet_shortcut(),
        }
    }
}

impl KeyboardConfig {
    /// Get shortcut for an action.
    pub fn get(&self, action: ShortcutAction) -> &Shortcut {
        match action {
            ShortcutAction::Paste => &self.paste,
            ShortcutAction::NewColor => &self.new_color,
            ShortcutAction::Escape => &self.escape,
            ShortcutAction::Delete => &self.delete,
            ShortcutAction::CopySnippet => &self.copy_snippet,
        }
    }

    /// Set shortcut for an action.
    pub fn set(&mut self, action: ShortcutAction, shortcut: Shortcut) {
        match action {
            ShortcutAction::Paste => self.paste = shortcut,
            ShortcutAction::NewColor => self.new_color = shortcut,
            ShortcutAction::Escape => self.escape = shortcut,
            ShortcutAction::Delete => self.delete = shortcut,
            ShortcutAction::CopySnippet => self.copy_snippet = shortcut,
        }
    }
}

// === Editor Configuration ===

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

    /// Keyboard shortcuts.
    #[serde(default)]
    pub keyboard: KeyboardConfig,

    /// Default color picker mode (HSL or OKLCH).
    #[serde(default)]
    pub default_picker_mode: PickerMode,
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
        assert_eq!(parsed.editor.preset, EditorPreset::Helix);
    }
}

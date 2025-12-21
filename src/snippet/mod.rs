//! Unified snippet module for managing different content types.
//!
//! Supports three snippet kinds:
//! - **Color**: Color values with RGBA components
//! - **Code**: Code snippets with syntax highlighting
//! - **Text**: Plain text notes

use nanoid::nanoid;
use serde::Serialize;

mod code;
mod color;
mod text;

pub use code::{detect_language, language_to_extension, CodeData};
pub use color::{
    extract_colors_from_text, hsl_to_rgb, oklch_to_rgb, rgb_to_hsl, rgb_to_oklch, ColorData,
};
pub use text::TextData;

/// The type of snippet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SnippetKind {
    Color,
    Code,
    Text,
}

impl SnippetKind {
    /// Get the display name for this snippet kind.
    pub fn display_name(&self) -> &'static str {
        match self {
            SnippetKind::Color => "Color",
            SnippetKind::Code => "Code",
            SnippetKind::Text => "Text",
        }
    }

    /// Get the database string for this snippet kind.
    pub fn as_db_str(&self) -> &'static str {
        match self {
            SnippetKind::Color => "color",
            SnippetKind::Code => "code",
            SnippetKind::Text => "text",
        }
    }

    /// Parse from database string.
    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "color" => Some(SnippetKind::Color),
            "code" => Some(SnippetKind::Code),
            "text" => Some(SnippetKind::Text),
            _ => None,
        }
    }
}

/// Content varies by snippet type.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SnippetContent {
    Color(ColorData),
    Code(CodeData),
    Text(TextData),
}

impl SnippetContent {
    /// Get the kind of this content.
    pub fn kind(&self) -> SnippetKind {
        match self {
            SnippetContent::Color(_) => SnippetKind::Color,
            SnippetContent::Code(_) => SnippetKind::Code,
            SnippetContent::Text(_) => SnippetKind::Text,
        }
    }

    /// Get a preview string for display (first line or summary).
    pub fn preview(&self, max_len: usize) -> String {
        match self {
            SnippetContent::Color(c) => c.to_hex(),
            SnippetContent::Code(c) => {
                let first_line = c.code.lines().next().unwrap_or("");
                if first_line.len() > max_len {
                    format!("{}...", &first_line[..max_len])
                } else {
                    first_line.to_string()
                }
            }
            SnippetContent::Text(t) => {
                let first_line = t.text.lines().next().unwrap_or("");
                if first_line.len() > max_len {
                    format!("{}...", &first_line[..max_len])
                } else {
                    first_line.to_string()
                }
            }
        }
    }

    /// Get the copyable text representation.
    pub fn to_copyable_string(&self) -> String {
        match self {
            SnippetContent::Color(c) => c.to_hex(),
            SnippetContent::Code(c) => c.code.clone(),
            SnippetContent::Text(t) => t.text.clone(),
        }
    }
}

/// A unified snippet that can hold different content types.
#[derive(Debug, Clone, Serialize)]
pub struct Snippet {
    pub id: i64,
    pub label: String,
    pub content: SnippetContent,
    #[serde(skip)]
    pub position: i64,
}

impl Snippet {
    /// Create a new snippet with auto-generated ID (0 = not yet saved).
    pub fn new(label: String, content: SnippetContent) -> Self {
        Self {
            id: 0,
            label,
            content,
            position: 0,
        }
    }

    /// Get the kind of this snippet.
    pub fn kind(&self) -> SnippetKind {
        self.content.kind()
    }

    /// Create a new color snippet.
    pub fn color(r: u8, g: u8, b: u8, a: f32, label: String) -> Self {
        let color_data = ColorData::new(r, g, b, a);
        let label = if label.is_empty() { nanoid!(8) } else { label };
        Self::new(label, SnippetContent::Color(color_data))
    }

    /// Create a new code snippet.
    pub fn code(code: String, language: String, label: String) -> Self {
        let label = if label.is_empty() { nanoid!(8) } else { label };
        Self::new(label, SnippetContent::Code(CodeData::new(code, language)))
    }

    /// Create a new text snippet.
    pub fn text(text: String, label: String) -> Self {
        let label = if label.is_empty() { nanoid!(8) } else { label };
        Self::new(label, SnippetContent::Text(TextData::new(text)))
    }

    /// Get the default label for this snippet based on content.
    pub fn default_label(&self) -> String {
        nanoid!(8)
    }

    /// Check if this snippet matches a filter string.
    pub fn matches_filter(&self, filter: &str) -> bool {
        if filter.is_empty() {
            return true;
        }
        let filter_lower = filter.to_lowercase();

        // Check label
        if self.label.to_lowercase().contains(&filter_lower) {
            return true;
        }

        // Check content-specific matching
        match &self.content {
            SnippetContent::Color(c) => {
                // Match hex, rgb string
                c.to_hex().to_lowercase().contains(&filter_lower)
                    || c.to_rgb().to_lowercase().contains(&filter_lower)
            }
            SnippetContent::Code(c) => {
                c.code.to_lowercase().contains(&filter_lower)
                    || c.language.to_lowercase().contains(&filter_lower)
            }
            SnippetContent::Text(t) => t.text.to_lowercase().contains(&filter_lower),
        }
    }
}

/// Detect what kind of snippet the given text might be.
pub fn detect_snippet_type(text: &str) -> Option<SnippetKind> {
    let trimmed = text.trim();

    // 1. Try color formats first (most specific patterns)
    if !extract_colors_from_text(trimmed).is_empty() {
        return Some(SnippetKind::Color);
    }

    // 2. Detect code (heuristics)
    if code::looks_like_code(trimmed) {
        return Some(SnippetKind::Code);
    }

    // 3. Default to text if multi-line or substantial
    if trimmed.lines().count() > 1 || trimmed.len() > 20 {
        return Some(SnippetKind::Text);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snippet_kind_db_roundtrip() {
        for kind in [SnippetKind::Color, SnippetKind::Code, SnippetKind::Text] {
            let s = kind.as_db_str();
            let parsed = SnippetKind::from_db_str(s).unwrap();
            assert_eq!(kind, parsed);
        }
    }

    #[test]
    fn test_detect_color_snippet() {
        assert_eq!(detect_snippet_type("#FF5733"), Some(SnippetKind::Color));
        assert_eq!(
            detect_snippet_type("rgb(255, 0, 0)"),
            Some(SnippetKind::Color)
        );
    }

    #[test]
    fn test_detect_code_snippet() {
        assert_eq!(
            detect_snippet_type("fn main() {\n    println!(\"Hello\");\n}"),
            Some(SnippetKind::Code)
        );
        assert_eq!(
            detect_snippet_type("def hello():\n    print('world')"),
            Some(SnippetKind::Code)
        );
    }

    #[test]
    fn test_detect_text_snippet() {
        assert_eq!(
            detect_snippet_type("This is a longer piece of text that should be detected as text"),
            Some(SnippetKind::Text)
        );
        assert_eq!(
            detect_snippet_type("Line one\nLine two"),
            Some(SnippetKind::Text)
        );
    }

    #[test]
    fn test_snippet_matches_filter() {
        let snippet = Snippet::color(255, 87, 51, 1.0, "Orange".to_string());
        assert!(snippet.matches_filter("orange"));
        assert!(snippet.matches_filter("FF5733"));
        assert!(!snippet.matches_filter("blue"));
    }
}

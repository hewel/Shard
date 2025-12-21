//! Text snippet data.

use serde::Serialize;

/// Plain text data.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TextData {
    pub text: String,
}

impl TextData {
    /// Create new text data.
    pub fn new(text: String) -> Self {
        Self { text }
    }

    /// Get the number of lines in the text.
    pub fn line_count(&self) -> usize {
        self.text.lines().count()
    }

    /// Get the character count.
    pub fn char_count(&self) -> usize {
        self.text.chars().count()
    }

    /// Get a preview of the text (first few lines).
    pub fn preview(&self, max_lines: usize) -> String {
        self.text
            .lines()
            .take(max_lines)
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Check if the text is empty or whitespace only.
    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_data_new() {
        let text = TextData::new("Hello, world!".to_string());
        assert_eq!(text.text, "Hello, world!");
    }

    #[test]
    fn test_line_count() {
        let text = TextData::new("line 1\nline 2\nline 3".to_string());
        assert_eq!(text.line_count(), 3);
    }

    #[test]
    fn test_char_count() {
        let text = TextData::new("Hello".to_string());
        assert_eq!(text.char_count(), 5);
    }

    #[test]
    fn test_preview() {
        let text = TextData::new("line 1\nline 2\nline 3\nline 4".to_string());
        assert_eq!(text.preview(2), "line 1\nline 2");
    }

    #[test]
    fn test_is_empty() {
        assert!(TextData::new("".to_string()).is_empty());
        assert!(TextData::new("   ".to_string()).is_empty());
        assert!(!TextData::new("hello".to_string()).is_empty());
    }
}

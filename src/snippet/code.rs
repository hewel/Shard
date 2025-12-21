//! Code snippet data and language detection.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// Code data with content and language.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeData {
    pub code: String,
    pub language: String,
}

impl CodeData {
    /// Create new code data.
    pub fn new(code: String, language: String) -> Self {
        let language = if language.is_empty() {
            detect_language(&code)
        } else {
            language
        };
        Self { code, language }
    }

    /// Get the number of lines in the code.
    pub fn line_count(&self) -> usize {
        self.code.lines().count()
    }

    /// Get a preview of the code (first few lines).
    pub fn preview(&self, max_lines: usize) -> String {
        self.code
            .lines()
            .take(max_lines)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// Language detection patterns
static RUST_PATTERNS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(fn\s+\w+|impl\s+|struct\s+|enum\s+|trait\s+|mod\s+|use\s+|let\s+mut|pub\s+fn|#\[derive)")
        .expect("Invalid rust regex")
});

static PYTHON_PATTERNS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(def\s+\w+|class\s+\w+|import\s+|from\s+\w+\s+import|if\s+__name__|@\w+)")
        .expect("Invalid python regex")
});

static JAVASCRIPT_PATTERNS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)(function\s+\w+|const\s+\w+\s*=|let\s+\w+\s*=|var\s+\w+\s*=|=>\s*\{|export\s+(default\s+)?|import\s+.*from|require\()")
        .expect("Invalid javascript regex")
});

static TYPESCRIPT_PATTERNS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)(:\s*(string|number|boolean|any|void|never)|interface\s+\w+|type\s+\w+\s*=|<\w+>)",
    )
    .expect("Invalid typescript regex")
});

static JSON_PATTERNS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^\s*[\{\[]"#).expect("Invalid json regex"));

static HTML_PATTERNS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*<!DOCTYPE|<html|<div|<span|<p\s|<a\s|<script|<style")
        .expect("Invalid html regex")
});

static CSS_PATTERNS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(\.|#|@media|@keyframes|[a-z-]+\s*:\s*[^;]+;)").expect("Invalid css regex")
});

static SQL_PATTERNS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*(SELECT|INSERT|UPDATE|DELETE|CREATE|ALTER|DROP|FROM|WHERE|JOIN)")
        .expect("Invalid sql regex")
});

static SHELL_PATTERNS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^(\s*#!\/bin\/(ba)?sh|^\s*\$\s+|\becho\s+|\bcd\s+|\bls\s|\bmkdir\s|\bgrep\s|\bsed\s)",
    )
    .expect("Invalid shell regex")
});

static GO_PATTERNS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(package\s+\w+|func\s+\w+|import\s+\(|type\s+\w+\s+struct)")
        .expect("Invalid go regex")
});

/// Detect the programming language of a code snippet.
pub fn detect_language(code: &str) -> String {
    // Check patterns in order of specificity

    // TypeScript before JavaScript (more specific)
    if TYPESCRIPT_PATTERNS.is_match(code) && JAVASCRIPT_PATTERNS.is_match(code) {
        return "typescript".to_string();
    }

    if RUST_PATTERNS.is_match(code) {
        return "rust".to_string();
    }

    if PYTHON_PATTERNS.is_match(code) {
        return "python".to_string();
    }

    if GO_PATTERNS.is_match(code) {
        return "go".to_string();
    }

    if JAVASCRIPT_PATTERNS.is_match(code) {
        return "javascript".to_string();
    }

    // JSON check - must be valid-ish JSON structure
    if JSON_PATTERNS.is_match(code) {
        let trimmed = code.trim();
        if (trimmed.starts_with('{') && trimmed.ends_with('}'))
            || (trimmed.starts_with('[') && trimmed.ends_with(']'))
        {
            return "json".to_string();
        }
    }

    if HTML_PATTERNS.is_match(code) {
        return "html".to_string();
    }

    if CSS_PATTERNS.is_match(code) {
        return "css".to_string();
    }

    if SQL_PATTERNS.is_match(code) {
        return "sql".to_string();
    }

    if SHELL_PATTERNS.is_match(code) {
        return "shell".to_string();
    }

    "plain".to_string()
}

/// Map language name to file extension for syntax highlighting.
pub fn language_to_extension(language: &str) -> &'static str {
    match language.to_lowercase().as_str() {
        "rust" => "rs",
        "python" => "py",
        "javascript" => "js",
        "typescript" => "ts",
        "json" => "json",
        "html" => "html",
        "css" => "css",
        "sql" => "sql",
        "shell" | "bash" | "sh" => "sh",
        "go" => "go",
        "c" => "c",
        "cpp" | "c++" => "cpp",
        "java" => "java",
        "ruby" => "rb",
        "php" => "php",
        "swift" => "swift",
        "kotlin" => "kt",
        "scala" => "scala",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "markdown" | "md" => "md",
        "xml" => "xml",
        _ => "txt",
    }
}

/// Check if text looks like code (heuristics).
pub fn looks_like_code(text: &str) -> bool {
    let trimmed = text.trim();

    // Too short to be meaningful code
    if trimmed.len() < 10 {
        return false;
    }

    // Check for common code indicators
    let indicators = [
        // Braces and brackets patterns
        (r"\{[\s\S]*\}", 5), // Contains { ... }
        (r"\[[\s\S]*\]", 3), // Contains [ ... ]
        (r"\([\s\S]*\)", 2), // Contains ( ... )
        // Line endings
        (r";\s*$", 4), // Ends with semicolon
        (r":\s*$", 3), // Ends with colon (Python, YAML)
        // Keywords
        (
            r"\b(fn|func|function|def|class|struct|enum|impl|trait|interface|type)\b",
            5,
        ),
        (r"\b(if|else|for|while|loop|match|switch|case)\b", 3),
        (r"\b(return|break|continue|yield)\b", 3),
        (r"\b(import|from|use|require|include)\b", 4),
        (r"\b(const|let|var|mut)\b", 3),
        (r"\b(pub|private|public|protected|static)\b", 3),
        // Operators
        (r"=>", 3),          // Arrow function
        (r"->", 3),          // Return type / arrow
        (r"::", 3),          // Namespace separator
        (r"==|!=|<=|>=", 2), // Comparison operators
        // Comments
        (r"//.*$", 3),         // Single line comment
        (r"/\*[\s\S]*\*/", 3), // Multi-line comment
        (r"#.*$", 2),          // Hash comment (Python, Shell, etc.)
    ];

    let mut score = 0;

    for (pattern, weight) in indicators {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(trimmed) {
                score += weight;
            }
        }
    }

    // Additional heuristics
    let lines: Vec<&str> = trimmed.lines().collect();

    // Multiple lines with consistent indentation suggest code
    if lines.len() > 1 {
        let indented_lines = lines
            .iter()
            .filter(|l| l.starts_with("  ") || l.starts_with("\t"))
            .count();
        if indented_lines > lines.len() / 3 {
            score += 5;
        }
    }

    // High ratio of special characters suggests code
    let special_chars = trimmed
        .chars()
        .filter(|c| "{}[]();:=<>+-*/&|!@#$%^".contains(*c))
        .count();
    let special_ratio = special_chars as f32 / trimmed.len() as f32;
    if special_ratio > 0.05 {
        score += 3;
    }

    score >= 8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_rust() {
        let code = r#"
fn main() {
    println!("Hello, world!");
}
"#;
        assert_eq!(detect_language(code), "rust");
    }

    #[test]
    fn test_detect_python() {
        let code = r#"
def hello():
    print("Hello, world!")

if __name__ == "__main__":
    hello()
"#;
        assert_eq!(detect_language(code), "python");
    }

    #[test]
    fn test_detect_javascript() {
        let code = r#"
const greeting = () => {
    console.log("Hello, world!");
};
"#;
        assert_eq!(detect_language(code), "javascript");
    }

    #[test]
    fn test_detect_json() {
        let code = r#"{"name": "test", "value": 42}"#;
        assert_eq!(detect_language(code), "json");
    }

    #[test]
    fn test_looks_like_code() {
        assert!(looks_like_code("fn main() {\n    println!(\"Hello\");\n}"));
        assert!(looks_like_code("def hello():\n    print('world')"));
        assert!(!looks_like_code("Hello world"));
        assert!(!looks_like_code("Short"));
    }

    #[test]
    fn test_code_data_preview() {
        let code = CodeData::new(
            "line 1\nline 2\nline 3\nline 4\nline 5".to_string(),
            "plain".to_string(),
        );
        assert_eq!(code.preview(2), "line 1\nline 2");
        assert_eq!(code.line_count(), 5);
    }
}

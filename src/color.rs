//! Color parsing and conversion utilities.
//!
//! Supports parsing colors from:
//! - Hex: `#RGB`, `#RRGGBB`, `#RRGGBBAA`
//! - RGB: `rgb(r, g, b)`, `rgba(r, g, b, a)`
//! - HSL: `hsl(h, s%, l%)`, `hsla(h, s%, l%, a)`
//! - OKLCH: `oklch(l% c h)`, `oklch(l% c h / a)`

use regex::Regex;
use std::sync::LazyLock;

/// A color with RGBA components.
#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    pub id: i64,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32, // 0.0 to 1.0
    pub label: String,
}

impl Color {
    /// Create a new color with the given RGBA values.
    pub fn new(r: u8, g: u8, b: u8, a: f32, label: String) -> Self {
        Self {
            id: 0,
            r,
            g,
            b,
            a: a.clamp(0.0, 1.0),
            label,
        }
    }

    /// Parse a color from a string. Supports hex, rgb, rgba, hsl, hsla formats.
    pub fn parse(input: &str) -> Result<Self, ColorParseError> {
        let input = input.trim();

        if let Some(color) = parse_hex(input) {
            return Ok(color);
        }

        if let Some(color) = parse_rgb(input) {
            return Ok(color);
        }

        if let Some(color) = parse_hsl(input) {
            return Ok(color);
        }

        if let Some(color) = parse_oklch(input) {
            return Ok(color);
        }

        Err(ColorParseError::InvalidFormat(input.to_string()))
    }

    /// Convert to hex string (with alpha if not fully opaque).
    pub fn to_hex(&self) -> String {
        if (self.a - 1.0).abs() < f32::EPSILON {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            let alpha = (self.a * 255.0).round() as u8;
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, alpha)
        }
    }

    /// Convert to rgb/rgba string.
    pub fn to_rgb(&self) -> String {
        if (self.a - 1.0).abs() < f32::EPSILON {
            format!("rgb({}, {}, {})", self.r, self.g, self.b)
        } else {
            format!("rgba({}, {}, {}, {:.2})", self.r, self.g, self.b, self.a)
        }
    }

    /// Convert to hsl/hsla string.
    pub fn to_hsl(&self) -> String {
        let (h, s, l) = rgb_to_hsl(self.r, self.g, self.b);
        if (self.a - 1.0).abs() < f32::EPSILON {
            format!(
                "hsl({}, {}%, {}%)",
                h.round() as i32,
                (s * 100.0).round() as i32,
                (l * 100.0).round() as i32
            )
        } else {
            format!(
                "hsla({}, {}%, {}%, {:.2})",
                h.round() as i32,
                (s * 100.0).round() as i32,
                (l * 100.0).round() as i32,
                self.a
            )
        }
    }

    /// Convert to oklch string.
    pub fn to_oklch(&self) -> String {
        let (l, c, h) = rgb_to_oklch(self.r, self.g, self.b);
        if (self.a - 1.0).abs() < f32::EPSILON {
            format!(
                "oklch({:.1}% {:.3} {:.0})",
                l * 100.0,
                c,
                h
            )
        } else {
            format!(
                "oklch({:.1}% {:.3} {:.0} / {:.2})",
                l * 100.0,
                c,
                h,
                self.a
            )
        }
    }

    /// Convert to iced::Color for rendering.
    pub fn to_iced_color(&self) -> iced::Color {
        iced::Color::from_rgba8(self.r, self.g, self.b, self.a)
    }

    /// Generate a default label from the hex value.
    pub fn default_label(&self) -> String {
        self.to_hex()
    }
}

#[derive(Debug, Clone)]
pub enum ColorParseError {
    InvalidFormat(String),
}

impl std::fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorParseError::InvalidFormat(s) => write!(f, "Invalid color format: '{}'", s),
        }
    }
}

// Regex patterns for parsing
static HEX_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^#?([0-9a-f]{3}|[0-9a-f]{4}|[0-9a-f]{6}|[0-9a-f]{8})$")
        .expect("Invalid hex regex")
});

static RGB_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^rgba?\s*\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})\s*(?:,\s*([\d.]+))?\s*\)$",
    )
    .expect("Invalid rgb regex")
});

static HSL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^hsla?\s*\(\s*(\d{1,3}(?:\.\d+)?)\s*,\s*(\d{1,3}(?:\.\d+)?)%?\s*,\s*(\d{1,3}(?:\.\d+)?)%?\s*(?:,\s*([\d.]+))?\s*\)$")
        .expect("Invalid hsl regex")
});

// OKLCH format: oklch(L% C H) or oklch(L% C H / A)
// L: 0-100% (lightness), C: 0-0.4+ (chroma), H: 0-360 (hue)
static OKLCH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^oklch\s*\(\s*([\d.]+)%?\s+([\d.]+)\s+([\d.]+)\s*(?:/\s*([\d.]+))?\s*\)$")
        .expect("Invalid oklch regex")
});

fn parse_hex(input: &str) -> Option<Color> {
    let caps = HEX_REGEX.captures(input)?;
    let hex = caps.get(1)?.as_str();

    let (r, g, b, a) = match hex.len() {
        3 => {
            // #RGB -> #RRGGBB
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            (r, g, b, 1.0)
        }
        4 => {
            // #RGBA -> #RRGGBBAA
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            let a = u8::from_str_radix(&hex[3..4].repeat(2), 16).ok()?;
            (r, g, b, a as f32 / 255.0)
        }
        6 => {
            // #RRGGBB
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            (r, g, b, 1.0)
        }
        8 => {
            // #RRGGBBAA
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            (r, g, b, a as f32 / 255.0)
        }
        _ => return None,
    };

    Some(Color::new(r, g, b, a, String::new()))
}

fn parse_rgb(input: &str) -> Option<Color> {
    let caps = RGB_REGEX.captures(input)?;

    let r: u8 = caps.get(1)?.as_str().parse().ok()?;
    let g: u8 = caps.get(2)?.as_str().parse().ok()?;
    let b: u8 = caps.get(3)?.as_str().parse().ok()?;
    let a: f32 = caps
        .get(4)
        .map(|m| m.as_str().parse().unwrap_or(1.0))
        .unwrap_or(1.0);

    Some(Color::new(r, g, b, a, String::new()))
}

fn parse_hsl(input: &str) -> Option<Color> {
    let caps = HSL_REGEX.captures(input)?;

    let h: f32 = caps.get(1)?.as_str().parse().ok()?;
    let s: f32 = caps.get(2)?.as_str().parse().ok()?;
    let l: f32 = caps.get(3)?.as_str().parse().ok()?;
    let a: f32 = caps
        .get(4)
        .map(|m| m.as_str().parse().unwrap_or(1.0))
        .unwrap_or(1.0);

    // Normalize s and l from percentage
    let s = s / 100.0;
    let l = l / 100.0;

    // Validate ranges
    if h > 360.0 || s > 1.0 || l > 1.0 {
        return None;
    }

    let (r, g, b) = hsl_to_rgb(h, s, l);
    Some(Color::new(r, g, b, a, String::new()))
}

fn parse_oklch(input: &str) -> Option<Color> {
    let caps = OKLCH_REGEX.captures(input)?;

    let l: f32 = caps.get(1)?.as_str().parse().ok()?;
    let c: f32 = caps.get(2)?.as_str().parse().ok()?;
    let h: f32 = caps.get(3)?.as_str().parse().ok()?;
    let a: f32 = caps
        .get(4)
        .map(|m| m.as_str().parse().unwrap_or(1.0))
        .unwrap_or(1.0);

    // Normalize L from percentage (0-100) to 0-1
    let l = l / 100.0;

    // Validate ranges
    if l > 1.0 || h > 360.0 {
        return None;
    }

    let (r, g, b) = oklch_to_rgb(l, c, h);
    Some(Color::new(r, g, b, a, String::new()))
}

/// Convert HSL to RGB.
pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    if s == 0.0 {
        let v = (l * 255.0).round() as u8;
        return (v, v, v);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let h = h / 360.0;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

/// Convert RGB to HSL.
pub fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < f32::EPSILON {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - r).abs() < f32::EPSILON {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if (max - g).abs() < f32::EPSILON {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };

    (h * 60.0, s, l)
}

// ============================================================================
// OKLCH Color Space Support
// ============================================================================

/// Convert sRGB component (0-1) to linear RGB.
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert linear RGB component to sRGB (0-1).
fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Convert RGB to OKLCH.
/// Returns (L: 0-1, C: 0-0.4+, H: 0-360).
#[allow(clippy::excessive_precision)]
pub fn rgb_to_oklch(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    // Convert to linear RGB
    let r_lin = srgb_to_linear(r as f32 / 255.0);
    let g_lin = srgb_to_linear(g as f32 / 255.0);
    let b_lin = srgb_to_linear(b as f32 / 255.0);

    // Linear RGB to LMS (using OKLab matrix)
    let l = 0.4122214708 * r_lin + 0.5363325363 * g_lin + 0.0514459929 * b_lin;
    let m = 0.2119034982 * r_lin + 0.6806995451 * g_lin + 0.1073969566 * b_lin;
    let s = 0.0883024619 * r_lin + 0.2817188376 * g_lin + 0.6299787005 * b_lin;

    // Apply cube root
    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    // LMS to OKLab
    let ok_l = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
    let ok_a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
    let ok_b = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;

    // OKLab to OKLCH
    let c = (ok_a * ok_a + ok_b * ok_b).sqrt();
    let h = if c < 1e-8 {
        0.0
    } else {
        let h_rad = ok_b.atan2(ok_a);
        let h_deg = h_rad.to_degrees();
        if h_deg < 0.0 {
            h_deg + 360.0
        } else {
            h_deg
        }
    };

    (ok_l, c, h)
}

/// Convert OKLCH to RGB.
/// L: 0-1 (lightness), C: 0-0.4+ (chroma), H: 0-360 (hue).
#[allow(clippy::excessive_precision)]
pub fn oklch_to_rgb(l: f32, c: f32, h: f32) -> (u8, u8, u8) {
    // OKLCH to OKLab
    let h_rad = h.to_radians();
    let ok_a = c * h_rad.cos();
    let ok_b = c * h_rad.sin();

    // OKLab to LMS (inverse of the forward transform)
    let l_ = l + 0.3963377774 * ok_a + 0.2158037573 * ok_b;
    let m_ = l - 0.1055613458 * ok_a - 0.0638541728 * ok_b;
    let s_ = l - 0.0894841775 * ok_a - 1.2914855480 * ok_b;

    // Cube the values
    let lms_l = l_ * l_ * l_;
    let lms_m = m_ * m_ * m_;
    let lms_s = s_ * s_ * s_;

    // LMS to linear RGB
    let r_lin = 4.0767416621 * lms_l - 3.3077115913 * lms_m + 0.2309699292 * lms_s;
    let g_lin = -1.2684380046 * lms_l + 2.6097574011 * lms_m - 0.3413193965 * lms_s;
    let b_lin = -0.0041960863 * lms_l - 0.7034186147 * lms_m + 1.7076147010 * lms_s;

    // Linear RGB to sRGB
    let r = linear_to_srgb(r_lin).clamp(0.0, 1.0);
    let g = linear_to_srgb(g_lin).clamp(0.0, 1.0);
    let b = linear_to_srgb(b_lin).clamp(0.0, 1.0);

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

/// Extract all color values from a text string.
pub fn extract_colors_from_text(text: &str) -> Vec<Color> {
    let mut colors = Vec::new();

    // Try to find hex colors
    let hex_finder = Regex::new(r"#[0-9a-fA-F]{3,8}\b").expect("Invalid hex finder regex");
    for cap in hex_finder.find_iter(text) {
        if let Ok(color) = Color::parse(cap.as_str()) {
            colors.push(color);
        }
    }

    // Try to find rgb/rgba colors
    let rgb_finder = Regex::new(r"(?i)rgba?\s*\([^)]+\)").expect("Invalid rgb finder regex");
    for cap in rgb_finder.find_iter(text) {
        if let Ok(color) = Color::parse(cap.as_str()) {
            colors.push(color);
        }
    }

    // Try to find hsl/hsla colors
    let hsl_finder = Regex::new(r"(?i)hsla?\s*\([^)]+\)").expect("Invalid hsl finder regex");
    for cap in hsl_finder.find_iter(text) {
        if let Ok(color) = Color::parse(cap.as_str()) {
            colors.push(color);
        }
    }

    // Try to find oklch colors
    let oklch_finder = Regex::new(r"(?i)oklch\s*\([^)]+\)").expect("Invalid oklch finder regex");
    for cap in oklch_finder.find_iter(text) {
        if let Ok(color) = Color::parse(cap.as_str()) {
            colors.push(color);
        }
    }

    colors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex() {
        let color = Color::parse("#FF5733").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 87);
        assert_eq!(color.b, 51);
        assert!((color.a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_hex_short() {
        let color = Color::parse("#F53").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 85);
        assert_eq!(color.b, 51);
    }

    #[test]
    fn test_parse_hex_with_alpha() {
        let color = Color::parse("#FF573380").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 87);
        assert_eq!(color.b, 51);
        assert!((color.a - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_parse_rgb() {
        let color = Color::parse("rgb(255, 87, 51)").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 87);
        assert_eq!(color.b, 51);
    }

    #[test]
    fn test_parse_rgba() {
        let color = Color::parse("rgba(255, 87, 51, 0.5)").unwrap();
        assert_eq!(color.r, 255);
        assert!((color.a - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_hsl() {
        let color = Color::parse("hsl(11, 100%, 60%)").unwrap();
        // Should be approximately #FF5733
        assert!(color.r > 250);
    }

    #[test]
    fn test_to_hex() {
        let color = Color::new(255, 87, 51, 1.0, String::new());
        assert_eq!(color.to_hex(), "#FF5733");
    }

    #[test]
    fn test_to_rgb() {
        let color = Color::new(255, 87, 51, 1.0, String::new());
        assert_eq!(color.to_rgb(), "rgb(255, 87, 51)");
    }

    #[test]
    fn test_extract_colors() {
        let text = "Colors: #FF5733 and rgb(0, 128, 255) and hsl(120, 50%, 50%)";
        let colors = extract_colors_from_text(text);
        assert_eq!(colors.len(), 3);
    }

    #[test]
    fn test_parse_oklch() {
        // oklch(70% 0.15 30) should produce an orange-ish color
        let color = Color::parse("oklch(70% 0.15 30)").unwrap();
        // Just verify it parses and produces reasonable RGB values
        assert!(color.r > 0);
        assert!((color.a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_oklch_with_alpha() {
        let color = Color::parse("oklch(70% 0.15 30 / 0.5)").unwrap();
        assert!((color.a - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rgb_to_oklch_roundtrip() {
        // Test that RGB -> OKLCH -> RGB roundtrip is reasonably accurate
        let original = (128_u8, 64_u8, 192_u8);
        let (l, c, h) = rgb_to_oklch(original.0, original.1, original.2);
        let (r, g, b) = oklch_to_rgb(l, c, h);
        // Allow some tolerance due to floating point and clamping
        assert!((r as i32 - original.0 as i32).abs() <= 2);
        assert!((g as i32 - original.1 as i32).abs() <= 2);
        assert!((b as i32 - original.2 as i32).abs() <= 2);
    }

    #[test]
    fn test_to_oklch() {
        let color = Color::new(255, 128, 64, 1.0, String::new());
        let oklch_str = color.to_oklch();
        assert!(oklch_str.starts_with("oklch("));
        assert!(!oklch_str.contains("/")); // No alpha when a = 1.0
    }

    #[test]
    fn test_to_oklch_with_alpha() {
        let color = Color::new(255, 128, 64, 0.5, String::new());
        let oklch_str = color.to_oklch();
        assert!(oklch_str.contains("/")); // Has alpha
    }

    #[test]
    fn test_extract_colors_with_oklch() {
        let text = "Colors: #FF5733 and oklch(70% 0.15 30)";
        let colors = extract_colors_from_text(text);
        assert_eq!(colors.len(), 2);
    }
}

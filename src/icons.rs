//! Phosphor Icons Light font integration.

#![allow(dead_code)]

use iced::widget::text;
use iced::Font;

/// Phosphor Light icon font.
pub const FONT: Font = Font::with_name("Phosphor-Light");

/// Font bytes for loading.
pub const FONT_BYTES: &[u8] = include_bytes!("../fonts/Phosphor-Light.ttf");

/// Create an icon text widget from a codepoint.
fn icon(codepoint: char) -> text::Text<'static> {
    text(codepoint.to_string()).font(FONT)
}

// === Icon Functions ===
// Each returns a Text widget with the icon glyph.

/// Pencil icon (for edit)
pub fn pencil() -> text::Text<'static> {
    icon('\u{e3ae}')
}

/// X icon (for close/delete)
pub fn x() -> text::Text<'static> {
    icon('\u{e4f6}')
}

/// Copy icon
pub fn copy() -> text::Text<'static> {
    icon('\u{e1ca}')
}

/// Trash icon (for delete)
pub fn trash() -> text::Text<'static> {
    icon('\u{e4a6}')
}

/// Plus icon (for add)
pub fn plus() -> text::Text<'static> {
    icon('\u{e3d4}')
}

/// Check icon (for confirm/save)
pub fn check() -> text::Text<'static> {
    icon('\u{e182}')
}

/// Magnifying glass icon (for search/filter)
pub fn magnifying_glass() -> text::Text<'static> {
    icon('\u{e30c}')
}

/// Floppy disk icon (for save)
pub fn floppy_disk() -> text::Text<'static> {
    icon('\u{e248}')
}

/// Palette icon (for color picker)
pub fn palette() -> text::Text<'static> {
    icon('\u{e6c8}')
}

/// Clipboard icon
pub fn clipboard() -> text::Text<'static> {
    icon('\u{e196}')
}

/// Eye icon (for preview)
pub fn eye() -> text::Text<'static> {
    icon('\u{e220}')
}

/// Funnel icon (for filter)
pub fn funnel() -> text::Text<'static> {
    icon('\u{e266}')
}

/// Arrow clockwise icon (for refresh/reset)
pub fn arrow_clockwise() -> text::Text<'static> {
    icon('\u{e036}')
}

/// X circle icon (for clear)
pub fn x_circle() -> text::Text<'static> {
    icon('\u{e4f8}')
}

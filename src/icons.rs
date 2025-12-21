//! Font integration for icons and text.

#![allow(dead_code)]

use iced::widget::text;
use iced::Font;

/// Phosphor Light icon font.
pub const ICON_FONT: Font = Font::with_name("Phosphor-Light");

/// Icon font bytes for loading.
pub const ICON_FONT_BYTES: &[u8] = include_bytes!("../fonts/Phosphor-Light.ttf");

/// Lilex Thin - default text font.
pub const TEXT_FONT: Font = Font::with_name("Lilex");

/// Text font bytes for loading.
pub const TEXT_FONT_BYTES: &[u8] = include_bytes!("../fonts/Lilex-Thin.ttf");

/// Create an icon text widget from a codepoint.
fn icon(codepoint: char) -> text::Text<'static> {
    text(codepoint.to_string()).font(ICON_FONT)
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

/// Code icon (for code snippets)
pub fn code() -> text::Text<'static> {
    icon('\u{e1bc}')
}

/// Text icon (for text snippets) - using TextT
pub fn text_icon() -> text::Text<'static> {
    icon('\u{e48a}')
}

/// Arrow square out icon (for open in external editor)
pub fn arrow_square_out() -> text::Text<'static> {
    icon('\u{e5de}')
}

/// Gear icon (for settings)
pub fn gear() -> text::Text<'static> {
    icon('\u{e27a}')
}

/// Tag icon (for palettes/categories)
pub fn tag() -> text::Text<'static> {
    icon('\u{e478}')
}

/// Folder icon (for grouping)
pub fn folder() -> text::Text<'static> {
    icon('\u{e24a}')
}

// === Icon constants for use in text widgets ===

/// Code icon codepoint
pub const CODE_ICON: char = '\u{e1bc}';

/// Text icon codepoint
pub const TEXT_ICON: char = '\u{e48a}';

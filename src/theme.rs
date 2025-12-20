//! Design tokens and styling utilities for Shard.

#![allow(dead_code)]

use iced::widget::{button, container, text_input};
use iced::{Border, Color, Theme};

// === Color Palette ===
pub const BG_BASE: Color = Color::from_rgb(0.059, 0.059, 0.059); // #0F0F0F
pub const BG_SURFACE: Color = Color::from_rgb(0.102, 0.102, 0.102); // #1A1A1A
pub const BG_ELEVATED: Color = Color::from_rgb(0.145, 0.145, 0.145); // #252525
pub const BORDER_SUBTLE: Color = Color::from_rgb(0.165, 0.165, 0.165); // #2A2A2A
pub const BORDER_ACCENT: Color = Color::from_rgb(0.231, 0.510, 0.965); // #3B82F6
pub const TEXT_PRIMARY: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.87);
pub const TEXT_SECONDARY: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.60);
pub const TEXT_MUTED: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.38);
pub const ACCENT: Color = Color::from_rgb(0.231, 0.510, 0.965); // #3B82F6
pub const ACCENT_HOVER: Color = Color::from_rgb(0.376, 0.647, 0.980); // #60A5FA
pub const DANGER: Color = Color::from_rgb(0.937, 0.267, 0.267); // #EF4444
pub const SUCCESS: Color = Color::from_rgb(0.133, 0.773, 0.369); // #22C55E

// === Spacing ===
pub const SPACE_XS: f32 = 4.0;
pub const SPACE_SM: f32 = 8.0;
pub const SPACE_MD: f32 = 16.0;
pub const SPACE_LG: f32 = 24.0;
pub const SPACE_XL: f32 = 32.0;

// === Border Radius ===
pub const RADIUS_SM: f32 = 6.0;
pub const RADIUS_MD: f32 = 8.0;
pub const RADIUS_LG: f32 = 12.0;
pub const RADIUS_XL: f32 = 16.0;

// === Style Functions ===

/// Card container style
pub fn card_style(_theme: &Theme, selected: bool) -> container::Style {
    container::Style::default()
        .background(BG_SURFACE)
        .border(Border {
            color: if selected {
                BORDER_ACCENT
            } else {
                BORDER_SUBTLE
            },
            width: if selected { 2.0 } else { 1.0 },
            radius: RADIUS_LG.into(),
        })
}

/// Header/toolbar container style
pub fn header_style(_theme: &Theme) -> container::Style {
    container::Style::default()
        .background(BG_SURFACE)
        .border(Border {
            color: BORDER_SUBTLE,
            width: 0.0,
            radius: 0.0.into(),
        })
}

/// Status bar style
pub fn status_bar_style(_theme: &Theme) -> container::Style {
    container::Style::default()
        .background(BG_BASE)
        .border(Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: 0.0.into(),
        })
}

/// Primary button style
pub fn primary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => ACCENT_HOVER,
        _ => ACCENT,
    };
    button::Style {
        background: Some(bg.into()),
        text_color: Color::WHITE,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: RADIUS_MD.into(),
        },
        ..button::Style::default()
    }
}

/// Secondary/ghost button style
pub fn secondary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => BG_ELEVATED,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(bg.into()),
        text_color: TEXT_PRIMARY,
        border: Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        ..button::Style::default()
    }
}

/// Subtle/text button style (for copy buttons)
pub fn subtle_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => BG_ELEVATED,
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(bg.into()),
        text_color: TEXT_SECONDARY,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: RADIUS_SM.into(),
        },
        ..button::Style::default()
    }
}

/// Danger button style (for delete)
pub fn danger_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let text = match status {
        button::Status::Hovered | button::Status::Pressed => Color::WHITE,
        _ => DANGER,
    };
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => Some(DANGER.into()),
        _ => None,
    };
    button::Style {
        background: bg,
        text_color: text,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: RADIUS_SM.into(),
        },
        ..button::Style::default()
    }
}

/// Styled text input
pub fn input_style(
    theme: &Theme,
    status: text_input::Status,
    has_error: bool,
) -> text_input::Style {
    let default = text_input::default(theme, status);
    text_input::Style {
        background: BG_ELEVATED.into(),
        border: Border {
            color: if has_error { DANGER } else { BORDER_SUBTLE },
            width: if has_error { 2.0 } else { 1.0 },
            radius: RADIUS_MD.into(),
        },
        placeholder: TEXT_MUTED,
        value: TEXT_PRIMARY,
        selection: ACCENT,
        ..default
    }
}

/// Modal overlay background style
pub fn modal_overlay_style(_theme: &Theme) -> container::Style {
    container::Style::default().background(Color::from_rgba(0.0, 0.0, 0.0, 0.7))
}

/// Modal dialog style
pub fn modal_dialog_style(_theme: &Theme) -> container::Style {
    container::Style::default()
        .background(BG_SURFACE)
        .border(Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: RADIUS_XL.into(),
        })
}

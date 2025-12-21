//! Color picker modal state and view.

use iced::widget::{
    button, column, container, mouse_area, opaque, row, slider, text, text_input, Canvas,
};
use iced::{Element, Length};

use crate::icons;
use crate::message::Message;
use crate::snippet::{
    hsl_to_rgb, oklch_to_rgb, rgb_to_hsl, rgb_to_oklch, ColorData, Snippet, SnippetContent,
};
use crate::theme::{
    input_style, modal_dialog_style, modal_overlay_style, primary_button_style,
    secondary_button_style, subtle_button_style, SPACE_MD, SPACE_SM, SPACE_XS, TEXT_MUTED,
    TEXT_PRIMARY, TEXT_SECONDARY,
};
use crate::widgets::{AlphaBar, ChromaLightnessBox, ColorSwatch, HueBar, SaturationLightnessBox};

/// Color picker mode: HSL or OKLCH color space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PickerMode {
    #[default]
    Hsl,
    Oklch,
}

/// State for the color picker modal.
#[derive(Debug, Clone)]
pub struct ColorPickerState {
    /// The snippet being edited (Some = editing existing, None = creating new)
    pub editing_id: Option<i64>,
    /// Current picker mode (HSL or OKLCH)
    pub mode: PickerMode,
    /// Current hue (0-360) - shared between HSL and OKLCH
    pub hue: f32,
    /// Current saturation (0-1) - HSL mode
    pub saturation: f32,
    /// Current lightness (0-1) - HSL mode
    pub lightness: f32,
    /// Current OKLCH lightness (0-1)
    pub oklch_l: f32,
    /// Current OKLCH chroma (0-0.4+)
    pub oklch_c: f32,
    /// Current OKLCH hue (0-360)
    pub oklch_h: f32,
    /// Current alpha (0-1)
    pub alpha: f32,
    /// Label for the color
    pub label: String,
    /// Original color when editing (r, g, b, a) - used to detect changes
    original_color: Option<(u8, u8, u8, f32)>,
}

impl ColorPickerState {
    /// Create a new picker state for creating a new color.
    pub fn new_color(default_mode: PickerMode) -> Self {
        Self {
            editing_id: None,
            mode: default_mode,
            hue: 0.0,
            saturation: 0.5,
            lightness: 0.5,
            oklch_l: 0.5,
            oklch_c: 0.15,
            oklch_h: 0.0,
            alpha: 1.0,
            label: String::new(),
            original_color: None,
        }
    }

    /// Create a picker state from an existing snippet.
    pub fn from_snippet(snippet: &Snippet, default_mode: PickerMode) -> Self {
        if let SnippetContent::Color(color) = &snippet.content {
            let (h, s, l) = rgb_to_hsl(color.r, color.g, color.b);
            let (ok_l, ok_c, ok_h) = rgb_to_oklch(color.r, color.g, color.b);
            Self {
                editing_id: Some(snippet.id),
                mode: default_mode,
                hue: h,
                saturation: s,
                lightness: l,
                oklch_l: ok_l,
                oklch_c: ok_c,
                oklch_h: ok_h,
                alpha: color.a,
                label: snippet.label.clone(),
                original_color: Some((color.r, color.g, color.b, color.a)),
            }
        } else {
            Self::new_color(default_mode)
        }
    }

    /// Get the current color as RGB values based on the current mode.
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        match self.mode {
            PickerMode::Hsl => hsl_to_rgb(self.hue, self.saturation, self.lightness),
            PickerMode::Oklch => oklch_to_rgb(self.oklch_l, self.oklch_c, self.oklch_h),
        }
    }

    /// Get the current color as an iced::Color for preview.
    pub fn to_iced_color(&self) -> iced::Color {
        let (r, g, b) = self.to_rgb();
        iced::Color::from_rgba8(r, g, b, self.alpha)
    }

    /// Get the current color as a ColorData struct.
    pub fn to_color_data(&self) -> ColorData {
        let (r, g, b) = self.to_rgb();
        ColorData::new(r, g, b, self.alpha)
    }

    /// Sync HSL values from the current RGB (used when switching modes).
    pub fn sync_hsl_from_rgb(&mut self) {
        let (r, g, b) = self.to_rgb();
        let (h, s, l) = rgb_to_hsl(r, g, b);
        self.hue = h;
        self.saturation = s;
        self.lightness = l;
    }

    /// Sync OKLCH values from the current RGB (used when switching modes).
    pub fn sync_oklch_from_rgb(&mut self) {
        let (r, g, b) = self.to_rgb();
        let (ok_l, ok_c, ok_h) = rgb_to_oklch(r, g, b);
        self.oklch_l = ok_l;
        self.oklch_c = ok_c;
        self.oklch_h = ok_h;
    }

    /// Check if the color has changed from the original (when editing).
    /// Returns true if editing and color differs from original.
    pub fn has_color_changed(&self) -> bool {
        if let Some((orig_r, orig_g, orig_b, orig_a)) = self.original_color {
            let (r, g, b) = self.to_rgb();
            r != orig_r || g != orig_g || b != orig_b || (self.alpha - orig_a).abs() > 0.001
        } else {
            false
        }
    }
}

/// Render the color picker modal.
pub fn view_color_picker_modal(picker: &ColorPickerState) -> Element<'_, Message> {
    let title = if picker.editing_id.is_some() {
        "Edit Color"
    } else {
        "New Color"
    };

    // Header row with title, mode toggle, and close button
    let mode_toggle = row![
        button(text("HSL").size(12))
            .on_press(Message::PickerModeChanged(PickerMode::Hsl))
            .padding([SPACE_XS, SPACE_SM])
            .style(if picker.mode == PickerMode::Hsl {
                primary_button_style
            } else {
                secondary_button_style
            }),
        button(text("OKLCH").size(12))
            .on_press(Message::PickerModeChanged(PickerMode::Oklch))
            .padding([SPACE_XS, SPACE_SM])
            .style(if picker.mode == PickerMode::Oklch {
                primary_button_style
            } else {
                secondary_button_style
            }),
    ]
    .spacing(2);

    let header_row = row![
        text(title).size(20).color(TEXT_PRIMARY),
        iced::widget::Space::new().width(Length::Fill),
        mode_toggle,
        iced::widget::Space::new().width(Length::Fixed(SPACE_SM)),
        button(icons::x().size(16))
            .on_press(Message::CloseColorPicker)
            .padding([SPACE_XS, SPACE_SM])
            .style(subtle_button_style),
    ]
    .align_y(iced::Alignment::Center);

    // Preview swatch (large)
    let preview_swatch = Canvas::new(ColorSwatch {
        color: picker.to_iced_color(),
    })
    .width(100)
    .height(100);

    // Current color values display
    let (r, g, b) = picker.to_rgb();
    let (ok_l, ok_c, ok_h) = rgb_to_oklch(r, g, b);
    let color_values = column![
        text(format!(
            "H: {:.0}°  S: {:.0}%  L: {:.0}%",
            picker.hue,
            picker.saturation * 100.0,
            picker.lightness * 100.0
        ))
        .size(12)
        .color(TEXT_SECONDARY),
        text(format!(
            "R: {}  G: {}  B: {}  A: {:.0}%",
            r,
            g,
            b,
            picker.alpha * 100.0
        ))
        .size(12)
        .color(TEXT_SECONDARY),
        text(format!(
            "L: {:.1}%  C: {:.3}  H: {:.0}°",
            ok_l * 100.0,
            ok_c,
            ok_h
        ))
        .size(12)
        .color(TEXT_SECONDARY),
        text(picker.to_color_data().to_hex())
            .size(12)
            .color(TEXT_MUTED),
    ]
    .spacing(SPACE_XS);

    let preview_row = row![preview_swatch, color_values]
        .spacing(SPACE_MD)
        .align_y(iced::Alignment::Center);

    // Build mode-specific controls
    let controls: Element<'_, Message> = match picker.mode {
        PickerMode::Hsl => {
            let sl_box = Canvas::new(SaturationLightnessBox {
                hue: picker.hue,
                saturation: picker.saturation,
                lightness: picker.lightness,
            })
            .width(280)
            .height(200);

            let saturation_slider = row![
                text("S")
                    .size(12)
                    .color(TEXT_SECONDARY)
                    .width(Length::Fixed(20.0)),
                slider(
                    0.0..=1.0,
                    picker.saturation,
                    Message::PickerSaturationChanged
                )
                .step(0.01)
                .width(Length::Fill),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center);

            let lightness_slider = row![
                text("L")
                    .size(12)
                    .color(TEXT_SECONDARY)
                    .width(Length::Fixed(20.0)),
                slider(0.0..=1.0, picker.lightness, Message::PickerLightnessChanged)
                    .step(0.01)
                    .width(Length::Fill),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center);

            let hue_bar = Canvas::new(HueBar {
                current_hue: picker.hue,
            })
            .width(280)
            .height(25);

            column![sl_box, saturation_slider, lightness_slider, hue_bar,]
                .spacing(SPACE_MD)
                .into()
        }
        PickerMode::Oklch => {
            let cl_box = Canvas::new(ChromaLightnessBox {
                hue: picker.oklch_h,
                chroma: picker.oklch_c,
                lightness: picker.oklch_l,
                max_chroma: 0.4,
            })
            .width(280)
            .height(200);

            let lightness_slider = row![
                text("L")
                    .size(12)
                    .color(TEXT_SECONDARY)
                    .width(Length::Fixed(20.0)),
                slider(0.0..=1.0, picker.oklch_l, Message::PickerOklchLChanged)
                    .step(0.01)
                    .width(Length::Fill),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center);

            let chroma_slider = row![
                text("C")
                    .size(12)
                    .color(TEXT_SECONDARY)
                    .width(Length::Fixed(20.0)),
                slider(0.0..=0.4, picker.oklch_c, Message::PickerOklchCChanged)
                    .step(0.005)
                    .width(Length::Fill),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center);

            let hue_slider = row![
                text("H")
                    .size(12)
                    .color(TEXT_SECONDARY)
                    .width(Length::Fixed(20.0)),
                slider(0.0..=360.0, picker.oklch_h, Message::PickerOklchHChanged)
                    .step(1.0)
                    .width(Length::Fill),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center);

            column![cl_box, lightness_slider, chroma_slider, hue_slider,]
                .spacing(SPACE_MD)
                .into()
        }
    };

    // Alpha bar
    let alpha_bar = Canvas::new(AlphaBar {
        color: {
            let (r, g, b) = picker.to_rgb();
            iced::Color::from_rgb8(r, g, b)
        },
        alpha: picker.alpha,
    })
    .width(280)
    .height(25);

    // Label input
    let label_input = row![
        text("Label:").size(12).color(TEXT_SECONDARY),
        text_input("Color label...", &picker.label)
            .on_input(Message::PickerLabelChanged)
            .padding(SPACE_SM)
            .width(Length::Fill)
            .style(|theme, status| input_style(theme, status, false)),
    ]
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center);

    // Action buttons
    let cancel_btn = button(text("Cancel").size(14))
        .on_press(Message::CloseColorPicker)
        .padding(SPACE_SM)
        .style(secondary_button_style);

    let confirm_btn = button(
        text(if picker.editing_id.is_some() {
            "Save"
        } else {
            "Add"
        })
        .size(14),
    )
    .on_press(Message::ConfirmColorPicker)
    .padding(SPACE_SM)
    .style(primary_button_style);

    // Show "Save as New" button when editing and color has changed
    let action_buttons: Element<'_, Message> =
        if picker.editing_id.is_some() && picker.has_color_changed() {
            let save_as_new_btn = button(text("Save as New").size(14))
                .on_press(Message::SaveColorAsNew)
                .padding(SPACE_SM)
                .style(secondary_button_style);

            row![cancel_btn, save_as_new_btn, confirm_btn]
                .spacing(SPACE_SM)
                .into()
        } else {
            row![cancel_btn, confirm_btn].spacing(SPACE_SM).into()
        };

    // Modal content
    let modal_content = column![
        header_row,
        preview_row,
        controls,
        alpha_bar,
        label_input,
        action_buttons,
    ]
    .spacing(SPACE_MD)
    .padding(SPACE_MD)
    .width(Length::Fixed(320.0));

    let modal_dialog = container(modal_content).style(modal_dialog_style);

    // Semi-transparent overlay
    let overlay = mouse_area(
        container(opaque(modal_dialog))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(modal_overlay_style),
    )
    .on_press(Message::CloseColorPicker);

    overlay.into()
}

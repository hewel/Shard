//! Color picker modal state and view.

use iced::widget::{
    button, column, container, mouse_area, opaque, row, slider, text, text_input, Canvas,
};
use iced::{Element, Length};

use crate::color::{hsl_to_rgb, rgb_to_hsl, Color};
use crate::message::Message;
use crate::theme::{
    input_style, modal_dialog_style, modal_overlay_style, primary_button_style,
    secondary_button_style, subtle_button_style, SPACE_MD, SPACE_SM, SPACE_XS, TEXT_MUTED,
    TEXT_PRIMARY, TEXT_SECONDARY,
};
use crate::widgets::{AlphaBar, ColorSwatch, HueBar, SaturationLightnessBox};

/// State for the color picker modal.
#[derive(Debug, Clone)]
pub struct ColorPickerState {
    /// The color being edited (Some = editing existing, None = creating new)
    pub editing_id: Option<i64>,
    /// Current hue (0-360)
    pub hue: f32,
    /// Current saturation (0-1)
    pub saturation: f32,
    /// Current lightness (0-1)
    pub lightness: f32,
    /// Current alpha (0-1)
    pub alpha: f32,
    /// Label for the color
    pub label: String,
}

impl ColorPickerState {
    /// Create a new picker state for creating a new color.
    pub fn new_color() -> Self {
        Self {
            editing_id: None,
            hue: 0.0,
            saturation: 0.5,
            lightness: 0.5,
            alpha: 1.0,
            label: String::new(),
        }
    }

    /// Create a picker state for editing an existing color.
    pub fn edit_color(color: &Color) -> Self {
        let (h, s, l) = rgb_to_hsl(color.r, color.g, color.b);
        Self {
            editing_id: Some(color.id),
            hue: h,
            saturation: s,
            lightness: l,
            alpha: color.a,
            label: color.label.clone(),
        }
    }

    /// Get the current color as RGB values.
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        hsl_to_rgb(self.hue, self.saturation, self.lightness)
    }

    /// Get the current color as an iced::Color for preview.
    pub fn to_iced_color(&self) -> iced::Color {
        let (r, g, b) = self.to_rgb();
        iced::Color::from_rgba8(r, g, b, self.alpha)
    }

    /// Get the current color as a Color struct.
    pub fn to_color(&self) -> Color {
        let (r, g, b) = self.to_rgb();
        let mut color = Color::new(r, g, b, self.alpha, self.label.clone());
        if color.label.is_empty() {
            color.label = color.default_label();
        }
        color
    }
}

/// Render the color picker modal.
pub fn view_color_picker_modal(picker: &ColorPickerState) -> Element<'_, Message> {
    let title = if picker.editing_id.is_some() {
        "Edit Color"
    } else {
        "New Color"
    };

    // Header row with title and close button
    let header_row = row![
        text(title).size(20).color(TEXT_PRIMARY),
        iced::widget::Space::new().width(Length::Fill),
        button(text("×").size(18))
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
        text(picker.to_color().to_hex()).size(12).color(TEXT_MUTED),
    ]
    .spacing(SPACE_XS);

    let preview_row = row![preview_swatch, color_values]
        .spacing(SPACE_MD)
        .align_y(iced::Alignment::Center);

    // Saturation/Lightness box
    let sl_box = Canvas::new(SaturationLightnessBox {
        hue: picker.hue,
        saturation: picker.saturation,
        lightness: picker.lightness,
    })
    .width(280)
    .height(200);

    // Saturation slider
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

    // Lightness slider
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

    // Hue bar
    let hue_bar = Canvas::new(HueBar {
        current_hue: picker.hue,
    })
    .width(280)
    .height(25);

    // Hue slider
    let hue_slider = row![
        text("H")
            .size(12)
            .color(TEXT_SECONDARY)
            .width(Length::Fixed(20.0)),
        slider(0.0..=360.0, picker.hue, Message::PickerHueChanged)
            .step(1.0)
            .width(Length::Fill),
    ]
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center);

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

    // Alpha slider
    let alpha_slider = row![
        text("A")
            .size(12)
            .color(TEXT_SECONDARY)
            .width(Length::Fixed(20.0)),
        slider(0.0..=1.0, picker.alpha, Message::PickerAlphaChanged)
            .step(0.01)
            .width(Length::Fill),
    ]
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center);

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
    let action_buttons = row![
        button(text("Cancel").size(14))
            .on_press(Message::CloseColorPicker)
            .padding(SPACE_SM)
            .style(secondary_button_style),
        button(
            text(if picker.editing_id.is_some() {
                "Save"
            } else {
                "Add"
            })
            .size(14)
        )
        .on_press(Message::ConfirmColorPicker)
        .padding(SPACE_SM)
        .style(primary_button_style),
    ]
    .spacing(SPACE_SM);

    // Modal content
    let modal_content = column![
        header_row,
        preview_row,
        sl_box,
        saturation_slider,
        lightness_slider,
        hue_bar,
        hue_slider,
        alpha_bar,
        alpha_slider,
        label_input,
        action_buttons,
    ]
    .spacing(SPACE_MD)
    .padding(SPACE_MD)
    .width(Length::Fixed(320.0));

    // Modal dialog with modal_dialog_style
    let modal_dialog = container(modal_content).style(modal_dialog_style);

    // Semi-transparent overlay that closes the modal when clicked
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

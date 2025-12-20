//! Color card view component.

use iced::widget::{button, column, container, row, text, text_input, Canvas};
use iced::{Element, Length, Theme};

use crate::color::Color;
use crate::message::Message;
use crate::widgets::ColorSwatch;

/// Render a color card for the palette list.
pub fn view_color_card<'a>(
    color: &'a Color,
    is_selected: bool,
    editing_label: Option<&'a (i64, String)>,
) -> Element<'a, Message> {
    let swatch = Canvas::new(ColorSwatch {
        color: color.to_iced_color(),
    })
    .width(50)
    .height(50);

    let is_editing = editing_label.map(|(id, _)| *id) == Some(color.id);

    let label_element: Element<'_, Message> = if is_editing {
        let edit_text = editing_label.map(|(_, t)| t.as_str()).unwrap_or("");
        row![
            text_input("Label...", edit_text)
                .on_input(Message::EditLabelChanged)
                .on_submit(Message::SaveLabel)
                .width(Length::Fixed(150.0))
                .padding(5),
            button(text("Save").size(12))
                .on_press(Message::SaveLabel)
                .padding(5),
            button(text("Cancel").size(12))
                .on_press(Message::CancelEditLabel)
                .padding(5),
        ]
        .spacing(5)
        .into()
    } else {
        button(text(&color.label).size(14))
            .on_press(Message::StartEditLabel(color.id))
            .style(button::text)
            .into()
    };

    let hex_display = text(color.to_hex())
        .size(12)
        .color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.7));

    let copy_buttons = row![
        button(text("Hex").size(11))
            .on_press(Message::CopyHex(color.id))
            .padding(5),
        button(text("RGB").size(11))
            .on_press(Message::CopyRgb(color.id))
            .padding(5),
        button(text("HSL").size(11))
            .on_press(Message::CopyHsl(color.id))
            .padding(5),
    ]
    .spacing(5);

    let edit_button = button(text("Edit").size(11))
        .on_press(Message::OpenColorPicker(Some(color.id)))
        .padding(5);

    let delete_button = button(
        text("Del")
            .size(11)
            .color(iced::Color::from_rgb(0.9, 0.3, 0.3)),
    )
    .on_press(Message::DeleteColor(color.id))
    .padding(5)
    .style(button::text);

    let info_column = column![label_element, hex_display].spacing(4);

    let card = row![
        swatch,
        info_column,
        copy_buttons,
        edit_button,
        delete_button
    ]
    .spacing(15)
    .padding(10)
    .align_y(iced::Alignment::Center);

    let color_id = color.id;
    let card_container = container(card)
        .style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            let border_color = if is_selected {
                iced::Color::from_rgb(0.4, 0.7, 1.0) // Highlight selected
            } else {
                palette.background.strong.color
            };
            let border_width = if is_selected { 2.0 } else { 1.0 };
            container::Style::default()
                .background(palette.background.weak.color)
                .border(iced::Border {
                    color: border_color,
                    width: border_width,
                    radius: 8.0.into(),
                })
        })
        .width(Length::Fill);

    // Wrap in a button for click-to-select
    button(card_container)
        .on_press(Message::SelectColor(Some(color_id)))
        .style(|_theme, _status| button::Style::default())
        .padding(0)
        .into()
}

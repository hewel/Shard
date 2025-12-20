//! Color card view component.

use iced::widget::{button, column, container, row, text, text_input, Canvas};
use iced::{Element, Length};

use crate::icons;
use crate::message::Message;
use crate::snippet::ColorData;
use crate::theme::{
    card_style, danger_button_style, input_style, primary_button_style, secondary_button_style,
    subtle_button_style, SPACE_MD, SPACE_SM, SPACE_XS, TEXT_MUTED,
};
use crate::widgets::ColorSwatch;

/// Render a color card for the palette list.
pub fn view_color_card<'a>(
    id: i64,
    label: &'a str,
    color: &'a ColorData,
    is_selected: bool,
    editing_label: Option<&'a (i64, String)>,
) -> Element<'a, Message> {
    // Color swatch (72x72)
    let swatch = Canvas::new(ColorSwatch {
        color: color.to_iced_color(),
    })
    .width(72)
    .height(72);

    let is_editing = editing_label.map(|(eid, _)| *eid) == Some(id);

    // Hex display
    let hex_display = text(color.to_hex()).size(12).color(TEXT_MUTED);

    // Copy buttons
    let copy_buttons = row![
        button(text("Hex").size(11))
            .on_press(Message::CopyHex(id))
            .padding([SPACE_XS, SPACE_SM])
            .style(subtle_button_style),
        button(text("RGB").size(11))
            .on_press(Message::CopyRgb(id))
            .padding([SPACE_XS, SPACE_SM])
            .style(subtle_button_style),
        button(text("HSL").size(11))
            .on_press(Message::CopyHsl(id))
            .padding([SPACE_XS, SPACE_SM])
            .style(subtle_button_style),
        button(text("OKLCH").size(11))
            .on_press(Message::CopyOklch(id))
            .padding([SPACE_XS, SPACE_SM])
            .style(subtle_button_style),
    ]
    .spacing(SPACE_XS);

    // Edit button
    let edit_button = button(icons::pencil().size(14))
        .on_press(Message::OpenColorPicker(Some(id)))
        .padding([SPACE_XS, SPACE_SM])
        .style(subtle_button_style);

    // Delete button
    let delete_button = button(icons::trash().size(14))
        .on_press(Message::DeleteSnippet(id))
        .padding([SPACE_XS, SPACE_SM])
        .style(danger_button_style);

    let info_column = column![
        if is_editing {
            let edit_text = editing_label.map(|(_, t)| t.as_str()).unwrap_or("");
            row![
                text_input("Label...", edit_text)
                    .on_input(Message::EditLabelChanged)
                    .on_submit(Message::SaveLabel)
                    .width(Length::Fixed(150.0))
                    .padding(SPACE_SM)
                    .style(|theme, status| input_style(theme, status, false)),
                button(text("Save").size(12))
                    .on_press(Message::SaveLabel)
                    .padding(SPACE_SM)
                    .style(primary_button_style),
                button(text("Cancel").size(12))
                    .on_press(Message::CancelEditLabel)
                    .padding(SPACE_SM)
                    .style(secondary_button_style),
            ]
            .spacing(SPACE_SM)
        } else {
            row![button(text(label).size(15))
                .on_press(Message::StartEditLabel(id))
                .style(subtle_button_style)]
        },
        hex_display
    ]
    .spacing(SPACE_XS);

    let card = row![
        swatch,
        info_column,
        copy_buttons,
        edit_button,
        delete_button
    ]
    .spacing(SPACE_MD)
    .padding(SPACE_MD)
    .align_y(iced::Alignment::Center);

    let card_container = container(card)
        .style(move |theme| card_style(theme, is_selected))
        .width(Length::Fill);

    // Wrap in a button for click-to-select
    button(card_container)
        .on_press(Message::SelectSnippet(Some(id)))
        .style(|_theme, _status| button::Style::default())
        .padding(0)
        .into()
}

//! Color card view component.

use iced::widget::{button, column, container, row, text, Canvas};
use iced::{Element, Length};

use crate::icons;
use crate::message::Message;
use crate::snippet::ColorData;
use crate::theme::{
    card_style, danger_button_style, subtle_button_style, BG_SURFACE, SPACE_MD, SPACE_SM, SPACE_XS,
    TEXT_MUTED, TEXT_SECONDARY,
};
use crate::widgets::ColorSwatch;

/// Render a color card for the palette list.
pub fn view_color_card<'a>(
    id: i64,
    label: &'a str,
    color: &'a ColorData,
    is_selected: bool,
) -> Element<'a, Message> {
    // Color swatch (64x64)
    let swatch = container(
        Canvas::new(ColorSwatch {
            color: color.to_iced_color(),
        })
        .width(48)
        .height(48),
    )
    .width(64)
    .height(64)
    .center_x(64)
    .center_y(64)
    .style(|_theme| iced::widget::container::Style::default().background(BG_SURFACE));

    // Hex display
    let hex_display = text(color.to_hex()).size(11).color(TEXT_MUTED);

    // Copy buttons row
    let copy_buttons = row![
        button(row![icons::copy().size(11), text("Hex").size(11)].spacing(4))
            .on_press(Message::CopyHex(id))
            .padding([SPACE_XS, SPACE_SM])
            .style(subtle_button_style),
        button(row![icons::copy().size(11), text("RGB").size(11)].spacing(4))
            .on_press(Message::CopyRgb(id))
            .padding([SPACE_XS, SPACE_SM])
            .style(subtle_button_style),
        button(row![icons::copy().size(11), text("HSL").size(11)].spacing(4))
            .on_press(Message::CopyHsl(id))
            .padding([SPACE_XS, SPACE_SM])
            .style(subtle_button_style),
        button(row![icons::copy().size(11), text("OKLCH").size(11)].spacing(4))
            .on_press(Message::CopyOklch(id))
            .padding([SPACE_XS, SPACE_SM])
            .style(subtle_button_style),
    ]
    .spacing(SPACE_XS);

    // Info column with label, hex, and copy buttons
    let info_column = column![
        text(label).size(14).color(TEXT_SECONDARY),
        hex_display,
        copy_buttons,
    ]
    .spacing(SPACE_XS)
    .width(Length::Fill);

    // Action buttons (pin, edit, delete)
    let action_row = row![
        button(icons::push_pin().size(14))
            .on_press(Message::PinSnippet(id))
            .padding([SPACE_XS, SPACE_SM])
            .style(subtle_button_style),
        button(icons::pencil().size(14))
            .on_press(Message::OpenColorPicker(Some(id)))
            .padding([SPACE_XS, SPACE_SM])
            .style(subtle_button_style),
        button(icons::trash().size(14))
            .on_press(Message::DeleteSnippet(id))
            .padding([SPACE_XS, SPACE_SM])
            .style(danger_button_style),
    ]
    .spacing(SPACE_XS)
    .align_y(iced::Alignment::Center);

    let card = row![swatch, info_column, action_row]
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

//! Text snippet card view component.

use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};

use crate::icons;
use crate::message::Message;
use crate::snippet::TextData;
use crate::theme::{
    card_style, danger_button_style, subtle_button_style, BG_SURFACE, SPACE_MD, SPACE_SM,
    SPACE_XS, TEXT_MUTED, TEXT_SECONDARY,
};

/// Render a text snippet card.
pub fn view_text_card<'a>(
    id: i64,
    label: &'a str,
    text_data: &'a TextData,
    is_selected: bool,
) -> Element<'a, Message> {
    // Text icon (64x64 container)
    let text_icon = container(
        text(icons::TEXT_ICON)
            .size(28)
            .font(icons::ICON_FONT)
            .color(TEXT_SECONDARY),
    )
    .width(64)
    .height(64)
    .center_x(64)
    .center_y(64)
    .style(|_theme| iced::widget::container::Style::default().background(BG_SURFACE));

    // Stats badge
    let stats_badge = container(
        text(format!(
            "{} chars, {} lines",
            text_data.char_count(),
            text_data.line_count()
        ))
        .size(10)
        .color(TEXT_MUTED),
    )
    .padding([2, 6])
    .style(|_theme| iced::widget::container::Style::default().background(BG_SURFACE));

    // Header row: label + stats badge
    let header_row = row![text(label).size(14).color(TEXT_SECONDARY), stats_badge]
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center);

    // Text preview (first 2 lines)
    let preview = text_data.preview(2);
    let preview_text = text(preview).size(11).color(TEXT_MUTED);

    // Info column with header and preview
    let info_column = column![header_row, preview_text]
        .spacing(SPACE_XS)
        .width(Length::Fill);

    // Action buttons
    let action_row = row![
        button(icons::copy().size(14))
            .on_press(Message::CopySnippet(id))
            .padding([SPACE_XS, SPACE_SM])
            .style(subtle_button_style),
        button(icons::arrow_square_out().size(14))
            .on_press(Message::OpenInExternalEditor(id, false))
            .padding([SPACE_XS, SPACE_SM])
            .style(subtle_button_style),
        button(icons::pencil().size(14))
            .on_press(Message::OpenTextEditor(Some(id)))
            .padding([SPACE_XS, SPACE_SM])
            .style(subtle_button_style),
        button(icons::trash().size(14))
            .on_press(Message::DeleteSnippet(id))
            .padding([SPACE_XS, SPACE_SM])
            .style(danger_button_style),
    ]
    .spacing(SPACE_XS)
    .align_y(iced::Alignment::Center);

    let card = row![text_icon, info_column, action_row]
        .spacing(SPACE_MD)
        .padding(SPACE_MD)
        .align_y(iced::Alignment::Center);

    let card_container = container(card)
        .style(move |theme| card_style(theme, is_selected))
        .width(Length::Fill);

    button(card_container)
        .on_press(Message::SelectSnippet(Some(id)))
        .style(|_theme, _status| button::Style::default())
        .padding(0)
        .into()
}

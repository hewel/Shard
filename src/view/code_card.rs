//! Code snippet card view component.

use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length};

use crate::icons;
use crate::message::Message;
use crate::snippet::CodeData;
use crate::theme::{
    card_style, danger_button_style, input_style, primary_button_style, secondary_button_style,
    subtle_button_style, BG_SURFACE, SPACE_MD, SPACE_SM, SPACE_XS, TEXT_MUTED, TEXT_SECONDARY,
};

/// Render a code snippet card.
pub fn view_code_card<'a>(
    id: i64,
    label: &'a str,
    code: &'a CodeData,
    is_selected: bool,
    editing_label: Option<&'a (i64, String)>,
) -> Element<'a, Message> {
    // Code icon placeholder (colored box for code)
    let code_icon = container(
        text(icons::CODE_ICON)
            .size(32)
            .font(icons::ICON_FONT)
            .color(TEXT_SECONDARY),
    )
    .width(72)
    .height(72)
    .center_x(72)
    .center_y(72)
    .style(|_theme| iced::widget::container::Style::default().background(BG_SURFACE));

    let is_editing = editing_label.map(|(eid, _)| *eid) == Some(id);

    let label_element: Element<'_, Message> = if is_editing {
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
        .into()
    } else {
        button(text(label).size(15))
            .on_press(Message::StartEditLabel(id))
            .style(subtle_button_style)
            .into()
    };

    // Language badge and line count
    let language_badge = container(text(&code.language).size(11).color(TEXT_MUTED))
        .padding([2, 6])
        .style(|_theme| iced::widget::container::Style::default().background(BG_SURFACE));

    let line_count = text(format!("{} lines", code.line_count()))
        .size(12)
        .color(TEXT_MUTED);

    // Code preview (first 2 lines)
    let preview = code.preview(2);
    let preview_text = text(preview)
        .size(11)
        .color(TEXT_SECONDARY)
        .width(Length::Fixed(300.0));

    // Copy button
    let copy_button = button(text("Copy").size(11))
        .on_press(Message::CopySnippet(id))
        .padding([SPACE_XS, SPACE_SM])
        .style(subtle_button_style);

    // Edit button
    let edit_button = button(icons::pencil().size(14))
        .on_press(Message::OpenCodeEditor(Some(id)))
        .padding([SPACE_XS, SPACE_SM])
        .style(subtle_button_style);

    // Delete button
    let delete_button = button(icons::trash().size(14))
        .on_press(Message::DeleteSnippet(id))
        .padding([SPACE_XS, SPACE_SM])
        .style(danger_button_style);

    let info_column = column![
        row![label_element, language_badge]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
        preview_text,
        line_count,
    ]
    .spacing(SPACE_XS);

    let action_row = row![copy_button, edit_button, delete_button]
        .spacing(SPACE_XS)
        .align_y(iced::Alignment::Center);

    let card = row![code_icon, info_column, action_row]
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

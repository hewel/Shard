//! View module containing UI components.

pub mod color_card;
pub mod color_picker;

pub use color_card::view_color_card;
pub use color_picker::{view_color_picker_modal, ColorPickerState, PickerMode};

use iced::widget::{
    button, checkbox, column, container, row, scrollable, stack, text, text_input, Id,
};
use iced::{Element, Length};

use crate::color::Color;
use crate::icons;
use crate::message::Message;
use crate::theme::{
    header_style, input_style, secondary_button_style, status_bar_style, BG_BASE, SPACE_MD,
    SPACE_SM, TEXT_SECONDARY,
};

/// Input field ID for keyboard focus.
pub const COLOR_INPUT_ID: &str = "color_input";

/// Context for rendering the main view.
pub struct ViewContext<'a> {
    pub colors: &'a [Color],
    pub color_input: &'a str,
    pub input_error: Option<&'a str>,
    pub is_listening_clipboard: bool,
    pub editing_label: Option<&'a (i64, String)>,
    pub status_message: Option<&'a str>,
    pub filter_text: &'a str,
    pub selected_color: Option<i64>,
    pub color_picker: Option<&'a ColorPickerState>,
}

/// Render the main application view.
pub fn view(ctx: ViewContext<'_>) -> Element<'_, Message> {
    let ViewContext {
        colors,
        color_input,
        input_error,
        is_listening_clipboard,
        editing_label,
        status_message,
        filter_text,
        selected_color,
        color_picker,
    } = ctx;

    let has_error = input_error.is_some();

    // Header with input and controls
    let color_input_widget = text_input("Enter color (hex, rgb, hsl)...", color_input)
        .id(Id::from(COLOR_INPUT_ID))
        .on_input(Message::ColorInputChanged)
        .on_submit(Message::AddColor)
        .width(Length::FillPortion(3))
        .padding(SPACE_SM)
        .style(move |theme, status| input_style(theme, status, has_error));

    let add_button = button(
        row![icons::plus().size(14), text("Add")]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
    )
    .on_press(Message::AddColor)
    .padding(SPACE_SM)
    .style(crate::theme::primary_button_style);

    let picker_button = button(
        row![icons::palette().size(14), text("Picker")]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
    )
    .on_press(Message::OpenColorPicker(None))
    .padding(SPACE_SM)
    .style(secondary_button_style);

    let clipboard_toggle = row![
        checkbox(is_listening_clipboard).on_toggle(Message::ToggleClipboard),
        text("Listen Clipboard").size(14).color(TEXT_SECONDARY),
    ]
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center);

    // Filter input with clear button
    let filter_input = text_input("Filter...", filter_text)
        .on_input(Message::FilterChanged)
        .width(Length::Fixed(150.0))
        .padding(SPACE_SM)
        .size(14)
        .style(|theme, status| input_style(theme, status, false));

    let filter_section: Element<'_, Message> = if filter_text.is_empty() {
        filter_input.into()
    } else {
        row![
            filter_input,
            button(icons::x_circle().size(14))
                .on_press(Message::FilterChanged(String::new()))
                .padding(SPACE_SM)
                .style(secondary_button_style)
        ]
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center)
        .into()
    };

    let input_row = row![
        color_input_widget,
        add_button,
        picker_button,
        clipboard_toggle,
        filter_section,
    ]
    .spacing(SPACE_SM)
    .padding(SPACE_MD)
    .align_y(iced::Alignment::Center);

    // Wrap header in container with header_style
    let header = container(input_row).width(Length::Fill).style(header_style);

    // Error message
    let error_text: Element<'_, Message> = if let Some(error) = input_error {
        container(text(error).size(12).color(crate::theme::DANGER))
            .padding(
                iced::Padding::new(0.0)
                    .left(SPACE_MD)
                    .right(SPACE_MD)
                    .bottom(SPACE_SM),
            )
            .into()
    } else {
        container(text("")).into()
    };

    // Color palette with filtering
    let filtered_colors: Vec<&Color> = if filter_text.trim().is_empty() {
        colors.iter().collect()
    } else {
        let query = filter_text.to_lowercase();
        colors
            .iter()
            .filter(|c| {
                c.label.to_lowercase().contains(&query)
                    || c.to_hex().to_lowercase().contains(&query)
                    || c.to_rgb().to_lowercase().contains(&query)
                    || c.to_hsl().to_lowercase().contains(&query)
            })
            .collect()
    };

    let colors_list: Element<'_, Message> = if colors.is_empty() {
        container(
            text("No colors yet. Add a color above or enable clipboard listening.")
                .size(14)
                .color(TEXT_SECONDARY),
        )
        .padding(SPACE_MD)
        .center_x(Length::Fill)
        .into()
    } else if filtered_colors.is_empty() {
        container(
            text(format!("No colors match '{}'", filter_text))
                .size(14)
                .color(TEXT_SECONDARY),
        )
        .padding(SPACE_MD)
        .center_x(Length::Fill)
        .into()
    } else {
        let items: Vec<Element<'_, Message>> = filtered_colors
            .iter()
            .map(|color| {
                let is_selected = selected_color == Some(color.id);
                view_color_card(color, is_selected, editing_label)
            })
            .collect();

        scrollable(column(items).spacing(SPACE_SM).padding(SPACE_MD))
            .height(Length::Fill)
            .into()
    };

    // Status bar
    let status_text = status_message.unwrap_or("Ready");
    let color_count = if filter_text.trim().is_empty() {
        format!("{} colors", colors.len())
    } else {
        format!("{} / {} colors", filtered_colors.len(), colors.len())
    };
    let status_bar_content = row![
        text(color_count).size(12).color(TEXT_SECONDARY),
        text("|").size(12).color(TEXT_SECONDARY),
        text(status_text).size(12).color(TEXT_SECONDARY),
    ]
    .spacing(SPACE_SM)
    .padding(SPACE_SM);

    // Wrap status bar in container with status_bar_style
    let status_bar = container(status_bar_content)
        .width(Length::Fill)
        .style(status_bar_style);

    // Main layout with BG_BASE background
    let main_content = container(column![header, error_text, colors_list, status_bar])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| iced::widget::container::Style::default().background(BG_BASE));

    // If color picker is open, show modal overlay
    if let Some(picker) = color_picker {
        stack![main_content, view_color_picker_modal(picker)].into()
    } else {
        main_content.into()
    }
}

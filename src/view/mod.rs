//! View module containing UI components.

pub mod color_card;
pub mod color_picker;

pub use color_card::view_color_card;
pub use color_picker::{view_color_picker_modal, ColorPickerState};

use iced::widget::{
    button, checkbox, column, container, row, scrollable, stack, text, text_input, Id,
};
use iced::{Element, Length};

use crate::color::Color;
use crate::message::Message;

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
    // Header with input and controls
    let color_input_widget = text_input("Enter color (hex, rgb, hsl)...", color_input)
        .id(Id::from(COLOR_INPUT_ID))
        .on_input(Message::ColorInputChanged)
        .on_submit(Message::AddColor)
        .width(Length::FillPortion(3))
        .padding(10)
        .style(move |theme, status| {
            if input_error.is_some() {
                iced::widget::text_input::Style {
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.8, 0.2, 0.2),
                        width: 2.0,
                        radius: 4.0.into(),
                    },
                    ..iced::widget::text_input::default(theme, status)
                }
            } else {
                iced::widget::text_input::default(theme, status)
            }
        });

    let add_button = button(text("Add")).on_press(Message::AddColor).padding(10);

    let picker_button = button(text("Picker"))
        .on_press(Message::OpenColorPicker(None))
        .padding(10);

    let clipboard_toggle = row![
        checkbox(is_listening_clipboard).on_toggle(Message::ToggleClipboard),
        text("Listen Clipboard").size(14),
    ]
    .spacing(5)
    .align_y(iced::Alignment::Center);

    // Filter input with clear button
    let filter_input = text_input("Filter...", filter_text)
        .on_input(Message::FilterChanged)
        .width(Length::Fixed(150.0))
        .padding(10)
        .size(14);

    let filter_section: Element<'_, Message> = if filter_text.is_empty() {
        filter_input.into()
    } else {
        row![
            filter_input,
            button(text("×").size(14))
                .on_press(Message::FilterChanged(String::new()))
                .padding(10)
                .style(button::text)
        ]
        .spacing(5)
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
    .spacing(10)
    .padding(15)
    .align_y(iced::Alignment::Center);

    // Error message
    let error_text: Element<'_, Message> = if let Some(error) = input_error {
        container(
            text(error)
                .size(12)
                .color(iced::Color::from_rgb(0.9, 0.3, 0.3)),
        )
        .padding(iced::Padding::new(0.0).left(15.0).right(15.0).bottom(10.0))
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
                .color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
        )
        .padding(20)
        .center_x(Length::Fill)
        .into()
    } else if filtered_colors.is_empty() {
        container(
            text(format!("No colors match '{}'", filter_text))
                .size(14)
                .color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
        )
        .padding(20)
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

        scrollable(column(items).spacing(10).padding(15))
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
    let status_bar = row![
        text(color_count).size(12),
        text("|").size(12),
        text(status_text).size(12),
    ]
    .spacing(10)
    .padding(10);

    // Main layout
    let main_content = column![input_row, error_text, colors_list, status_bar];

    // If color picker is open, show modal overlay
    if let Some(picker) = color_picker {
        stack![main_content, view_color_picker_modal(picker)].into()
    } else {
        main_content.into()
    }
}

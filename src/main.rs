mod color;
mod db;

use color::{extract_colors_from_text, Color};
use iced::widget::{
    button, canvas, checkbox, column, container, row, scrollable, text, text_input, Canvas,
};
use iced::{mouse, Element, Length, Rectangle, Renderer, Task, Theme};

pub fn main() -> iced::Result {
    iced::application(Shard::new, Shard::update, Shard::view)
        .title("Shard - Color Palette Manager")
        .theme(Shard::theme)
        .subscription(Shard::subscription)
        .run()
}

#[derive(Default)]
struct Shard {
    colors: Vec<Color>,
    color_input: String,
    input_error: Option<String>,
    is_listening_clipboard: bool,
    last_clipboard_content: Option<String>,
    editing_label: Option<(i64, String)>, // (color_id, current_edit_text)
    status_message: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    // Initialization
    ColorsLoaded(Result<Vec<Color>, String>),

    // Color input
    ColorInputChanged(String),
    AddColor,
    ColorAdded(Result<Color, String>),

    // Color actions
    CopyHex(i64),
    CopyRgb(i64),
    CopyHsl(i64),
    CopyFinished(Result<String, String>),
    DeleteColor(i64),
    ColorDeleted(Result<i64, String>),

    // Label editing
    StartEditLabel(i64),
    EditLabelChanged(String),
    SaveLabel,
    CancelEditLabel,
    LabelSaved(Result<(i64, String), String>),

    // Clipboard listening
    ToggleClipboard(bool),
    ClipboardTick,
    ClipboardContentReceived(Option<String>),
}

impl Shard {
    fn new() -> (Self, Task<Message>) {
        let load_task = Task::perform(async { db::load_colors() }, Message::ColorsLoaded);

        (Self::default(), load_task)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ColorsLoaded(result) => {
                match result {
                    Ok(colors) => {
                        self.status_message = Some(format!("{} colors loaded", colors.len()));
                        self.colors = colors;
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Load error: {}", e));
                    }
                }
                Task::none()
            }

            Message::ColorInputChanged(input) => {
                self.color_input = input.clone();

                // Real-time validation
                if input.trim().is_empty() {
                    self.input_error = None;
                } else {
                    match Color::parse(&input) {
                        Ok(_) => self.input_error = None,
                        Err(e) => self.input_error = Some(e.to_string()),
                    }
                }
                Task::none()
            }

            Message::AddColor => {
                let input = self.color_input.clone();
                if input.trim().is_empty() {
                    return Task::none();
                }

                match Color::parse(&input) {
                    Ok(mut color) => {
                        if color.label.is_empty() {
                            color.label = color.default_label();
                        }

                        Task::perform(
                            async move { db::add_or_move_color(color) },
                            Message::ColorAdded,
                        )
                    }
                    Err(e) => {
                        self.input_error = Some(e.to_string());
                        Task::none()
                    }
                }
            }

            Message::ColorAdded(result) => {
                match result {
                    Ok(color) => {
                        // Remove if already exists (for move-to-top case)
                        self.colors.retain(|c| c.id != color.id);
                        // Add at the beginning
                        self.colors.insert(0, color);
                        self.color_input.clear();
                        self.input_error = None;
                        self.status_message = Some("Color added".to_string());
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Error: {}", e));
                    }
                }
                Task::none()
            }

            Message::CopyHex(id) => {
                if let Some(color) = self.colors.iter().find(|c| c.id == id) {
                    let hex = color.to_hex();
                    Task::perform(
                        async move {
                            match arboard::Clipboard::new() {
                                Ok(mut clipboard) => {
                                    clipboard.set_text(&hex).map_err(|e| e.to_string())?;
                                    Ok(format!("Copied: {}", hex))
                                }
                                Err(e) => Err(e.to_string()),
                            }
                        },
                        Message::CopyFinished,
                    )
                } else {
                    Task::none()
                }
            }

            Message::CopyRgb(id) => {
                if let Some(color) = self.colors.iter().find(|c| c.id == id) {
                    let rgb = color.to_rgb();
                    Task::perform(
                        async move {
                            match arboard::Clipboard::new() {
                                Ok(mut clipboard) => {
                                    clipboard.set_text(&rgb).map_err(|e| e.to_string())?;
                                    Ok(format!("Copied: {}", rgb))
                                }
                                Err(e) => Err(e.to_string()),
                            }
                        },
                        Message::CopyFinished,
                    )
                } else {
                    Task::none()
                }
            }

            Message::CopyHsl(id) => {
                if let Some(color) = self.colors.iter().find(|c| c.id == id) {
                    let hsl = color.to_hsl();
                    Task::perform(
                        async move {
                            match arboard::Clipboard::new() {
                                Ok(mut clipboard) => {
                                    clipboard.set_text(&hsl).map_err(|e| e.to_string())?;
                                    Ok(format!("Copied: {}", hsl))
                                }
                                Err(e) => Err(e.to_string()),
                            }
                        },
                        Message::CopyFinished,
                    )
                } else {
                    Task::none()
                }
            }

            Message::CopyFinished(result) => {
                match result {
                    Ok(msg) => self.status_message = Some(msg),
                    Err(e) => self.status_message = Some(format!("Copy failed: {}", e)),
                }
                Task::none()
            }

            Message::DeleteColor(id) => {
                Task::perform(async move { db::delete_color(id) }, Message::ColorDeleted)
            }

            Message::ColorDeleted(result) => {
                match result {
                    Ok(id) => {
                        self.colors.retain(|c| c.id != id);
                        self.status_message = Some("Color deleted".to_string());
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Delete failed: {}", e));
                    }
                }
                Task::none()
            }

            Message::StartEditLabel(id) => {
                if let Some(color) = self.colors.iter().find(|c| c.id == id) {
                    self.editing_label = Some((id, color.label.clone()));
                }
                Task::none()
            }

            Message::EditLabelChanged(text) => {
                if let Some((id, _)) = &self.editing_label {
                    self.editing_label = Some((*id, text));
                }
                Task::none()
            }

            Message::SaveLabel => {
                if let Some((id, label)) = self.editing_label.take() {
                    Task::perform(
                        async move { db::update_label(id, label) },
                        Message::LabelSaved,
                    )
                } else {
                    Task::none()
                }
            }

            Message::CancelEditLabel => {
                self.editing_label = None;
                Task::none()
            }

            Message::LabelSaved(result) => {
                match result {
                    Ok((id, label)) => {
                        if let Some(color) = self.colors.iter_mut().find(|c| c.id == id) {
                            color.label = label;
                        }
                        self.status_message = Some("Label saved".to_string());
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Save failed: {}", e));
                    }
                }
                Task::none()
            }

            Message::ToggleClipboard(enabled) => {
                self.is_listening_clipboard = enabled;
                if enabled {
                    self.status_message = Some("Clipboard listening enabled".to_string());
                } else {
                    self.status_message = Some("Clipboard listening disabled".to_string());
                }
                Task::none()
            }

            Message::ClipboardTick => Task::perform(
                async {
                    match arboard::Clipboard::new() {
                        Ok(mut clipboard) => clipboard.get_text().ok(),
                        Err(_) => None,
                    }
                },
                Message::ClipboardContentReceived,
            ),

            Message::ClipboardContentReceived(content) => {
                if let Some(text) = content {
                    if !text.is_empty() && Some(&text) != self.last_clipboard_content.as_ref() {
                        self.last_clipboard_content = Some(text.clone());

                        // Extract colors from clipboard content
                        let colors = extract_colors_from_text(&text);
                        if !colors.is_empty() {
                            // Add the first detected color
                            let mut color = colors.into_iter().next().expect("checked not empty");
                            if color.label.is_empty() {
                                color.label = color.default_label();
                            }

                            return Task::perform(
                                async move { db::add_or_move_color(color) },
                                Message::ColorAdded,
                            );
                        }
                    }
                }
                Task::none()
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        if self.is_listening_clipboard {
            iced::time::every(std::time::Duration::from_millis(500)).map(|_| Message::ClipboardTick)
        } else {
            iced::Subscription::none()
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Header with input and controls
        let color_input = text_input("Enter color (hex, rgb, hsl)...", &self.color_input)
            .on_input(Message::ColorInputChanged)
            .on_submit(Message::AddColor)
            .width(Length::FillPortion(3))
            .padding(10)
            .style(move |theme, status| {
                if self.input_error.is_some() {
                    text_input::Style {
                        border: iced::Border {
                            color: iced::Color::from_rgb(0.8, 0.2, 0.2),
                            width: 2.0,
                            radius: 4.0.into(),
                        },
                        ..text_input::default(theme, status)
                    }
                } else {
                    text_input::default(theme, status)
                }
            });

        let add_button = button(text("Add")).on_press(Message::AddColor).padding(10);

        let clipboard_toggle = row![
            checkbox(self.is_listening_clipboard).on_toggle(Message::ToggleClipboard),
            text("Listen Clipboard").size(14),
        ]
        .spacing(5)
        .align_y(iced::Alignment::Center);

        let input_row = row![color_input, add_button, clipboard_toggle]
            .spacing(10)
            .padding(15)
            .align_y(iced::Alignment::Center);

        // Error message
        let error_text: Element<'_, Message> = if let Some(error) = &self.input_error {
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

        // Color palette
        let colors_list: Element<'_, Message> = if self.colors.is_empty() {
            container(
                text("No colors yet. Add a color above or enable clipboard listening.")
                    .size(14)
                    .color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
            )
            .padding(20)
            .center_x(Length::Fill)
            .into()
        } else {
            let items: Vec<Element<'_, Message>> = self
                .colors
                .iter()
                .map(|color| self.view_color_card(color))
                .collect();

            scrollable(column(items).spacing(10).padding(15))
                .height(Length::Fill)
                .into()
        };

        // Status bar
        let status_text = self
            .status_message
            .clone()
            .unwrap_or_else(|| "Ready".to_string());
        let color_count = format!("{} colors", self.colors.len());
        let status_bar = row![
            text(color_count).size(12),
            text("|").size(12),
            text(status_text).size(12),
        ]
        .spacing(10)
        .padding(10);

        // Main layout
        column![input_row, error_text, colors_list, status_bar].into()
    }

    fn view_color_card<'a>(&'a self, color: &'a Color) -> Element<'a, Message> {
        let swatch = Canvas::new(ColorSwatch {
            color: color.to_iced_color(),
        })
        .width(50)
        .height(50);

        let is_editing = self.editing_label.as_ref().map(|(id, _)| *id) == Some(color.id);

        let label_element: Element<'_, Message> = if is_editing {
            let edit_text = self
                .editing_label
                .as_ref()
                .map(|(_, t)| t.as_str())
                .unwrap_or("");
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

        let delete_button = button(
            text("Del")
                .size(11)
                .color(iced::Color::from_rgb(0.9, 0.3, 0.3)),
        )
        .on_press(Message::DeleteColor(color.id))
        .padding(5)
        .style(button::text);

        let info_column = column![label_element, hex_display].spacing(4);

        let card = row![swatch, info_column, copy_buttons, delete_button]
            .spacing(15)
            .padding(10)
            .align_y(iced::Alignment::Center);

        container(card)
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();
                container::Style::default()
                    .background(palette.background.weak.color)
                    .border(iced::Border {
                        color: palette.background.strong.color,
                        width: 1.0,
                        radius: 8.0.into(),
                    })
            })
            .width(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

/// A canvas program that draws a color swatch with a checkerboard background for transparency.
struct ColorSwatch {
    color: iced::Color,
}

impl canvas::Program<Message> for ColorSwatch {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Draw checkerboard pattern for transparency preview
        let check_size = 8.0;
        let light = iced::Color::from_rgb(0.8, 0.8, 0.8);
        let dark = iced::Color::from_rgb(0.5, 0.5, 0.5);

        let cols = (bounds.width / check_size).ceil() as usize;
        let rows = (bounds.height / check_size).ceil() as usize;

        for row in 0..rows {
            for col in 0..cols {
                let is_light = (row + col) % 2 == 0;
                let color = if is_light { light } else { dark };
                frame.fill_rectangle(
                    iced::Point::new(col as f32 * check_size, row as f32 * check_size),
                    iced::Size::new(check_size, check_size),
                    color,
                );
            }
        }

        // Draw the actual color on top
        frame.fill_rectangle(iced::Point::ORIGIN, bounds.size(), self.color);

        // Draw border
        frame.stroke(
            &canvas::Path::rectangle(iced::Point::ORIGIN, bounds.size()),
            canvas::Stroke::default()
                .with_color(iced::Color::from_rgb(0.3, 0.3, 0.3))
                .with_width(1.0),
        );

        vec![frame.into_geometry()]
    }
}

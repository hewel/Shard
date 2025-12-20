//! Update logic for the Shard application.

use iced::widget::operation;
use iced::Task;

use crate::color::{extract_colors_from_text, Color};
use crate::db;
use crate::message::Message;
use crate::view::{ColorPickerState, PickerMode, COLOR_INPUT_ID};

/// Application state.
#[derive(Default)]
pub struct Shard {
    pub colors: Vec<Color>,
    pub color_input: String,
    pub input_error: Option<String>,
    pub is_listening_clipboard: bool,
    pub last_clipboard_content: Option<String>,
    pub editing_label: Option<(i64, String)>,
    pub status_message: Option<String>,
    pub filter_text: String,
    pub selected_color: Option<i64>,
    pub color_picker: Option<ColorPickerState>,
}

impl Shard {
    /// Create a new application instance.
    pub fn new() -> (Self, Task<Message>) {
        let load_task = Task::perform(async { db::load_colors() }, Message::ColorsLoaded);
        (Self::default(), load_task)
    }

    /// Handle application messages.
    pub fn update(&mut self, message: Message) -> Task<Message> {
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

            Message::CopyHex(id) => self.copy_color_format(id, |c| c.to_hex()),

            Message::CopyRgb(id) => self.copy_color_format(id, |c| c.to_rgb()),

            Message::CopyHsl(id) => self.copy_color_format(id, |c| c.to_hsl()),

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

            Message::FilterChanged(text) => {
                self.filter_text = text;
                Task::none()
            }

            Message::PasteFromClipboard => {
                // Read clipboard and try to add as color
                Task::perform(
                    async {
                        match arboard::Clipboard::new() {
                            Ok(mut clipboard) => clipboard.get_text().ok(),
                            Err(_) => None,
                        }
                    },
                    |content| {
                        if let Some(text) = content {
                            Message::ColorInputChanged(text)
                        } else {
                            Message::ColorInputChanged(String::new())
                        }
                    },
                )
            }

            Message::FocusColorInput => operation::focus(COLOR_INPUT_ID),

            Message::EscapePressed => {
                // Priority: close color picker > cancel label editing > clear filter > deselect
                if self.color_picker.is_some() {
                    self.color_picker = None;
                } else if self.editing_label.is_some() {
                    self.editing_label = None;
                } else if !self.filter_text.is_empty() {
                    self.filter_text.clear();
                } else {
                    self.selected_color = None;
                }
                Task::none()
            }

            Message::DeleteSelectedColor => {
                if let Some(id) = self.selected_color {
                    self.selected_color = None;
                    Task::perform(async move { db::delete_color(id) }, Message::ColorDeleted)
                } else {
                    Task::none()
                }
            }

            Message::SelectColor(id) => {
                self.selected_color = id;
                Task::none()
            }

            // Color picker messages
            Message::OpenColorPicker(id) => {
                self.color_picker = Some(if let Some(color_id) = id {
                    // Edit existing color
                    if let Some(color) = self.colors.iter().find(|c| c.id == color_id) {
                        ColorPickerState::edit_color(color)
                    } else {
                        ColorPickerState::new_color()
                    }
                } else {
                    // New color
                    ColorPickerState::new_color()
                });
                Task::none()
            }

            Message::CloseColorPicker => {
                self.color_picker = None;
                Task::none()
            }

            Message::PickerHueChanged(hue) => {
                if let Some(picker) = &mut self.color_picker {
                    picker.hue = hue;
                }
                Task::none()
            }

            Message::PickerSaturationChanged(saturation) => {
                if let Some(picker) = &mut self.color_picker {
                    picker.saturation = saturation;
                }
                Task::none()
            }

            Message::PickerLightnessChanged(lightness) => {
                if let Some(picker) = &mut self.color_picker {
                    picker.lightness = lightness;
                }
                Task::none()
            }

            Message::PickerSLChanged(saturation, lightness) => {
                if let Some(picker) = &mut self.color_picker {
                    picker.saturation = saturation;
                    picker.lightness = lightness;
                }
                Task::none()
            }

            Message::PickerAlphaChanged(alpha) => {
                if let Some(picker) = &mut self.color_picker {
                    picker.alpha = alpha;
                }
                Task::none()
            }

            Message::PickerLabelChanged(label) => {
                if let Some(picker) = &mut self.color_picker {
                    picker.label = label;
                }
                Task::none()
            }

            Message::PickerModeChanged(mode) => {
                if let Some(picker) = &mut self.color_picker {
                    // Sync values when switching modes
                    match mode {
                        PickerMode::Hsl => {
                            // Switching to HSL: sync HSL from current RGB
                            picker.sync_hsl_from_rgb();
                        }
                        PickerMode::Oklch => {
                            // Switching to OKLCH: sync OKLCH from current RGB
                            picker.sync_oklch_from_rgb();
                        }
                    }
                    picker.mode = mode;
                }
                Task::none()
            }

            Message::PickerOklchLChanged(l) => {
                if let Some(picker) = &mut self.color_picker {
                    picker.oklch_l = l;
                }
                Task::none()
            }

            Message::PickerOklchCChanged(c) => {
                if let Some(picker) = &mut self.color_picker {
                    picker.oklch_c = c;
                }
                Task::none()
            }

            Message::PickerOklchHChanged(h) => {
                if let Some(picker) = &mut self.color_picker {
                    picker.oklch_h = h;
                }
                Task::none()
            }

            Message::PickerCLChanged(chroma, lightness) => {
                if let Some(picker) = &mut self.color_picker {
                    picker.oklch_c = chroma;
                    picker.oklch_l = lightness;
                }
                Task::none()
            }

            Message::ConfirmColorPicker => {
                if let Some(picker) = self.color_picker.take() {
                    let color = picker.to_color();
                    if let Some(editing_id) = picker.editing_id {
                        // Update existing color
                        let (r, g, b) = picker.to_rgb();
                        let label = color.label.clone();
                        let alpha = picker.alpha;
                        Task::perform(
                            async move { db::update_color(editing_id, r, g, b, alpha, label) },
                            Message::ColorUpdated,
                        )
                    } else {
                        // Add new color
                        Task::perform(
                            async move { db::add_or_move_color(color) },
                            Message::ColorAdded,
                        )
                    }
                } else {
                    Task::none()
                }
            }

            Message::ColorUpdated(result) => {
                match result {
                    Ok(color) => {
                        // Update the color in the list
                        if let Some(existing) = self.colors.iter_mut().find(|c| c.id == color.id) {
                            *existing = color;
                        }
                        self.status_message = Some("Color updated".to_string());
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Update failed: {}", e));
                    }
                }
                Task::none()
            }
        }
    }

    /// Helper to copy a color format to clipboard.
    fn copy_color_format<F>(&self, id: i64, format_fn: F) -> Task<Message>
    where
        F: FnOnce(&Color) -> String + Send + 'static,
    {
        if let Some(color) = self.colors.iter().find(|c| c.id == id) {
            let text = format_fn(color);
            Task::perform(
                async move {
                    match arboard::Clipboard::new() {
                        Ok(mut clipboard) => {
                            clipboard.set_text(&text).map_err(|e| e.to_string())?;
                            Ok(format!("Copied: {}", text))
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
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Shard - Snippet Manager
//!
//! A desktop application for managing colors, code snippets, and text snippets.

mod config;
mod db;
mod icons;
mod message;
mod snippet;
mod theme;
mod update;
mod view;
mod widgets;

use config::{Modifiers, Shortcut};
use iced::keyboard;
use iced::window;
use iced::{Element, Subscription, Theme};

pub use message::Message;
pub use update::{Shard, WindowKind};

pub fn main() -> iced::Result {
    iced::daemon(Shard::new, Shard::update, Shard::view)
        .font(icons::ICON_FONT_BYTES)
        .font(icons::TEXT_FONT_BYTES)
        .default_font(icons::TEXT_FONT)
        .title(Shard::title)
        .theme(Shard::theme)
        .subscription(Shard::subscription)
        .run()
}

impl Shard {
    /// Render the application view for a specific window.
    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        // Check if this is a pinned window or the main window
        match self.windows.get(&window_id) {
            Some(WindowKind::Pinned(snippet_id)) => {
                // Render pinned snippet view (minimal)
                self.view_pinned_snippet(*snippet_id, window_id)
            }
            Some(WindowKind::Main) | None => {
                // Render main application view
                view::view(view::ViewContext {
                    snippets: &self.snippets,
                    is_listening_clipboard: self.is_listening_clipboard,
                    status_message: self.status_message.as_deref(),
                    filter_text: &self.filter_text,
                    filter_kind: self.filter_kind.as_ref(),
                    selected_snippet: self.selected_snippet,
                    color_picker: self.color_picker.as_ref(),
                    code_editor: self.code_editor.as_ref(),
                    text_editor: self.text_editor.as_ref(),
                    settings: self.settings.as_ref(),
                    add_menu_open: self.add_menu_open,
                    palettes: &self.palettes,
                    filter_palette: self.filter_palette,
                    palette_manager_open: self.palette_manager_open,
                    palette_dropdown_snippet: self.palette_dropdown_snippet,
                    snippet_palettes: &self.snippet_palettes,
                    new_palette_name: &self.new_palette_name,
                })
            }
        }
    }

    /// Render a pinned snippet window.
    fn view_pinned_snippet(&self, snippet_id: i64, window_id: window::Id) -> Element<'_, Message> {
        use iced::widget::{button, center, column, container, row, text, Canvas};
        use crate::snippet::SnippetContent;
        use crate::theme::{
            BG_BASE, SPACE_MD, SPACE_SM, TEXT_MUTED, TEXT_PRIMARY, TEXT_SECONDARY,
            danger_button_style,
        };
        use crate::widgets::ColorSwatch;

        let Some(snippet) = self.snippets.iter().find(|s| s.id == snippet_id) else {
            return center(text("Snippet not found").size(14).color(TEXT_MUTED)).into();
        };

        // Content based on snippet type
        let content: Element<'_, Message> = match &snippet.content {
            SnippetContent::Color(color) => {
                // Large color swatch
                let swatch = Canvas::new(ColorSwatch {
                    color: color.to_iced_color(),
                })
                .width(80)
                .height(80);

                let hex_text = text(color.to_hex()).size(14).color(TEXT_PRIMARY);
                let label_text = text(&snippet.label).size(12).color(TEXT_SECONDARY);

                column![swatch, hex_text, label_text]
                    .spacing(SPACE_SM)
                    .align_x(iced::Alignment::Center)
                    .into()
            }
            SnippetContent::Code(code) => {
                let preview = code.preview(4);
                let lang_text = text(&code.language).size(10).color(TEXT_MUTED);
                let code_text = text(preview).size(11).color(TEXT_PRIMARY);
                let label_text = text(&snippet.label).size(12).color(TEXT_SECONDARY);

                column![label_text, lang_text, code_text]
                    .spacing(SPACE_SM)
                    .into()
            }
            SnippetContent::Text(text_data) => {
                let preview = text_data.preview(4);
                let preview_text = text(preview).size(11).color(TEXT_PRIMARY);
                let label_text = text(&snippet.label).size(12).color(TEXT_SECONDARY);

                column![label_text, preview_text]
                    .spacing(SPACE_SM)
                    .into()
            }
        };

        // Close button
        let close_btn = button(icons::x().size(12))
            .on_press(Message::UnpinSnippet(window_id))
            .padding(SPACE_SM)
            .style(danger_button_style);

        // Copy button
        let copy_btn = button(icons::copy().size(12))
            .on_press(Message::CopySnippet(snippet_id))
            .padding(SPACE_SM)
            .style(crate::theme::subtle_button_style);

        let header = row![copy_btn, close_btn]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center);

        let layout = column![header, content]
            .spacing(SPACE_SM)
            .padding(SPACE_MD)
            .align_x(iced::Alignment::Center);

        container(layout)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .style(|_| iced::widget::container::Style::default().background(BG_BASE))
            .into()
    }

    /// Get the window title for a specific window.
    pub fn title(&self, window_id: window::Id) -> String {
        match self.windows.get(&window_id) {
            Some(WindowKind::Pinned(snippet_id)) => {
                if let Some(snippet) = self.snippets.iter().find(|s| s.id == *snippet_id) {
                    format!("📌 {}", snippet.label)
                } else {
                    "Pinned Snippet".to_string()
                }
            }
            Some(WindowKind::Main) | None => "Shard - Snippet Manager".to_string(),
        }
    }

    /// Get the application theme for a specific window.
    pub fn theme(&self, _window_id: window::Id) -> Theme {
        Theme::Dark
    }

    /// Handle keyboard and clipboard subscriptions.
    pub fn subscription(&self) -> Subscription<Message> {
        // Get recording action and keyboard config
        let recording_action = self.settings.as_ref().and_then(|s| s.recording_action);

        let keyboard_config = self.config.keyboard.clone();

        // Use Subscription::with to pass captured state
        let keyboard_sub = keyboard::listen()
            .with((recording_action, keyboard_config))
            .filter_map(|((recording_action, keyboard_config), event)| {
                let keyboard::Event::KeyPressed { key, modifiers, .. } = event else {
                    return None;
                };

                // If recording, capture the key press for shortcut assignment
                if let Some(action) = recording_action {
                    // Create shortcut from key press
                    let shortcut = create_shortcut_from_key(&key, modifiers);
                    if let Some(shortcut) = shortcut {
                        return Some(Message::ShortcutRecorded(action, shortcut));
                    }
                    return None;
                }

                // Normal mode - check configured shortcuts
                if keyboard_config.paste.matches(&key, modifiers) {
                    Some(Message::PasteFromClipboard)
                } else if keyboard_config.new_color.matches(&key, modifiers) {
                    Some(Message::OpenColorPicker(None))
                } else if keyboard_config.escape.matches(&key, modifiers) {
                    Some(Message::EscapePressed)
                } else if keyboard_config.delete.matches(&key, modifiers) {
                    Some(Message::DeleteSelectedSnippet)
                } else if keyboard_config.copy_snippet.matches(&key, modifiers) {
                    Some(Message::CopySelectedSnippet)
                } else {
                    None
                }
            });

        let clipboard_sub = if self.is_listening_clipboard {
            iced::time::every(std::time::Duration::from_millis(500)).map(|_| Message::ClipboardTick)
        } else {
            Subscription::none()
        };

        // Subscribe to window close events
        let window_close_sub = window::close_events().map(Message::WindowClosed);

        Subscription::batch([keyboard_sub, clipboard_sub, window_close_sub])
    }
}

/// Create a Shortcut from a key press event.
fn create_shortcut_from_key(
    key: &keyboard::Key,
    modifiers: keyboard::Modifiers,
) -> Option<Shortcut> {
    let mods = Modifiers::new(modifiers.command(), modifiers.alt(), modifiers.shift());

    match key {
        keyboard::Key::Character(c) => {
            // Don't capture pure modifier keys
            let c_str = c.to_string();
            if c_str.is_empty() {
                return None;
            }
            Some(Shortcut::char_key(c_str.chars().next()?, mods))
        }
        keyboard::Key::Named(named) => {
            // Allow named keys like Escape, Delete, Enter, etc.
            let name = format!("{:?}", named);
            Some(Shortcut::named(&name, mods))
        }
        _ => None,
    }
}

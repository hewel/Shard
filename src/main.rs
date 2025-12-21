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
use iced::{Element, Subscription, Theme};

pub use message::Message;
pub use update::Shard;

pub fn main() -> iced::Result {
    iced::application(Shard::new, Shard::update, Shard::view)
        .title("Shard - Snippet Manager")
        .font(icons::ICON_FONT_BYTES)
        .font(icons::TEXT_FONT_BYTES)
        .default_font(icons::TEXT_FONT)
        .theme(Shard::theme)
        .subscription(Shard::subscription)
        .run()
}

impl Shard {
    /// Render the application view.
    pub fn view(&self) -> Element<'_, Message> {
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

    /// Get the application theme.
    pub fn theme(&self) -> Theme {
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
                } else {
                    None
                }
            });

        let clipboard_sub = if self.is_listening_clipboard {
            iced::time::every(std::time::Duration::from_millis(500)).map(|_| Message::ClipboardTick)
        } else {
            Subscription::none()
        };

        Subscription::batch([keyboard_sub, clipboard_sub])
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

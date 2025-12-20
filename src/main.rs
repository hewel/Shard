//! Shard - Snippet Manager
//!
//! A desktop application for managing colors, code snippets, and text snippets.

mod db;
mod icons;
mod message;
mod snippet;
mod theme;
mod update;
mod view;
mod widgets;

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
            color_input: &self.color_input,
            input_error: self.input_error.as_deref(),
            is_listening_clipboard: self.is_listening_clipboard,
            editing_label: self.editing_label.as_ref(),
            status_message: self.status_message.as_deref(),
            filter_text: &self.filter_text,
            filter_kind: self.filter_kind.as_ref(),
            selected_snippet: self.selected_snippet,
            color_picker: self.color_picker.as_ref(),
            code_editor: self.code_editor.as_ref(),
            text_editor: self.text_editor.as_ref(),
        })
    }

    /// Get the application theme.
    pub fn theme(&self) -> Theme {
        Theme::Dark
    }

    /// Handle keyboard and clipboard subscriptions.
    pub fn subscription(&self) -> Subscription<Message> {
        use keyboard::key;

        let keyboard_sub = keyboard::listen().filter_map(|event| {
            let keyboard::Event::KeyPressed { key, modifiers, .. } = event else {
                return None;
            };

            match key.as_ref() {
                keyboard::Key::Character("v") if modifiers.command() => {
                    Some(Message::PasteFromClipboard)
                }
                keyboard::Key::Character("n") if modifiers.command() => {
                    Some(Message::FocusColorInput)
                }
                keyboard::Key::Named(key::Named::Escape) => Some(Message::EscapePressed),
                keyboard::Key::Named(key::Named::Delete) => Some(Message::DeleteSelectedSnippet),
                _ => None,
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

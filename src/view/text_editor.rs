//! Text editor modal for editing text snippets.

use iced::widget::{
    button, column, container, mouse_area, opaque, row, text, text_editor, text_input,
};
use iced::{Element, Length};

use crate::icons;
use crate::message::Message;
use crate::snippet::{Snippet, SnippetContent};
use crate::theme::{
    input_style, modal_dialog_style, modal_overlay_style, primary_button_style,
    secondary_button_style, subtle_button_style, SPACE_MD, SPACE_SM, SPACE_XS, TEXT_PRIMARY,
    TEXT_SECONDARY,
};

/// State for the text editor modal.
#[derive(Debug, Clone)]
pub struct TextEditorState {
    /// The snippet being edited (Some = editing existing, None = creating new)
    pub editing_id: Option<i64>,
    /// The text content
    pub content: text_editor::Content,
    /// Label for the snippet
    pub label: String,
}

impl TextEditorState {
    /// Create a new editor state for a new text snippet.
    pub fn new_text() -> Self {
        Self {
            editing_id: None,
            content: text_editor::Content::new(),
            label: String::new(),
        }
    }

    /// Create an editor state from an existing snippet.
    pub fn from_snippet(snippet: &Snippet) -> Self {
        if let SnippetContent::Text(text_data) = &snippet.content {
            Self {
                editing_id: Some(snippet.id),
                content: text_editor::Content::with_text(&text_data.text),
                label: snippet.label.clone(),
            }
        } else {
            Self::new_text()
        }
    }

    /// Get the current text.
    pub fn text(&self) -> String {
        self.content.text()
    }
}

/// Render the text editor modal.
pub fn view_text_editor_modal(editor: &TextEditorState) -> Element<'_, Message> {
    let title = if editor.editing_id.is_some() {
        "Edit Text Snippet"
    } else {
        "New Text Snippet"
    };

    // Header
    let header_row = row![
        text(title).size(20).color(TEXT_PRIMARY),
        iced::widget::Space::new().width(Length::Fill),
        button(icons::x().size(16))
            .on_press(Message::CloseTextEditor)
            .padding([SPACE_XS, SPACE_SM])
            .style(subtle_button_style),
    ]
    .align_y(iced::Alignment::Center);

    // Text editor
    let the_text_editor = text_editor(&editor.content)
        .on_action(Message::TextEditorContentChanged)
        .height(Length::Fixed(300.0))
        .padding(SPACE_SM);

    // Label input
    let label_input = row![
        text("Label:").size(12).color(TEXT_SECONDARY),
        text_input("Snippet label...", &editor.label)
            .on_input(Message::TextEditorLabelChanged)
            .padding(SPACE_SM)
            .width(Length::Fill)
            .style(|theme, status| input_style(theme, status, false)),
    ]
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center);

    // Action buttons
    let action_buttons = row![
        button(text("Cancel").size(14))
            .on_press(Message::CloseTextEditor)
            .padding(SPACE_SM)
            .style(secondary_button_style),
        button(
            text(if editor.editing_id.is_some() {
                "Save"
            } else {
                "Add"
            })
            .size(14)
        )
        .on_press(Message::ConfirmTextEditor)
        .padding(SPACE_SM)
        .style(primary_button_style),
    ]
    .spacing(SPACE_SM);

    // Modal content
    let modal_content = column![header_row, the_text_editor, label_input, action_buttons,]
        .spacing(SPACE_MD)
        .padding(SPACE_MD)
        .width(Length::Fixed(500.0));

    let modal_dialog = container(modal_content).style(modal_dialog_style);

    // Semi-transparent overlay
    let overlay = mouse_area(
        container(opaque(modal_dialog))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(modal_overlay_style),
    )
    .on_press(Message::CloseTextEditor);

    overlay.into()
}

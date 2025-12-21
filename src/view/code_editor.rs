//! Code editor modal for editing code snippets.

use iced::highlighter;
use iced::widget::{
    button, column, container, mouse_area, opaque, row, text, text_editor, text_input,
};
use iced::{Element, Length};

use crate::icons;
use crate::message::Message;
use crate::snippet::{detect_language, language_to_extension, Snippet, SnippetContent};
use crate::theme::{
    input_style, modal_dialog_style, modal_overlay_style, primary_button_style,
    secondary_button_style, subtle_button_style, SPACE_MD, SPACE_SM, SPACE_XS, TEXT_PRIMARY,
    TEXT_SECONDARY,
};

/// State for the code editor modal.
#[derive(Debug, Clone)]
pub struct CodeEditorState {
    /// The snippet being edited (Some = editing existing, None = creating new)
    pub editing_id: Option<i64>,
    /// The code content
    pub content: text_editor::Content,
    /// The detected/selected language
    pub language: String,
    /// Label for the snippet
    pub label: String,
}

impl CodeEditorState {
    /// Create a new editor state for a new code snippet.
    pub fn new_code() -> Self {
        Self {
            editing_id: None,
            content: text_editor::Content::new(),
            language: "plain".to_string(),
            label: String::new(),
        }
    }

    /// Create an editor state from an existing snippet.
    pub fn from_snippet(snippet: &Snippet) -> Self {
        if let SnippetContent::Code(code) = &snippet.content {
            Self {
                editing_id: Some(snippet.id),
                content: text_editor::Content::with_text(&code.code),
                language: code.language.clone(),
                label: snippet.label.clone(),
            }
        } else {
            Self::new_code()
        }
    }

    /// Get the current code text.
    pub fn code(&self) -> String {
        self.content.text()
    }

    /// Detect language from current content.
    pub fn detect_language(&mut self) {
        let code = self.content.text();
        self.language = detect_language(&code);
    }
}

/// Render the code editor modal.
pub fn view_code_editor_modal(editor: &CodeEditorState) -> Element<'_, Message> {
    let title = if editor.editing_id.is_some() {
        "Edit Code Snippet"
    } else {
        "New Code Snippet"
    };

    // Header
    let external_editor_button = if let Some(id) = editor.editing_id {
        button(icons::arrow_square_out().size(14))
            .on_press(Message::OpenInExternalEditor(id, true))
            .padding([SPACE_XS, SPACE_SM])
            .style(subtle_button_style)
    } else {
        button(icons::arrow_square_out().size(14))
            .padding([SPACE_XS, SPACE_SM])
            .style(subtle_button_style)
    };

    let header_row = row![
        text(title).size(20).color(TEXT_PRIMARY),
        iced::widget::Space::new().width(Length::Fill),
        external_editor_button,
        button(icons::x().size(16))
            .on_press(Message::CloseCodeEditor)
            .padding([SPACE_XS, SPACE_SM])
            .style(subtle_button_style),
    ]
    .spacing(SPACE_XS)
    .align_y(iced::Alignment::Center);

    // Code editor with syntax highlighting
    let extension = language_to_extension(&editor.language);
    let code_editor = text_editor(&editor.content)
        .on_action(Message::CodeEditorContentChanged)
        .height(Length::Fixed(300.0))
        .padding(SPACE_SM)
        .highlight(extension, highlighter::Theme::SolarizedDark);

    // Language input
    let language_input = row![
        text("Language:").size(12).color(TEXT_SECONDARY),
        text_input("plain", &editor.language)
            .on_input(Message::CodeEditorLanguageChanged)
            .padding(SPACE_SM)
            .width(Length::Fixed(120.0))
            .style(|theme, status| input_style(theme, status, false)),
    ]
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center);

    // Label input
    let label_input = row![
        text("Label:").size(12).color(TEXT_SECONDARY),
        text_input("Snippet label...", &editor.label)
            .on_input(Message::CodeEditorLabelChanged)
            .padding(SPACE_SM)
            .width(Length::Fill)
            .style(|theme, status| input_style(theme, status, false)),
    ]
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center);

    // Action buttons
    let action_buttons = row![
        button(text("Cancel").size(14))
            .on_press(Message::CloseCodeEditor)
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
        .on_press(Message::ConfirmCodeEditor)
        .padding(SPACE_SM)
        .style(primary_button_style),
    ]
    .spacing(SPACE_SM);

    // Modal content
    let modal_content = column![
        header_row,
        code_editor,
        language_input,
        label_input,
        action_buttons,
    ]
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
    .on_press(Message::CloseCodeEditor);

    overlay.into()
}

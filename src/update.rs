//! Update logic for the Shard application.

use iced::widget::operation;
use iced::Task;

use crate::db;
use crate::message::Message;
use crate::snippet::{
    detect_snippet_type, extract_colors_from_text, ColorData, Snippet, SnippetContent, SnippetKind,
};
use crate::view::{CodeEditorState, ColorPickerState, PickerMode, TextEditorState, COLOR_INPUT_ID};

/// Application state.
#[derive(Default)]
pub struct Shard {
    pub snippets: Vec<Snippet>,
    pub color_input: String,
    pub input_error: Option<String>,
    pub is_listening_clipboard: bool,
    pub last_clipboard_content: Option<String>,
    pub editing_label: Option<(i64, String)>,
    pub status_message: Option<String>,
    pub filter_text: String,
    pub filter_kind: Option<SnippetKind>,
    pub selected_snippet: Option<i64>,
    pub color_picker: Option<ColorPickerState>,
    pub code_editor: Option<CodeEditorState>,
    pub text_editor: Option<TextEditorState>,
}

impl Shard {
    /// Create a new application instance.
    pub fn new() -> (Self, Task<Message>) {
        let load_task = Task::perform(async { db::load_snippets() }, Message::SnippetsLoaded);
        (Self::default(), load_task)
    }

    /// Handle application messages.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SnippetsLoaded(result) => {
                match result {
                    Ok(snippets) => {
                        self.status_message = Some(format!("{} snippets loaded", snippets.len()));
                        self.snippets = snippets;
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Load error: {}", e));
                    }
                }
                Task::none()
            }

            Message::ColorInputChanged(input) => {
                self.color_input = input.clone();

                // Real-time validation for color input
                if input.trim().is_empty() {
                    self.input_error = None;
                } else {
                    match ColorData::parse(&input) {
                        Ok(_) => self.input_error = None,
                        Err(e) => self.input_error = Some(e.to_string()),
                    }
                }
                Task::none()
            }

            Message::AddColorFromInput => {
                let input = self.color_input.clone();
                if input.trim().is_empty() {
                    return Task::none();
                }

                match ColorData::parse(&input) {
                    Ok(color) => {
                        let label = color.to_hex();
                        Task::perform(
                            async move {
                                db::add_or_move_color(color.r, color.g, color.b, color.a, label)
                            },
                            Message::SnippetAdded,
                        )
                    }
                    Err(e) => {
                        self.input_error = Some(e.to_string());
                        Task::none()
                    }
                }
            }

            Message::SnippetAdded(result) => {
                match result {
                    Ok(snippet) => {
                        // Remove if already exists (for move-to-top case)
                        self.snippets.retain(|s| s.id != snippet.id);
                        // Add at the beginning
                        self.snippets.insert(0, snippet);
                        self.color_input.clear();
                        self.input_error = None;
                        self.status_message = Some("Snippet added".to_string());
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Error: {}", e));
                    }
                }
                Task::none()
            }

            Message::CopySnippet(id) => {
                if let Some(snippet) = self.snippets.iter().find(|s| s.id == id) {
                    let text = snippet.content.to_copyable_string();
                    Task::perform(
                        async move { copy_to_clipboard(&text).await },
                        Message::CopyFinished,
                    )
                } else {
                    Task::none()
                }
            }

            Message::CopyHex(id) => self.copy_color_format(id, |c| c.to_hex()),
            Message::CopyRgb(id) => self.copy_color_format(id, |c| c.to_rgb()),
            Message::CopyHsl(id) => self.copy_color_format(id, |c| c.to_hsl()),
            Message::CopyOklch(id) => self.copy_color_format(id, |c| c.to_oklch()),

            Message::CopyFinished(result) => {
                match result {
                    Ok(msg) => self.status_message = Some(msg),
                    Err(e) => self.status_message = Some(format!("Copy failed: {}", e)),
                }
                Task::none()
            }

            Message::DeleteSnippet(id) => Task::perform(
                async move { db::delete_snippet(id) },
                Message::SnippetDeleted,
            ),

            Message::SnippetDeleted(result) => {
                match result {
                    Ok(id) => {
                        self.snippets.retain(|s| s.id != id);
                        self.status_message = Some("Snippet deleted".to_string());
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Delete failed: {}", e));
                    }
                }
                Task::none()
            }

            Message::SelectSnippet(id) => {
                self.selected_snippet = id;
                Task::none()
            }

            Message::StartEditLabel(id) => {
                if let Some(snippet) = self.snippets.iter().find(|s| s.id == id) {
                    self.editing_label = Some((id, snippet.label.clone()));
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
                        if let Some(snippet) = self.snippets.iter_mut().find(|s| s.id == id) {
                            snippet.label = label;
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

                        // Detect snippet type and add accordingly
                        if let Some(kind) = detect_snippet_type(&text) {
                            match kind {
                                SnippetKind::Color => {
                                    let colors = extract_colors_from_text(&text);
                                    if let Some(color) = colors.into_iter().next() {
                                        let label = color.to_hex();
                                        return Task::perform(
                                            async move {
                                                db::add_or_move_color(
                                                    color.r, color.g, color.b, color.a, label,
                                                )
                                            },
                                            Message::SnippetAdded,
                                        );
                                    }
                                }
                                SnippetKind::Code => {
                                    let code = text.clone();
                                    return Task::perform(
                                        async move {
                                            db::add_code_snippet(code, String::new(), String::new())
                                        },
                                        Message::SnippetAdded,
                                    );
                                }
                                SnippetKind::Text => {
                                    let text_content = text.clone();
                                    return Task::perform(
                                        async move { db::add_text_snippet(text_content, String::new()) },
                                        Message::SnippetAdded,
                                    );
                                }
                            }
                        }
                    }
                }
                Task::none()
            }

            Message::FilterChanged(text) => {
                self.filter_text = text;
                Task::none()
            }

            Message::FilterKindChanged(kind) => {
                self.filter_kind = kind;
                Task::none()
            }

            Message::PasteFromClipboard => Task::perform(
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
            ),

            Message::FocusColorInput => operation::focus(COLOR_INPUT_ID),

            Message::EscapePressed => {
                // Priority: close modals > cancel label editing > clear filter > deselect
                if self.color_picker.is_some() {
                    self.color_picker = None;
                } else if self.code_editor.is_some() {
                    self.code_editor = None;
                } else if self.text_editor.is_some() {
                    self.text_editor = None;
                } else if self.editing_label.is_some() {
                    self.editing_label = None;
                } else if !self.filter_text.is_empty() {
                    self.filter_text.clear();
                } else {
                    self.selected_snippet = None;
                }
                Task::none()
            }

            Message::DeleteSelectedSnippet => {
                if let Some(id) = self.selected_snippet {
                    self.selected_snippet = None;
                    Task::perform(
                        async move { db::delete_snippet(id) },
                        Message::SnippetDeleted,
                    )
                } else {
                    Task::none()
                }
            }

            // === Color Picker Messages ===
            Message::OpenColorPicker(id) => {
                self.color_picker = Some(if let Some(snippet_id) = id {
                    // Edit existing color
                    if let Some(snippet) = self.snippets.iter().find(|s| s.id == snippet_id) {
                        ColorPickerState::from_snippet(snippet)
                    } else {
                        ColorPickerState::new_color()
                    }
                } else {
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
                    match mode {
                        PickerMode::Hsl => picker.sync_hsl_from_rgb(),
                        PickerMode::Oklch => picker.sync_oklch_from_rgb(),
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
                    let (r, g, b) = picker.to_rgb();
                    let alpha = picker.alpha;
                    let label = if picker.label.is_empty() {
                        ColorData::new(r, g, b, alpha).to_hex()
                    } else {
                        picker.label.clone()
                    };

                    if let Some(editing_id) = picker.editing_id {
                        // Update existing color
                        Task::perform(
                            async move { db::update_color(editing_id, r, g, b, alpha, label) },
                            Message::SnippetUpdated,
                        )
                    } else {
                        // Add new color
                        Task::perform(
                            async move { db::add_or_move_color(r, g, b, alpha, label) },
                            Message::SnippetAdded,
                        )
                    }
                } else {
                    Task::none()
                }
            }

            Message::SnippetUpdated(result) => {
                match result {
                    Ok(snippet) => {
                        if let Some(existing) =
                            self.snippets.iter_mut().find(|s| s.id == snippet.id)
                        {
                            *existing = snippet;
                        }
                        self.status_message = Some("Snippet updated".to_string());
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Update failed: {}", e));
                    }
                }
                Task::none()
            }

            // === Code Editor Messages ===
            Message::OpenCodeEditor(id) => {
                self.code_editor = Some(if let Some(snippet_id) = id {
                    if let Some(snippet) = self.snippets.iter().find(|s| s.id == snippet_id) {
                        CodeEditorState::from_snippet(snippet)
                    } else {
                        CodeEditorState::new_code()
                    }
                } else {
                    CodeEditorState::new_code()
                });
                Task::none()
            }

            Message::CloseCodeEditor => {
                self.code_editor = None;
                Task::none()
            }

            Message::CodeEditorContentChanged(action) => {
                if let Some(editor) = &mut self.code_editor {
                    editor.content.perform(action);
                }
                Task::none()
            }

            Message::CodeEditorLanguageChanged(language) => {
                if let Some(editor) = &mut self.code_editor {
                    editor.language = language;
                }
                Task::none()
            }

            Message::CodeEditorLabelChanged(label) => {
                if let Some(editor) = &mut self.code_editor {
                    editor.label = label;
                }
                Task::none()
            }

            Message::ConfirmCodeEditor => {
                if let Some(editor) = self.code_editor.take() {
                    let code = editor.content.text();
                    let language = editor.language.clone();
                    let label = editor.label.clone();

                    if let Some(editing_id) = editor.editing_id {
                        Task::perform(
                            async move { db::update_code(editing_id, code, language, label) },
                            Message::SnippetUpdated,
                        )
                    } else {
                        Task::perform(
                            async move { db::add_code_snippet(code, language, label) },
                            Message::SnippetAdded,
                        )
                    }
                } else {
                    Task::none()
                }
            }

            // === Text Editor Messages ===
            Message::OpenTextEditor(id) => {
                self.text_editor = Some(if let Some(snippet_id) = id {
                    if let Some(snippet) = self.snippets.iter().find(|s| s.id == snippet_id) {
                        TextEditorState::from_snippet(snippet)
                    } else {
                        TextEditorState::new_text()
                    }
                } else {
                    TextEditorState::new_text()
                });
                Task::none()
            }

            Message::CloseTextEditor => {
                self.text_editor = None;
                Task::none()
            }

            Message::TextEditorContentChanged(action) => {
                if let Some(editor) = &mut self.text_editor {
                    editor.content.perform(action);
                }
                Task::none()
            }

            Message::TextEditorLabelChanged(label) => {
                if let Some(editor) = &mut self.text_editor {
                    editor.label = label;
                }
                Task::none()
            }

            Message::ConfirmTextEditor => {
                if let Some(editor) = self.text_editor.take() {
                    let text = editor.content.text();
                    let label = editor.label.clone();

                    if let Some(editing_id) = editor.editing_id {
                        Task::perform(
                            async move { db::update_text(editing_id, text, label) },
                            Message::SnippetUpdated,
                        )
                    } else {
                        Task::perform(
                            async move { db::add_text_snippet(text, label) },
                            Message::SnippetAdded,
                        )
                    }
                } else {
                    Task::none()
                }
            }
        }
    }

    /// Helper to copy a color format to clipboard.
    fn copy_color_format<F>(&self, id: i64, format_fn: F) -> Task<Message>
    where
        F: FnOnce(&ColorData) -> String + Send + 'static,
    {
        if let Some(snippet) = self.snippets.iter().find(|s| s.id == id) {
            if let SnippetContent::Color(color) = &snippet.content {
                let text = format_fn(color);
                return Task::perform(
                    async move { copy_to_clipboard(&text).await },
                    Message::CopyFinished,
                );
            }
        }
        Task::none()
    }
}

/// Copy text to clipboard.
async fn copy_to_clipboard(text: &str) -> Result<String, String> {
    let text = text.to_string();
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => {
            clipboard.set_text(&text).map_err(|e| e.to_string())?;
            Ok(format!("Copied: {}", text))
        }
        Err(e) => Err(e.to_string()),
    }
}

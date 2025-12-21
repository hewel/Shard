//! Update logic for the Shard application.

use iced::widget::operation;
use iced::Task;

use crate::config::Config;
use crate::db;
use crate::message::Message;
use crate::snippet::{
    detect_snippet_type, extract_colors_from_text, language_to_extension, ColorData, Snippet,
    SnippetContent, SnippetKind,
};
use crate::view::{
    CodeEditorState, ColorPickerState, PickerMode, SettingsState, TextEditorState, COLOR_INPUT_ID,
};

/// Application state.
pub struct Shard {
    pub snippets: Vec<Snippet>,
    pub color_input: String,
    pub input_error: Option<String>,
    pub is_listening_clipboard: bool,
    pub last_clipboard_content: Option<String>,
    pub status_message: Option<String>,
    pub filter_text: String,
    pub filter_kind: Option<SnippetKind>,
    pub selected_snippet: Option<i64>,
    pub color_picker: Option<ColorPickerState>,
    pub code_editor: Option<CodeEditorState>,
    pub text_editor: Option<TextEditorState>,
    pub settings: Option<SettingsState>,
    pub config: Config,
    pub add_menu_open: bool,
}

impl Default for Shard {
    fn default() -> Self {
        Self {
            snippets: Vec::new(),
            color_input: String::new(),
            input_error: None,
            is_listening_clipboard: false,
            last_clipboard_content: None,
            status_message: None,
            filter_text: String::new(),
            filter_kind: None,
            selected_snippet: None,
            color_picker: None,
            code_editor: None,
            text_editor: None,
            settings: None,
            config: Config::load(),
            add_menu_open: false,
        }
    }
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

            Message::OpenInExternalEditor(id, is_code) => {
                // Find the snippet content
                let Some(snippet) = self.snippets.iter().find(|s| s.id == id) else {
                    self.status_message = Some("Snippet not found".to_string());
                    return Task::none();
                };

                let content = match &snippet.content {
                    SnippetContent::Code(code) => code.code.clone(),
                    SnippetContent::Text(text) => text.text.clone(),
                    SnippetContent::Color(_) => {
                        self.status_message = Some("Cannot open colors in editor".to_string());
                        return Task::none();
                    }
                };

                // Get file extension
                let extension = if is_code {
                    if let SnippetContent::Code(code) = &snippet.content {
                        language_to_extension(&code.language)
                    } else {
                        "txt"
                    }
                } else {
                    "txt"
                };

                let config = self.config.clone();
                self.status_message = Some("Opening in external editor...".to_string());

                Task::perform(
                    async move {
                        open_in_external_editor(id, &content, extension, is_code, &config).await
                    },
                    Message::ExternalEditorClosed,
                )
            }

            Message::ExternalEditorClosed(result) => {
                match result {
                    Ok((id, new_content, is_code)) => {
                        // Update the snippet in memory and database
                        if is_code {
                            let content = new_content.clone();
                            return Task::perform(
                                async move {
                                    // Get existing snippet to preserve language and label
                                    db::update_code_content(id, content)
                                },
                                Message::SnippetUpdated,
                            );
                        } else {
                            let content = new_content.clone();
                            return Task::perform(
                                async move { db::update_text_content(id, content) },
                                Message::SnippetUpdated,
                            );
                        }
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Editor error: {}", e));
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
                Message::PasteContentReceived,
            ),

            Message::PasteContentReceived(content) => {
                if let Some(text) = content {
                    if !text.is_empty() {
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

            Message::FocusColorInput => operation::focus(COLOR_INPUT_ID),

            Message::EscapePressed => {
                // Priority: close modals/menus > clear filter > deselect
                if self.add_menu_open {
                    self.add_menu_open = false;
                } else if self.settings.is_some() {
                    self.settings = None;
                } else if self.color_picker.is_some() {
                    self.color_picker = None;
                } else if self.code_editor.is_some() {
                    self.code_editor = None;
                } else if self.text_editor.is_some() {
                    self.text_editor = None;
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
                self.add_menu_open = false;
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
                self.add_menu_open = false;
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
                self.add_menu_open = false;
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

            // === Settings Messages ===
            Message::OpenSettings => {
                self.settings = Some(SettingsState::from_config(&self.config));
                Task::none()
            }

            Message::CloseSettings => {
                self.settings = None;
                Task::none()
            }

            Message::SettingsEditorPresetChanged(preset) => {
                if let Some(settings) = &mut self.settings {
                    settings.editor_preset = preset;
                }
                Task::none()
            }

            Message::SettingsCustomCommandChanged(cmd) => {
                if let Some(settings) = &mut self.settings {
                    settings.custom_command = cmd;
                }
                Task::none()
            }

            Message::ConfirmSettings => {
                if let Some(settings) = self.settings.take() {
                    settings.apply_to_config(&mut self.config);
                    let config = self.config.clone();
                    Task::perform(async move { config.save() }, Message::ConfigSaved)
                } else {
                    Task::none()
                }
            }

            Message::ConfigSaved(result) => {
                match result {
                    Ok(()) => {
                        self.status_message = Some("Settings saved".to_string());
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Failed to save settings: {}", e));
                    }
                }
                Task::none()
            }

            // === Add Menu Messages ===
            Message::ToggleAddMenu => {
                self.add_menu_open = !self.add_menu_open;
                Task::none()
            }

            Message::CloseAddMenu => {
                self.add_menu_open = false;
                Task::none()
            }

            // === Export/Import Messages ===
            Message::ExportSnippetsJson => {
                let snippets = self.snippets.clone();
                Task::perform(
                    async move { export_snippets_json(snippets).await },
                    Message::ExportFinished,
                )
            }

            Message::ExportFinished(result) => {
                match result {
                    Ok(msg) => self.status_message = Some(msg),
                    Err(e) => self.status_message = Some(format!("Export failed: {}", e)),
                }
                Task::none()
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
            // Truncate display text for status bar (max 40 chars)
            let display = truncate_for_status(&text, 40);
            Ok(format!("Copied: {}", display))
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Truncate text for status bar display.
fn truncate_for_status(text: &str, max_len: usize) -> String {
    // Take first line only
    let first_line = text.lines().next().unwrap_or(text);
    let trimmed = first_line.trim();

    if trimmed.chars().count() <= max_len {
        trimmed.to_string()
    } else {
        // Truncate with ellipsis
        let truncated: String = trimmed.chars().take(max_len - 1).collect();
        format!("{}…", truncated.trim_end())
    }
}

/// Open content in external editor and return updated content when closed.
async fn open_in_external_editor(
    id: i64,
    content: &str,
    extension: &str,
    is_code: bool,
    config: &crate::config::Config,
) -> Result<(i64, String, bool), String> {
    use std::fs;
    use std::process::Command;

    // Create temp file
    let temp_dir = std::env::temp_dir();
    let file_name = format!("shard_snippet_{}.{}", id, extension);
    let temp_path = temp_dir.join(&file_name);

    // Write content to temp file
    fs::write(&temp_path, content).map_err(|e| format!("Failed to create temp file: {}", e))?;

    // Get command from config
    let temp_path_str = temp_path.to_string_lossy().to_string();
    let Some((program, args)) = config.editor.build_command(&temp_path_str) else {
        // Clean up temp file
        let _ = fs::remove_file(&temp_path);
        return Err("No editor command configured".to_string());
    };

    // Run editor (blocking)
    let result = Command::new(&program)
        .args(&args)
        .status()
        .map_err(|e| format!("Failed to launch editor '{}': {}", program, e))?;

    if !result.success() {
        // Clean up temp file
        let _ = fs::remove_file(&temp_path);
        return Err(format!(
            "Editor exited with code: {}",
            result.code().unwrap_or(-1)
        ));
    }

    // Read updated content
    let new_content =
        fs::read_to_string(&temp_path).map_err(|e| format!("Failed to read temp file: {}", e))?;

    // Clean up temp file
    let _ = fs::remove_file(&temp_path);

    Ok((id, new_content, is_code))
}

/// Export all snippets to a JSON file.
async fn export_snippets_json(
    snippets: Vec<crate::snippet::Snippet>,
) -> Result<String, String> {
    use std::fs;

    let json = serde_json::to_string_pretty(&snippets)
        .map_err(|e| format!("Serialization failed: {}", e))?;

    // Get documents directory or fall back to temp
    let export_dir = directories::UserDirs::new()
        .and_then(|d| d.document_dir().map(|p| p.to_path_buf()))
        .unwrap_or_else(std::env::temp_dir);

    let path = export_dir.join("shard_export.json");

    fs::write(&path, &json).map_err(|e| format!("Write failed: {}", e))?;

    Ok(format!(
        "Exported {} snippets to {}",
        snippets.len(),
        path.display()
    ))
}

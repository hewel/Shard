//! Application messages for the Shard snippet manager.

use crate::snippet::{Snippet, SnippetKind};
use crate::view::PickerMode;

/// All messages that can be sent in the application.
#[derive(Debug, Clone)]
pub enum Message {
    // === Initialization ===
    SnippetsLoaded(Result<Vec<Snippet>, String>),

    // === Snippet Input (Color) ===
    ColorInputChanged(String),
    AddColorFromInput,
    SnippetAdded(Result<Snippet, String>),

    // === Unified Snippet Actions ===
    CopySnippet(i64),
    CopyHex(i64),
    CopyRgb(i64),
    CopyHsl(i64),
    CopyOklch(i64),
    CopyFinished(Result<String, String>),
    DeleteSnippet(i64),
    SnippetDeleted(Result<i64, String>),
    SelectSnippet(Option<i64>),

    // === External Editor ===
    /// Open snippet in external editor (snippet_id, is_code: true=code, false=text)
    OpenInExternalEditor(i64, bool),
    /// Editor closed, content returned (snippet_id, new_content, is_code)
    ExternalEditorClosed(Result<(i64, String, bool), String>),

    // === Clipboard Listening ===
    ToggleClipboard(bool),
    ClipboardTick,
    ClipboardContentReceived(Option<String>),

    // === Filtering ===
    FilterChanged(String),
    FilterKindChanged(Option<SnippetKind>),

    // === Keyboard Shortcuts ===
    PasteFromClipboard,
    FocusColorInput,
    EscapePressed,
    DeleteSelectedSnippet,

    // === Color Picker ===
    OpenColorPicker(Option<i64>), // None = new color, Some(id) = edit existing
    CloseColorPicker,
    PickerModeChanged(PickerMode),
    PickerHueChanged(f32),
    PickerSaturationChanged(f32),
    PickerLightnessChanged(f32),
    PickerSLChanged(f32, f32),
    PickerAlphaChanged(f32),
    PickerLabelChanged(String),
    // OKLCH mode
    PickerOklchLChanged(f32),
    PickerOklchCChanged(f32),
    PickerOklchHChanged(f32),
    PickerCLChanged(f32, f32),
    ConfirmColorPicker,
    SnippetUpdated(Result<Snippet, String>),

    // === Code Editor ===
    OpenCodeEditor(Option<i64>), // None = new, Some(id) = edit existing
    CloseCodeEditor,
    CodeEditorContentChanged(iced::widget::text_editor::Action),
    CodeEditorLanguageChanged(String),
    CodeEditorLabelChanged(String),
    ConfirmCodeEditor,

    // === Text Editor ===
    OpenTextEditor(Option<i64>), // None = new, Some(id) = edit existing
    CloseTextEditor,
    TextEditorContentChanged(iced::widget::text_editor::Action),
    TextEditorLabelChanged(String),
    ConfirmTextEditor,
}

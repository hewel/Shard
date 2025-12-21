//! Application messages for the Shard snippet manager.

use crate::config::{EditorPreset, Shortcut, ShortcutAction};
use crate::db::Palette;
use crate::snippet::{Snippet, SnippetKind};
use crate::view::PickerMode;

/// All messages that can be sent in the application.
#[derive(Debug, Clone)]
pub enum Message {
    // === Initialization ===
    SnippetsLoaded(Result<Vec<Snippet>, String>),
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
    PasteContentReceived(Option<String>),
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
    SaveColorAsNew,
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

    // === Settings ===
    OpenSettings,
    CloseSettings,
    SettingsEditorPresetChanged(EditorPreset),
    SettingsCustomCommandChanged(String),
    SettingsDefaultPickerModeChanged(PickerMode),
    ConfirmSettings,
    ConfigSaved(Result<(), String>),

    // === Export/Import ===
    ExportSnippetsJson,
    ExportFinished(Result<String, String>),
    ImportSnippetsJson,
    ImportFinished(Result<String, String>),

    // === Add Menu Dropdown ===
    ToggleAddMenu,
    CloseAddMenu,

    // === Settings - Keyboard Shortcuts ===
    StartRecordingShortcut(ShortcutAction),
    StopRecordingShortcut,
    ShortcutRecorded(ShortcutAction, Shortcut),
    ResetShortcutToDefault(ShortcutAction),

    // === Palettes ===
    PalettesLoaded(Result<Vec<Palette>, String>),
    FilterPaletteChanged(Option<i64>),
    OpenPaletteManager,
    ClosePaletteManager,
    NewPaletteNameChanged(String),
    CreatePalette(String),
    PaletteCreated(Result<Palette, String>),
    RenamePalette(i64, String),
    PaletteRenamed(Result<Palette, String>),
    DeletePalette(i64),
    PaletteDeleted(Result<i64, String>),
    AddSnippetToPalette(i64, i64),      // (snippet_id, palette_id)
    RemoveSnippetFromPalette(i64, i64), // (snippet_id, palette_id)
    SnippetPaletteUpdated(Result<(), String>),
    TogglePaletteDropdown(Option<i64>), // snippet_id to show dropdown for
}

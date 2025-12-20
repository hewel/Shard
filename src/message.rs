//! Application messages for the Shard color palette manager.

use crate::color::Color;
use crate::view::PickerMode;

/// All messages that can be sent in the application.
#[derive(Debug, Clone)]
pub enum Message {
    // Initialization
    ColorsLoaded(Result<Vec<Color>, String>),

    // Color input
    ColorInputChanged(String),
    AddColor,
    ColorAdded(Result<Color, String>),

    // Color actions
    CopyHex(i64),
    CopyRgb(i64),
    CopyHsl(i64),
    CopyFinished(Result<String, String>),
    DeleteColor(i64),
    ColorDeleted(Result<i64, String>),

    // Label editing
    StartEditLabel(i64),
    EditLabelChanged(String),
    SaveLabel,
    CancelEditLabel,
    LabelSaved(Result<(i64, String), String>),

    // Clipboard listening
    ToggleClipboard(bool),
    ClipboardTick,
    ClipboardContentReceived(Option<String>),

    // Search/Filter
    FilterChanged(String),

    // Keyboard shortcuts
    PasteFromClipboard,
    FocusColorInput,
    EscapePressed,
    DeleteSelectedColor,
    SelectColor(Option<i64>),

    // Color picker
    OpenColorPicker(Option<i64>), // None = new color, Some(id) = edit existing
    CloseColorPicker,
    PickerModeChanged(PickerMode),
    PickerHueChanged(f32),
    PickerSaturationChanged(f32),
    PickerLightnessChanged(f32),
    PickerSLChanged(f32, f32), // Combined saturation + lightness from SL box drag
    PickerAlphaChanged(f32),
    PickerLabelChanged(String),
    // OKLCH mode messages
    PickerOklchLChanged(f32),
    PickerOklchCChanged(f32),
    PickerOklchHChanged(f32),
    PickerCLChanged(f32, f32), // Combined chroma + lightness from CL box drag
    ConfirmColorPicker,
    ColorUpdated(Result<Color, String>),
}

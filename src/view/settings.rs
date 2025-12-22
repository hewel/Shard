//! Settings modal for application configuration.

use iced::widget::{
    button, column, container, mouse_area, opaque, row, scrollable, text, text_input,
};
use iced::{Element, Length};

use crate::config::{Config, EditorPreset, KeyboardConfig, ShortcutAction};
use crate::icons;
use crate::message::Message;
use crate::theme::{
    input_style, modal_dialog_style, modal_overlay_style, primary_button_style, scrollbar_style,
    secondary_button_style, subtle_button_style, SPACE_MD, SPACE_SM, SPACE_XS, TEXT_MUTED,
    TEXT_PRIMARY, TEXT_SECONDARY,
};
use crate::view::PickerMode;

/// State for the settings modal.
#[derive(Debug, Clone)]
pub struct SettingsState {
    /// Current editor preset selection.
    pub editor_preset: EditorPreset,
    /// Custom command (used when preset is Custom).
    pub custom_command: String,
    /// Keyboard shortcuts configuration.
    pub keyboard: KeyboardConfig,
    /// Which shortcut action is currently being recorded (if any).
    pub recording_action: Option<ShortcutAction>,
    /// Default color picker mode.
    pub default_picker_mode: PickerMode,
}

impl SettingsState {
    /// Create settings state from current config.
    pub fn from_config(config: &Config) -> Self {
        Self {
            editor_preset: config.editor.preset,
            custom_command: config.editor.custom_command.clone(),
            keyboard: config.keyboard.clone(),
            recording_action: None,
            default_picker_mode: config.default_picker_mode,
        }
    }

    /// Apply settings to config.
    pub fn apply_to_config(&self, config: &mut Config) {
        config.editor.preset = self.editor_preset;
        config.editor.custom_command = self.custom_command.clone();
        config.keyboard = self.keyboard.clone();
        config.default_picker_mode = self.default_picker_mode;
    }
}

/// All available editor presets for selection.
const EDITOR_PRESETS: [EditorPreset; 5] = [
    EditorPreset::Vscode,
    EditorPreset::Helix,
    EditorPreset::Neovim,
    EditorPreset::Vim,
    EditorPreset::Custom,
];

/// Render a single shortcut row.
fn view_shortcut_row<'a>(
    action: ShortcutAction,
    settings: &'a SettingsState,
) -> Element<'a, Message> {
    let shortcut = settings.keyboard.get(action);
    let is_recording = settings.recording_action == Some(action);

    let label = text(action.display_name())
        .size(12)
        .color(TEXT_PRIMARY)
        .width(Length::Fixed(140.0));

    let shortcut_display = if is_recording {
        text("Press keys...")
            .size(12)
            .color(TEXT_MUTED)
            .width(Length::Fixed(100.0))
    } else {
        text(shortcut.to_string())
            .size(12)
            .color(TEXT_SECONDARY)
            .width(Length::Fixed(100.0))
    };

    let record_button = if is_recording {
        button(text("Cancel").size(11))
            .on_press(Message::StopRecordingShortcut)
            .padding([SPACE_XS, SPACE_SM])
            .style(secondary_button_style)
    } else {
        button(text("Record").size(11))
            .on_press(Message::StartRecordingShortcut(action))
            .padding([SPACE_XS, SPACE_SM])
            .style(secondary_button_style)
    };

    let reset_button = button(text("Reset").size(11))
        .on_press(Message::ResetShortcutToDefault(action))
        .padding([SPACE_XS, SPACE_SM])
        .style(subtle_button_style);

    row![label, shortcut_display, record_button, reset_button]
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center)
        .into()
}

/// Render the settings modal.
pub fn view_settings_modal(settings: &SettingsState) -> Element<'_, Message> {
    // Header
    let header_row = row![
        text("Settings").size(20).color(TEXT_PRIMARY),
        iced::widget::Space::new().width(Length::Fill),
        button(icons::x().size(16))
            .on_press(Message::CloseSettings)
            .padding([SPACE_XS, SPACE_SM])
            .style(subtle_button_style),
    ]
    .padding(iced::Padding::new(SPACE_XS).vertical(SPACE_SM).left(SPACE_MD))
    .align_y(iced::Alignment::Center);

    // Editor section title
    let editor_section_title = text("External Editor").size(14).color(TEXT_SECONDARY);

    // Editor preset selection - radio-like buttons
    let preset_buttons: Vec<Element<'_, Message>> = EDITOR_PRESETS
        .iter()
        .map(|preset| {
            let is_selected = settings.editor_preset == *preset;
            let style = if is_selected {
                primary_button_style
            } else {
                secondary_button_style
            };

            button(text(preset.display_name()).size(12))
                .on_press(Message::SettingsEditorPresetChanged(*preset))
                .padding([SPACE_XS, SPACE_SM])
                .style(style)
                .into()
        })
        .collect();

    let preset_row = row(preset_buttons)
        .spacing(SPACE_XS)
        .align_y(iced::Alignment::Center);

    // Show the effective command
    let effective_command = if settings.editor_preset == EditorPreset::Custom {
        if settings.custom_command.is_empty() {
            "(no command set)".to_string()
        } else {
            settings.custom_command.clone()
        }
    } else {
        settings
            .editor_preset
            .default_command()
            .unwrap_or("")
            .to_string()
    };

    let command_preview = text(format!("Command: {}", effective_command))
        .size(11)
        .color(TEXT_MUTED);

    // Custom command input (only visible when Custom is selected)
    let custom_command_section: Element<'_, Message> =
        if settings.editor_preset == EditorPreset::Custom {
            column![
                row![
                    text("Custom Command:").size(12).color(TEXT_SECONDARY),
                    text_input("e.g., subl -w {file}", &settings.custom_command)
                        .on_input(Message::SettingsCustomCommandChanged)
                        .padding(SPACE_SM)
                        .width(Length::Fill)
                        .style(|theme, status| input_style(theme, status, false)),
                ]
                .spacing(SPACE_SM)
                .align_y(iced::Alignment::Center),
                text("Use {file} as placeholder for the file path")
                    .size(11)
                    .color(TEXT_MUTED),
            ]
            .spacing(SPACE_XS)
            .into()
        } else {
            container(text("")).into()
        };

    // Color Picker section
    let picker_section_title = text("Color Picker").size(14).color(TEXT_SECONDARY);

    let picker_mode_buttons = row![
        button(text("HSL").size(12))
            .on_press(Message::SettingsDefaultPickerModeChanged(PickerMode::Hsl))
            .padding([SPACE_XS, SPACE_SM])
            .style(if settings.default_picker_mode == PickerMode::Hsl {
                primary_button_style
            } else {
                secondary_button_style
            }),
        button(text("OKLCH").size(12))
            .on_press(Message::SettingsDefaultPickerModeChanged(PickerMode::Oklch))
            .padding([SPACE_XS, SPACE_SM])
            .style(if settings.default_picker_mode == PickerMode::Oklch {
                primary_button_style
            } else {
                secondary_button_style
            }),
    ]
    .spacing(SPACE_XS);

    let picker_hint = text("Default color space when opening the color picker")
        .size(11)
        .color(TEXT_MUTED);

    // Data section - Export/Import
    let data_section_title = text("Data").size(14).color(TEXT_SECONDARY);

    let export_button = button(text("Export as JSON").size(12))
        .on_press(Message::ExportSnippetsJson)
        .padding([SPACE_XS, SPACE_SM])
        .style(secondary_button_style);

    let import_button = button(text("Import from JSON").size(12))
        .on_press(Message::ImportSnippetsJson)
        .padding([SPACE_XS, SPACE_SM])
        .style(secondary_button_style);

    let data_buttons = row![export_button, import_button].spacing(SPACE_SM);

    // Keyboard shortcuts section
    let keyboard_section_title = text("Keyboard Shortcuts").size(14).color(TEXT_SECONDARY);

    let shortcut_rows: Vec<Element<'_, Message>> = ShortcutAction::ALL
        .iter()
        .map(|action| view_shortcut_row(*action, settings))
        .collect();

    let keyboard_section = column(shortcut_rows).spacing(SPACE_XS);

    let recording_hint: Element<'_, Message> = if settings.recording_action.is_some() {
        text("Press any key combination to assign...")
            .size(11)
            .color(TEXT_MUTED)
            .into()
    } else {
        container(text("")).into()
    };

    // Action buttons
    let action_buttons = row![
        iced::widget::Space::new().width(Length::Fill),
        button(text("Cancel").size(14))
            .on_press(Message::CloseSettings)
            .padding(SPACE_SM)
            .style(secondary_button_style),
        button(text("Save").size(14))
            .on_press(Message::ConfirmSettings)
            .padding(SPACE_SM)
            .style(primary_button_style),
    ]
    .spacing(SPACE_SM)
    .padding([SPACE_SM, SPACE_MD]);

    // Scrollable content (everything between header and action buttons)
    let scrollable_content = scrollable(
        column![
            editor_section_title,
            preset_row,
            command_preview,
            custom_command_section,
            iced::widget::Space::new().height(Length::Fixed(SPACE_SM)),
            picker_section_title,
            picker_mode_buttons,
            picker_hint,
            iced::widget::Space::new().height(Length::Fixed(SPACE_SM)),
            keyboard_section_title,
            keyboard_section,
            recording_hint,
            iced::widget::Space::new().height(Length::Fixed(SPACE_SM)),
            data_section_title,
            data_buttons,
        ]
        .spacing(SPACE_MD)
        .padding(SPACE_MD),
    )
    .height(Length::FillPortion(1))
    .width(Length::Fill)
    .style(scrollbar_style);

    // Modal content with fixed header and footer, scrollable middle
    let modal_content = column![
        header_row,
        scrollable_content,
        iced::widget::Space::new().height(Length::Fixed(SPACE_SM)),
        action_buttons,
    ]
    .spacing(SPACE_MD)
    .width(Length::Fixed(500.0));

    let modal_dialog = container(modal_content)
        .max_height(600.0)
        .style(modal_dialog_style);

    // Semi-transparent overlay
    let overlay = mouse_area(
        container(opaque(modal_dialog))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(modal_overlay_style),
    )
    .on_press(Message::CloseSettings);

    overlay.into()
}

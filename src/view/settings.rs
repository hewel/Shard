//! Settings modal for application configuration.

use iced::widget::{button, column, container, mouse_area, opaque, row, text, text_input};
use iced::{Element, Length};

use crate::config::{Config, EditorPreset};
use crate::icons;
use crate::message::Message;
use crate::theme::{
    input_style, modal_dialog_style, modal_overlay_style, primary_button_style,
    secondary_button_style, subtle_button_style, SPACE_MD, SPACE_SM, SPACE_XS, TEXT_MUTED,
    TEXT_PRIMARY, TEXT_SECONDARY,
};

/// State for the settings modal.
#[derive(Debug, Clone)]
pub struct SettingsState {
    /// Current editor preset selection.
    pub editor_preset: EditorPreset,
    /// Custom command (used when preset is Custom).
    pub custom_command: String,
}

impl SettingsState {
    /// Create settings state from current config.
    pub fn from_config(config: &Config) -> Self {
        Self {
            editor_preset: config.editor.preset,
            custom_command: config.editor.custom_command.clone(),
        }
    }

    /// Apply settings to config.
    pub fn apply_to_config(&self, config: &mut Config) {
        config.editor.preset = self.editor_preset;
        config.editor.custom_command = self.custom_command.clone();
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

    // Action buttons
    let action_buttons = row![
        button(text("Cancel").size(14))
            .on_press(Message::CloseSettings)
            .padding(SPACE_SM)
            .style(secondary_button_style),
        button(text("Save").size(14))
            .on_press(Message::ConfirmSettings)
            .padding(SPACE_SM)
            .style(primary_button_style),
    ]
    .spacing(SPACE_SM);

    // Modal content
    let modal_content = column![
        header_row,
        editor_section_title,
        preset_row,
        command_preview,
        custom_command_section,
        iced::widget::Space::new().height(Length::Fixed(SPACE_MD)),
        action_buttons,
    ]
    .spacing(SPACE_MD)
    .padding(SPACE_MD)
    .width(Length::Fixed(450.0));

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
    .on_press(Message::CloseSettings);

    overlay.into()
}

//! View module containing UI components.

pub mod code_card;
pub mod code_editor;
pub mod color_card;
pub mod color_picker;
pub mod settings;
pub mod text_card;
pub mod text_editor;

pub use code_card::view_code_card;
pub use code_editor::CodeEditorState;
pub use color_card::view_color_card;
pub use color_picker::{view_color_picker_modal, ColorPickerState, PickerMode};
pub use settings::SettingsState;
pub use text_card::view_text_card;
pub use text_editor::TextEditorState;

use iced::widget::{
    button, checkbox, column, container, mouse_area, row, scrollable, stack, text, text_input,
};
use iced::{Element, Length};

use std::collections::HashMap;

use crate::db::Palette;
use crate::icons;
use crate::message::Message;
use crate::snippet::{Snippet, SnippetContent, SnippetKind};
use crate::theme::{
    dropdown_item_style, dropdown_menu_style, header_style, input_style, primary_button_style,
    scrollbar_style, secondary_button_style, status_bar_style, subtle_button_style, BG_BASE,
    SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XS, TEXT_MUTED, TEXT_SECONDARY,
};

/// Context for rendering the main view.
pub struct ViewContext<'a> {
    pub snippets: &'a [Snippet],
    pub is_listening_clipboard: bool,
    pub status_message: Option<&'a str>,
    pub filter_text: &'a str,
    pub filter_kind: Option<&'a SnippetKind>,
    pub selected_snippet: Option<i64>,
    pub color_picker: Option<&'a ColorPickerState>,
    pub code_editor: Option<&'a CodeEditorState>,
    pub text_editor: Option<&'a TextEditorState>,
    pub settings: Option<&'a SettingsState>,
    pub add_menu_open: bool,
    // Palette fields
    pub palettes: &'a [Palette],
    pub filter_palette: Option<i64>,
    pub palette_manager_open: bool,
    pub palette_dropdown_snippet: Option<i64>,
    pub snippet_palettes: &'a HashMap<i64, Vec<i64>>,
    pub new_palette_name: &'a str,
}

/// Render the main application view.
pub fn view(ctx: ViewContext<'_>) -> Element<'_, Message> {
    let ViewContext {
        snippets,
        is_listening_clipboard,
        status_message,
        filter_text,
        filter_kind,
        selected_snippet,
        color_picker,
        code_editor,
        text_editor,
        settings,
        add_menu_open,
        palettes,
        filter_palette,
        palette_manager_open,
        palette_dropdown_snippet,
        snippet_palettes,
        new_palette_name,
    } = ctx;

    // Add button (plus icon) that toggles dropdown
    let add_button = button(icons::plus().size(12))
        .on_press(Message::ToggleAddMenu)
        .padding([SPACE_XS, SPACE_SM])
        .style(primary_button_style);

    // === HEADER: Tabs + Filter + Clipboard + Settings ===

    // Tab filter buttons
    let tab_row = row![
        tab_button(
            "All",
            filter_kind.is_none(),
            Message::FilterKindChanged(None)
        ),
        tab_button(
            "Colors",
            filter_kind == Some(&SnippetKind::Color),
            Message::FilterKindChanged(Some(SnippetKind::Color))
        ),
        tab_button(
            "Code",
            filter_kind == Some(&SnippetKind::Code),
            Message::FilterKindChanged(Some(SnippetKind::Code))
        ),
        tab_button(
            "Text",
            filter_kind == Some(&SnippetKind::Text),
            Message::FilterKindChanged(Some(SnippetKind::Text))
        ),
    ]
    .spacing(SPACE_XS);

    // Palette filter dropdown
    let palette_filter = view_palette_filter(palettes, filter_palette);

    // Filter input
    let filter_input = text_input("Search...", filter_text)
        .on_input(Message::FilterChanged)
        .width(Length::Fixed(160.0))
        .padding([SPACE_XS, SPACE_SM])
        .size(13)
        .style(|theme, status| input_style(theme, status, false));

    // Clipboard toggle
    let clipboard_toggle = container(
        row![
            checkbox(is_listening_clipboard).on_toggle(Message::ToggleClipboard),
            text("Auto-capture").size(12).color(TEXT_MUTED),
        ]
        .spacing(SPACE_XS)
        .align_y(iced::Alignment::Center),
    )
    .padding([SPACE_XS, SPACE_SM]);

    // Settings button
    let settings_button = button(icons::gear().size(16))
        .on_press(Message::OpenSettings)
        .padding([SPACE_XS, SPACE_SM])
        .style(subtle_button_style);

    // Spacer to push right-side elements
    let spacer = container(text("")).width(Length::Fill);

    let header_row = row![
        tab_row,
        palette_filter,
        spacer,
        filter_input,
        clipboard_toggle,
        settings_button,
        add_button,
    ]
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center);

    // Header container
    let header = container(header_row)
        .width(Length::Fill)
        .padding([SPACE_MD, SPACE_MD])
        .style(header_style);

    // Filter snippets
    let filtered_snippets: Vec<&Snippet> = snippets
        .iter()
        .filter(|s| {
            // Filter by kind
            if let Some(kind) = filter_kind {
                if &s.kind() != kind {
                    return false;
                }
            }
            // Filter by palette
            if let Some(palette_id) = filter_palette {
                if let Some(snippet_palette_ids) = snippet_palettes.get(&s.id) {
                    if !snippet_palette_ids.contains(&palette_id) {
                        return false;
                    }
                } else {
                    return false; // Snippet not in any palette
                }
            }
            // Filter by text
            if !filter_text.trim().is_empty() {
                return s.matches_filter(filter_text);
            }
            true
        })
        .collect();

    // Snippet list
    let snippets_list: Element<'_, Message> = if snippets.is_empty() {
        container(
            text("No snippets yet. Add a color, code, or text snippet above.")
                .size(14)
                .color(TEXT_SECONDARY),
        )
        .padding(SPACE_MD)
        .center_x(Length::Fill)
        .into()
    } else if filtered_snippets.is_empty() {
        container(
            text(format!("No snippets match '{}'", filter_text))
                .size(14)
                .color(TEXT_SECONDARY),
        )
        .padding(SPACE_MD)
        .center_x(Length::Fill)
        .into()
    } else {
        let items: Vec<Element<'_, Message>> = filtered_snippets
            .iter()
            .map(|snippet| {
                let is_selected = selected_snippet == Some(snippet.id);
                view_snippet_card(snippet, is_selected)
            })
            .collect();

        scrollable(column(items).spacing(SPACE_SM).padding(SPACE_MD))
            .height(Length::Fill)
            .style(scrollbar_style)
            .into()
    };

    // Status bar
    let status_text = status_message.unwrap_or("Ready");
    let count_text =
        if filter_text.trim().is_empty() && filter_kind.is_none() && filter_palette.is_none() {
            format!("{} snippets", snippets.len())
        } else {
            format!("{} / {} snippets", filtered_snippets.len(), snippets.len())
        };
    let status_bar_content = row![
        text(count_text).size(12).color(TEXT_SECONDARY),
        text("|").size(12).color(TEXT_SECONDARY),
        text(status_text).size(12).color(TEXT_SECONDARY),
    ]
    .spacing(SPACE_SM)
    .padding(SPACE_SM);

    let status_bar = container(status_bar_content)
        .width(Length::Fill)
        .style(status_bar_style);

    // Main layout
    let main_content = container(column![header, snippets_list, status_bar])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| iced::widget::container::Style::default().background(BG_BASE));

    // Build overlay layer (always present to maintain consistent widget tree)
    let overlay: Element<'_, Message> = if let Some(s) = settings {
        settings::view_settings_modal(s)
    } else if palette_manager_open {
        view_palette_manager_modal(palettes, new_palette_name)
    } else if let Some(picker) = color_picker {
        view_color_picker_modal(picker)
    } else if let Some(editor) = code_editor {
        code_editor::view_code_editor_modal(editor)
    } else if let Some(editor) = text_editor {
        text_editor::view_text_editor_modal(editor)
    } else if add_menu_open {
        view_add_menu_dropdown()
    } else if palette_dropdown_snippet.is_some() {
        // Palette assignment dropdown (shown over snippet card)
        view_palette_assignment_dropdown(
            palettes,
            palette_dropdown_snippet.unwrap(),
            snippet_palettes,
        )
    } else {
        // Empty overlay - preserves widget tree structure
        container(text("")).width(0).height(0).into()
    };

    stack![main_content, overlay].into()
}

/// Render a tab filter button.
fn tab_button(label: &str, is_active: bool, on_press: Message) -> Element<'_, Message> {
    button(text(label).size(12))
        .on_press(on_press)
        .padding([SPACE_XS, SPACE_SM])
        .style(if is_active {
            primary_button_style
        } else {
            secondary_button_style
        })
        .into()
}

/// Render a snippet card based on its type.
fn view_snippet_card(snippet: &Snippet, is_selected: bool) -> Element<'_, Message> {
    match &snippet.content {
        SnippetContent::Color(color) => {
            view_color_card(snippet.id, &snippet.label, color, is_selected)
        }
        SnippetContent::Code(code) => view_code_card(snippet.id, &snippet.label, code, is_selected),
        SnippetContent::Text(text_data) => {
            view_text_card(snippet.id, &snippet.label, text_data, is_selected)
        }
    }
}

/// Render the add menu dropdown overlay.
fn view_add_menu_dropdown() -> Element<'static, Message> {
    // Dropdown menu items
    let color_item = button(
        row![icons::palette().size(14), text("Color").size(13)]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
    )
    .on_press(Message::OpenColorPicker(None))
    .padding([SPACE_SM, SPACE_MD])
    .width(Length::Fill)
    .style(dropdown_item_style);

    let code_item = button(
        row![icons::code().size(14), text("Code").size(13)]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
    )
    .on_press(Message::OpenCodeEditor(None))
    .padding([SPACE_SM, SPACE_MD])
    .width(Length::Fill)
    .style(dropdown_item_style);

    let text_item = button(
        row![icons::text_icon().size(14), text("Text").size(13)]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
    )
    .on_press(Message::OpenTextEditor(None))
    .padding([SPACE_SM, SPACE_MD])
    .width(Length::Fill)
    .style(dropdown_item_style);

    // Menu container
    let menu = container(column![color_item, code_item, text_item].spacing(2))
        .padding(SPACE_XS)
        .width(Length::Fixed(140.0))
        .style(dropdown_menu_style);

    // Position menu at top-right using container alignment
    let positioned_menu = container(menu)
        .width(Length::Fill)
        .padding(iced::Padding::new(0.0).top(52.0).right(SPACE_MD))
        .align_x(iced::alignment::Horizontal::Right);

    // Click-outside-to-close overlay (transparent, minimal overhead)
    mouse_area(positioned_menu)
        .on_press(Message::CloseAddMenu)
        .into()
}

/// Render the palette filter dropdown in header.
fn view_palette_filter<'a>(
    palettes: &'a [Palette],
    filter_palette: Option<i64>,
) -> Element<'a, Message> {
    // Create palette selection buttons
    let all_btn = button(text("All Palettes").size(12))
        .on_press(Message::FilterPaletteChanged(None))
        .padding([SPACE_XS, SPACE_SM])
        .style(if filter_palette.is_none() {
            primary_button_style
        } else {
            secondary_button_style
        });

    let palette_buttons: Vec<Element<'a, Message>> = palettes
        .iter()
        .map(|p| {
            let is_selected = filter_palette == Some(p.id);
            button(text(&p.name).size(12))
                .on_press(Message::FilterPaletteChanged(Some(p.id)))
                .padding([SPACE_XS, SPACE_SM])
                .style(if is_selected {
                    primary_button_style
                } else {
                    secondary_button_style
                })
                .into()
        })
        .collect();

    // Manage palettes button
    let manage_btn = button(icons::tag().size(12))
        .on_press(Message::OpenPaletteManager)
        .padding([SPACE_XS, SPACE_SM])
        .style(subtle_button_style);

    let mut items: Vec<Element<'a, Message>> = vec![all_btn.into()];
    items.extend(palette_buttons);
    items.push(manage_btn.into());

    row(items).spacing(SPACE_XS).into()
}

/// Render the palette manager modal.
fn view_palette_manager_modal<'a>(
    palettes: &'a [Palette],
    new_palette_name: &'a str,
) -> Element<'a, Message> {
    use crate::theme::{modal_dialog_style, modal_overlay_style};

    // Title
    let title = text("Manage Palettes").size(18);

    // New palette input
    let new_input = text_input("New palette name...", new_palette_name)
        .on_input(Message::NewPaletteNameChanged)
        .on_submit(Message::CreatePalette(new_palette_name.to_string()))
        .padding(SPACE_SM)
        .width(Length::Fill)
        .style(|theme, status| input_style(theme, status, false));

    let create_btn = button(text("Create").size(13))
        .on_press(Message::CreatePalette(new_palette_name.to_string()))
        .padding([SPACE_SM, SPACE_MD])
        .style(primary_button_style);

    let new_row = row![new_input, create_btn]
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center);

    // Palette list
    let palette_items: Vec<Element<'a, Message>> = palettes
        .iter()
        .map(|p| {
            let delete_btn = button(icons::trash().size(14))
                .on_press(Message::DeletePalette(p.id))
                .padding(SPACE_XS)
                .style(subtle_button_style);

            container(
                row![text(&p.name).size(14).width(Length::Fill), delete_btn,]
                    .spacing(SPACE_SM)
                    .align_y(iced::Alignment::Center),
            )
            .padding(SPACE_SM)
            .style(|_theme| {
                iced::widget::container::Style::default()
                    .background(crate::theme::BG_ELEVATED)
                    .border(iced::Border::default().rounded(4.0))
            })
            .into()
        })
        .collect();

    let palette_list: Element<'a, Message> = if palette_items.is_empty() {
        container(
            text("No palettes yet. Create one above.")
                .size(13)
                .color(TEXT_MUTED),
        )
        .padding(SPACE_MD)
        .center_x(Length::Fill)
        .into()
    } else {
        scrollable(column(palette_items).spacing(SPACE_XS))
            .height(Length::Fixed(200.0))
            .style(scrollbar_style)
            .into()
    };

    // Close button
    let close_btn = button(text("Close").size(13))
        .on_press(Message::ClosePaletteManager)
        .padding([SPACE_SM, SPACE_MD])
        .style(secondary_button_style);

    // Modal content
    let modal_content = container(
        column![title, new_row, palette_list, close_btn]
            .spacing(SPACE_MD)
            .align_x(iced::Alignment::End),
    )
    .padding(SPACE_LG)
    .width(Length::Fixed(400.0))
    .style(modal_dialog_style);

    // Center modal
    let centered = container(modal_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

    // Backdrop
    mouse_area(
        container(centered)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(modal_overlay_style),
    )
    .on_press(Message::ClosePaletteManager)
    .into()
}

/// Render palette assignment dropdown overlay.
fn view_palette_assignment_dropdown<'a>(
    palettes: &'a [Palette],
    snippet_id: i64,
    snippet_palettes: &'a HashMap<i64, Vec<i64>>,
) -> Element<'a, Message> {
    let snippet_palette_ids = snippet_palettes
        .get(&snippet_id)
        .cloned()
        .unwrap_or_default();

    let items: Vec<Element<'a, Message>> = palettes
        .iter()
        .map(|p| {
            let is_in_palette = snippet_palette_ids.contains(&p.id);
            let msg = if is_in_palette {
                Message::RemoveSnippetFromPalette(snippet_id, p.id)
            } else {
                Message::AddSnippetToPalette(snippet_id, p.id)
            };
            let msg_for_checkbox = msg.clone();

            button(
                row![
                    checkbox(is_in_palette).on_toggle(move |_| msg_for_checkbox.clone()),
                    text(&p.name).size(13),
                ]
                .spacing(SPACE_SM)
                .align_y(iced::Alignment::Center),
            )
            .on_press(msg)
            .padding([SPACE_SM, SPACE_MD])
            .width(Length::Fill)
            .style(dropdown_item_style)
            .into()
        })
        .collect();

    let menu_content: Element<'a, Message> = if items.is_empty() {
        container(
            text("No palettes. Create one in settings.")
                .size(12)
                .color(TEXT_MUTED),
        )
        .padding(SPACE_SM)
        .into()
    } else {
        column(items).spacing(2).into()
    };

    let menu = container(menu_content)
        .padding(SPACE_XS)
        .width(Length::Fixed(180.0))
        .style(dropdown_menu_style);

    // Position dropdown (center of screen for simplicity)
    let positioned = container(menu)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

    mouse_area(positioned)
        .on_press(Message::TogglePaletteDropdown(None))
        .into()
}

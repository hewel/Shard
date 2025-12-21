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
    button, checkbox, column, container, mouse_area, row, scrollable, stack, text, text_input, Id,
};
use iced::{Element, Length};

use crate::icons;
use crate::message::Message;
use crate::snippet::{Snippet, SnippetContent, SnippetKind};
use crate::theme::{
    dropdown_item_style, dropdown_menu_style, header_style, input_style, primary_button_style,
    scrollbar_style, secondary_button_style, status_bar_style, subtle_button_style, BG_BASE,
    SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XS, TEXT_MUTED, TEXT_SECONDARY,
};

/// Input field ID for keyboard focus.
pub const COLOR_INPUT_ID: &str = "color_input";

/// Context for rendering the main view.
pub struct ViewContext<'a> {
    pub snippets: &'a [Snippet],
    pub color_input: &'a str,
    pub input_error: Option<&'a str>,
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
}

/// Render the main application view.
pub fn view(ctx: ViewContext<'_>) -> Element<'_, Message> {
    let ViewContext {
        snippets,
        color_input,
        input_error,
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
    } = ctx;

    let has_error = input_error.is_some();

    // === HEADER ROW 1: Color input + Add dropdown ===
    let color_input_widget = text_input("Enter color (hex, rgb, hsl, oklch)...", color_input)
        .id(Id::from(COLOR_INPUT_ID))
        .on_input(Message::ColorInputChanged)
        .on_submit(Message::AddColorFromInput)
        .width(Length::Fill)
        .padding(SPACE_SM)
        .style(move |theme, status| input_style(theme, status, has_error));

    // Add button (plus icon) that toggles dropdown
    let add_button = button(icons::plus().size(16))
        .on_press(Message::ToggleAddMenu)
        .padding([SPACE_SM, SPACE_SM])
        .style(primary_button_style);

    let header_row_1 = row![color_input_widget, add_button]
        .spacing(SPACE_MD)
        .align_y(iced::Alignment::Center);

    // === HEADER ROW 2: Tabs + Filter + Clipboard + Settings ===

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

    let header_row_2 = row![
        tab_row,
        spacer,
        filter_input,
        clipboard_toggle,
        settings_button,
    ]
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center);

    // Combined header
    let header = container(column![header_row_1, header_row_2].spacing(SPACE_SM))
        .width(Length::Fill)
        .padding([SPACE_MD, SPACE_MD])
        .style(header_style);

    // Error message
    let error_text: Element<'_, Message> = if let Some(error) = input_error {
        container(text(error).size(12).color(crate::theme::DANGER))
            .padding(
                iced::Padding::new(0.0)
                    .left(SPACE_LG)
                    .right(SPACE_LG)
                    .bottom(SPACE_SM),
            )
            .into()
    } else {
        container(text("")).into()
    };

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
    let count_text = if filter_text.trim().is_empty() && filter_kind.is_none() {
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
    let main_content = container(column![header, error_text, snippets_list, status_bar])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| iced::widget::container::Style::default().background(BG_BASE));

    // Build overlay layer (always present to maintain consistent widget tree)
    let overlay: Element<'_, Message> = if let Some(s) = settings {
        settings::view_settings_modal(s)
    } else if let Some(picker) = color_picker {
        view_color_picker_modal(picker)
    } else if let Some(editor) = code_editor {
        code_editor::view_code_editor_modal(editor)
    } else if let Some(editor) = text_editor {
        text_editor::view_text_editor_modal(editor)
    } else if add_menu_open {
        view_add_menu_dropdown()
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

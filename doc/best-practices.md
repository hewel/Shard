Iced offers robust capabilities for building text-handling applications, leveraging an Elm-inspired architecture for clear separation of concerns. This document outlines best practices for creating a text handling app using Iced.

**1. Application Structure and Core Principles**

Iced applications follow the Elm architecture, which involves:
*   **State:** The current data of your application.
*   **Messages:** Events triggered by user interactions or other meaningful occurrences.
*   **View Logic:** How your state is displayed as widgets, potentially producing messages.
*   **Update Logic:** How the application reacts to messages and modifies the state.

This pattern is fundamental for building reactive user interfaces, ensuring that the UI consistently reflects the application's state.

**2. Handling Text Input and Display**

Iced provides several components for text management:

*   **[`iced::widget::text_editor`](%2Ficed-rs%2Ficed%2Fwidget%2Fsrc%2Fhelpers.rs#L1434) for Interactive Text Input:**
    *   For multi-line, editable text, use [`text_editor`](%2Ficed-rs%2Ficed%2Fwidget%2Fsrc%2Fhelpers.rs#L1460). It wraps [`cosmic_text::Editor`](%2Ficed-rs%2Ficed%2Fgraphics%2Fsrc%2Ftext%2Feditor.rs#L21) and provides functionalities for text management, selection, cursor control, and editing actions.
    *   Messages from [`text_editor`](%2Ficed-rs%2Ficed%2Fwidget%2Fsrc%2Fhelpers.rs#L1460) can be captured using the `.on_action()` method, which emits [`text_editor::Action`](%2Ficed-rs%2Ficed%2Ftester%2Fsrc%2Flib.rs#L159) variants for various user interactions like typing, pasting, or navigation. You can react to these in your [`update`](%2Ficed-rs%2Ficed%2Fsrc%2Fdaemon.rs#L81) function to modify your application's [`text_editor::Content`](%2Ficed-rs%2Ficed%2FCHANGELOG.md#L116) state.
    *   Example usage can be seen in `examples/editor/src/main.rs`.

*   **[`iced::widget::text`](%2Ficed-rs%2Ficed%2Fsrc%2Flib.rs#L37) for Static Text Display:**
    *   For displaying static text, use [`iced::widget::text`](%2Ficed-rs%2Ficed%2Fsrc%2Flib.rs#L37). It's suitable for labels, read-only content, or rich text when combined with [`rich_text`](%2Ficed-rs%2Ficed%2Fwidget%2Fsrc%2Fhelpers.rs#L1165).
    *   Text widgets can be customized with various properties like [`size`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Ftext.rs#L432), [`color`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Fsvg.rs#L46), [`font`](%2Ficed-rs%2Ficed%2Fsrc%2Fdaemon.rs#L177), [`align_x`](%2Ficed-rs%2Ficed%2Fwidget%2Fsrc%2Frow.rs#L365), [`align_y`](%2Ficed-rs%2Ficed%2Fwidget%2Fsrc%2Frow.rs#L115), [`shaping`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Ftext.rs#L143), and [`wrapping`](%2Ficed-rs%2Ficed%2Fwidget%2Fsrc%2Frow.rs#L351). These are encapsulated in a [`Format`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Fwidget%2Ftext.rs#L265) struct, as seen in `core/src/widget/text.rs`.

*   **`iced::core::text::Paragraph` and `iced::core::text::Editor` Traits:**
    *   These traits, defined in `core/src/text/paragraph.rs` and `core/src/text/editor.rs`, abstract the underlying text rendering and editing capabilities. The [`iced_graphics`](%2Ficed-rs%2Ficed%2FCargo.toml#L177) crate provides concrete implementations that integrate with [`cosmic_text`](%2Ficed-rs%2Ficed%2Fgraphics%2Fsrc%2Ftext.rs#L10) for efficient layout and font handling, as detailed in [Text Rendering and Management](#graphics-and-rendering-infrastructure-text-rendering-and-management).

**3. Styling and Theming Text**

*   **Themes:** Your application's default theme can be set using `iced::application().theme()`. This influences the default appearance of text and other widgets.
*   **Custom Styling:**
    *   Text widgets can be styled using the `.style()` method, which accepts a closure that returns a `widget::text::Style`. This allows for dynamic styling based on the application's theme or other criteria.
    *   Convenience functions like `text::base`, `text::primary`, `text::secondary`, [`text::success`](%2Ficed-rs%2Ficed%2Fexamples%2Ftable%2Fsrc%2Fmain.rs#L72), [`text::warning`](%2Ficed-rs%2Ficed%2Fexamples%2Ftable%2Fsrc%2Fmain.rs#L57), and [`text::danger`](%2Ficed-rs%2Ficed%2Fsrc%2Flib.rs#L278) are available for common text styles, as shown in `core/src/widget/text.rs`.

**4. Advanced Text Features**

*   **Syntax Highlighting:**
    *   Iced integrates with the [`syntect`](%2Ficed-rs%2Ficed%2FCargo.toml#L231) library for syntax highlighting. The [`iced::highlighter`](%2Ficed-rs%2Ficed%2Fexamples%2Feditor%2Fsrc%2Fmain.rs#L1) module provides [`Highlighter`](%2Ficed-rs%2Ficed%2Fwidget%2Fsrc%2Fmarkdown.rs#L437) for line-by-line highlighting and [`Stream`](%2Ficed-rs%2Ficed%2Fhighlighter%2Fsrc%2Flib.rs#L138) for real-time interactive highlighting.
    *   The [`Editor`](%2Ficed-rs%2Ficed%2Fwgpu%2Fsrc%2Flib.rs#L686) widget's `highlight()` method allows you to apply syntax highlighting based on a file extension or language token and a chosen theme, as demonstrated in `examples/editor/src/main.rs` and discussed in [Syntax Highlighting](#syntax-highlighting).
*   **Rich Text:**
    *   For text with mixed formatting (different fonts, colors, sizes, or even links), use `iced::widget::rich_text`. It accepts an iterator of [`span`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Ftext.rs#L388)s, where each [`Span`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Ftext.rs#L388) can have its own formatting properties like [`size`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Ftext.rs#L432), [`line_height`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Ftext.rs#L438), [`font`](%2Ficed-rs%2Ficed%2Fsrc%2Fdaemon.rs#L177), [`color`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Fsvg.rs#L46), [`link`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Ftext.rs#L468), [`highlight`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Ftext.rs#L415), [`padding`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Ftext.rs#L538), [`underline`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Ftext.rs#L544), and [`strikethrough`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Ftext.rs#L550), as documented in `core/src/text.rs`.
    *   The [`markdown`](%2Ficed-rs%2Ficed%2Fexamples%2Fmarkdown%2Fsrc%2Fmain.rs#L30) example in `examples/markdown/src/main.rs` showcases how [`rich_text`](%2Ficed-rs%2Ficed%2Fwidget%2Fsrc%2Fhelpers.rs#L1165) can be used to display formatted text with embedded images and links.
*   **Font Management:**
    *   Iced uses a global [`FontSystem`](%2Ficed-rs%2Ficed%2Fgraphics%2Fsrc%2Ftext.rs#L136) to load and manage fonts efficiently, ensuring they are loaded only once and shared across the application. New fonts can be loaded using `iced::application().font()`.
    *   The `iced_graphics::text` module provides functions to convert Iced's font types ([`Font`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Ffont.rs#L6), [`Weight`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Ffont.rs#L74), [`Stretch`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Ffont.rs#L90), [`Style`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Ftheme.rs#L219)) to [`cosmic_text`](%2Ficed-rs%2Ficed%2Fgraphics%2Fsrc%2Ftext.rs#L10) equivalents.

**5. Programmatic Control and Interaction**

*   **Widget Operations:** The `iced::widget::operation` module provides functions to programmatically control text input widgets.
    *   For [`text_input`](%2Ficed-rs%2Ficed%2Fselector%2Fsrc%2Ffind.rs#L203) widgets, you can use operations like [`move_cursor_to_end()`](%2Ficed-rs%2Ficed%2Fwidget%2Fsrc%2Ftext_input.rs#L1394), [`move_cursor_to_front()`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Fwidget%2Foperation%2Ftext_input.rs#L39), `move_cursor_to()`, [`select_all()`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Fwidget%2Foperation%2Ftext_input.rs#L114), and `select_range()` to manipulate the text and cursor, as defined in `core/src/widget/operation/text_input.rs` and re-exported by `runtime/src/widget/operation.rs`.
*   **Widget Selection:** The `iced::widget::selector` module (re-exporting from [`iced_selector`](%2Ficed-rs%2Ficed%2FCargo.toml#L182)) allows you to find specific UI elements within your widget tree.
    *   You can use selectors to find text inputs by their content ([`&str`](%2Ficed-rs%2Ficed%2Fwgpu%2Fsrc%2Flib.rs#L864) or [`String`](%2Ficed-rs%2Ficed%2Fcore%2Fsrc%2Ftext.rs#L635) selectors) or by their [`widget::Id`](%2Ficed-rs%2Ficed%2Fselector%2Fsrc%2Flib.rs#L99). This is useful for testing or building features that require dynamic interaction with UI elements, as explained in [UI Element Selection](#ui-element-selection).

**6. Asynchronous Operations**

*   **Tasks:** Long-running operations, such as loading large text files or processing text, should be offloaded to [`iced::Task`](%2Ficed-rs%2Ficed%2Fsrc%2Flib.rs#L296)s to keep the UI responsive.
    *   Tasks can be created from futures or streams and can send messages back to the [`update`](%2Ficed-rs%2Ficed%2Fsrc%2Fdaemon.rs#L81) function.
    *   The [`editor`](%2Ficed-rs%2Ficed%2Fwgpu%2Fsrc%2Flib.rs#L686) example in `examples/editor/src/main.rs` demonstrates using tasks for file operations like [`load_file`](%2Ficed-rs%2Ficed%2Fexamples%2Feditor%2Fsrc%2Fmain.rs#L261) and [`save_file`](%2Ficed-rs%2Ficed%2Fexamples%2Feditor%2Fsrc%2Fmain.rs#L272).

By following these best practices, you can build robust and feature-rich text-handling applications with Iced.

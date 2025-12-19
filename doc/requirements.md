# Product Requirements Document: Smart Text Processing & Snippet Manager

## 1. Project Overview

This project aims to build a desktop text processing tool using Rust and the `iced` GUI framework. The tool integrates a text editor with code highlighting, smart content recognition, regex-based batch processing, and a "pinned" text snippet management system. It is designed as a lightweight, efficient workbench for developers and writers.

## 2. Core Feature Requirements

### 2.1 Smart Editor

- **Basic Editing**: Provide a main editing area supporting multi-line text input, pasting, and modification.
- **Syntax Highlighting**:
- Automatically apply syntax highlighting when code content (e.g., Rust, JSON, Python) is detected.
- _Technical Requirement_: Utilize the `highlighter` component from the `iced` library.

- **Smart Content Recognition**:
- **Code Blocks**: Auto-detect and highlight.
- **Color Values**: If the text contains color codes (e.g., `#FF5733`, `rgb(255, 0, 0)`), visually display a preview of that color (e.g., a small color block in the sidebar or next to the line number).
- **Links/Paths**: (Optional) Identify URLs or file paths.

### 2.2 Pinned Snippets Management

- **Pinning Functionality**: Users can "pin" the current text in the main editor (or a specific selection) to a sidebar list with a single click.
- **Snippet List**:
- Display all pinned text snippets in a sidebar.
- Show a preview (first few lines) for each snippet.
- **Actions**:
- **Load**: Click to load the snippet content back into the main editor.
- **Delete**: Remove the snippet from the list.
- **Copy**: One-click copy to system clipboard.

### 2.3 Regex Batch Processing

- **Control Panel**: Provide input fields for "Regex Pattern" and "Replacement Text".
- **Processing Logic**:
- Support standard Regular Expression syntax (via Rust `regex` crate).
- Support "Replace All" operations.
- **Feedback**: Show "Live Preview" or error indicators (e.g., red text) if the regex syntax is invalid.

### 2.4 Listening & Monitoring

- **Clipboard Listener**:
- Toggle switch for "Listen to Clipboard". When enabled, if the system clipboard content changes and is text, it can automatically be added to the "Pinned List" or populate the editor (behavior configurable).

- **Input Monitoring**:
- Real-time monitoring of text changes within the editor to trigger "Smart Content Recognition" logic (e.g., typing `#` triggers color matching).

## 3. UI Layout Suggestions

Adopt a classic **Two-Column Layout**:

- **Top Bar**:
- Toolbar: Contains Regex Pattern input, Replacement input, and Execution buttons.
- Status Indicator: Shows current mode (e.g., "Regex Mode" or "Listening").

- **Left Side (Main Area - ~70%)**:
- Full-size Text Editor.
- Footer showing line/column numbers and detected content type (e.g., "Detected: Rust Code").

- **Right Side (Sidebar - ~30%)**:
- Header: "Pinned Snippets".
- Scrollable List: Displays snippets as cards.
- Card Elements: Text preview, Load button, Delete button.

## 4. Data Flow & Interaction Logic

1. **Text Input -> Recognition Engine**: User inputs text -> Triggers `update` -> Regex engine checks for color/code -> Updates `view` to show highlighting or color blocks.
2. **Regex Replace -> Editor**: User clicks "Replace" -> Reads current content -> Applies Regex -> Updates Editor content state.
3. **Pin Action -> State Management**: User clicks "Pin" -> Clones current `Content` -> Pushes to `Vec<Snippet>` -> Sidebar UI updates.

## 5. Technical Constraints

- **Language**: Rust
- **GUI Framework**: `iced` (Bleeding Edge / Master Branch version)
- **Crucial API Note**: Must use `Task` instead of `Command`.
- **Entry Point**: Must use the `iced::application` builder pattern.

- **Dependencies**:
- `iced` (features: `highlighter`, `tokio`, `advanced`)
- `regex` (for batch processing)
- `arboard` or `window_clipboard` (optional, for clipboard listening)

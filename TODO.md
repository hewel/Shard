# Shard Project Plan & Todo

## Phase 1: Project Initialization & Layout
- [x] **Setup Application Structure**
    - [x] Define `Shard` struct with core state fields.
    - [x] Implement `iced::application` entry point in `src/main.rs`.
    - [x] Create `Message` enum with variants for different domains (`Editor`, `Regex`, `Snippet`).
- [x] **Basic UI Layout**
    - [x] Implement `view` function with a Column.
    - [x] Create Top Toolbar (Row) for Regex controls.
    - [x] Create Main Content Area (Row) splitting Editor (Left) and Sidebar (Right).

## Phase 2: Core Editor Features
- [x] **Text Editor Implementation**
    - [x] Add `text_editor::Content` to `Shard` state.
    - [x] Integrate `iced::widget::text_editor` into the left column.
    - [x] Handle `EditorMessage::Edit` to update content.
- [x] **Syntax Highlighting**
    - [x] Configure `text_editor` with `iced::highlighter`.
    - [x] Pick a default theme (e.g., `Theme::Dark`).
    - [ ] **Improvement**: Auto-detect language (currently hardcoded to "rs").

## Phase 3: Regex Batch Processing
- [x] **Regex UI**
    - [x] Add text inputs for "Pattern" and "Replacement" in the Toolbar.
    - [x] Add "Replace All" button.
- [x] **Regex Logic**
    - [x] Implement `RegexMessage` handling.
    - [x] Create an async `Task` using `tokio` and `regex` crate to perform replacements without freezing UI.
    - [x] Handle regex compilation errors (display in status bar or near input).

## Phase 4: Snippet Management
- [x] **Snippet Data Structure**
    - [x] Define `Snippet` struct (id, title/preview, content).
    - [x] Add `snippets: Vec<Snippet>` to `Shard`.
- [x] **Sidebar UI**
    - [x] Implement `view_snippets` helper.
    - [x] Use `Scrollable` for the list.
    - [x] Render each snippet as a card with "Load", "Copy", "Delete" buttons.
- [x] **Snippet Actions**
    - [x] Implement "Pin Current Text" logic.
    - [x] Implement "Load Snippet" (replace editor content).
    - [x] Implement "Delete Snippet".
    - [x] Implement "Copy Snippet" to clipboard (using `arboard`).

## Phase 5: Clipboard & Smart Features
- [x] **Clipboard Listener**
    - [x] Add toggle in UI for "Listen to Clipboard".
    - [x] Implement a background subscription/loop to check clipboard content using `arboard`.
    - [x] Auto-pin or notify on clipboard change.
- [ ] **Status & Polish**
    - [ ] **Improvement**: Add Line/Col numbers to Status Bar (need to find API).
    - [ ] Style the application (spacing, padding, colors).
    - [ ] **Smart Content Recognition**: Color previews, etc.

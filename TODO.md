# Shard Project Plan & Todo

## Current Version: Multi-Snippet Manager

The app manages three types of snippets: Colors, Code, and Text.

## Completed Features

### Core Infrastructure
- [x] **Application Structure**
    - [x] Define `Shard` struct with snippet state
    - [x] Implement `iced::application` entry point
    - [x] Create `Message` enum for all user interactions
    - [x] Unified snippet model (`Snippet`, `SnippetKind`, `SnippetContent`)

### Snippet Types (`src/snippet/`)
- [x] **Color Snippets** (`color.rs`)
    - [x] Parse hex: `#RGB`, `#RRGGBB`, `#RRGGBBAA`
    - [x] Parse RGB/RGBA: `rgb(r, g, b)`, `rgba(r, g, b, a)`
    - [x] Parse HSL/HSLA: `hsl(h, s%, l%)`, `hsla(h, s%, l%, a)`
    - [x] Parse OKLCH: `oklch(l% c h)`, `oklch(l% c h / a)`
    - [x] Convert between formats (hex, rgb, hsl, oklch)
    - [x] Extract colors from arbitrary text
- [x] **Code Snippets** (`code.rs`)
    - [x] Language detection (Rust, Python, JS, TS, JSON, HTML, CSS, SQL, Shell, Go)
    - [x] Code heuristics (`looks_like_code()`)
    - [x] Line count and preview
- [x] **Text Snippets** (`text.rs`)
    - [x] Plain text storage
    - [x] Character/line count
    - [x] Preview generation

### Database Persistence (`src/db.rs`)
- [x] **SQLite Storage**
    - [x] Unified `snippets` table with kind-specific columns
    - [x] Auto-create database in user data directory
    - [x] Migration system with schema versioning
    - [x] Auto-migrate from old `colors` table
- [x] **CRUD Operations**
    - [x] Load all snippets on startup
    - [x] Insert new snippets (color, code, text)
    - [x] Update snippets
    - [x] Delete snippets
    - [x] Move duplicate colors to top

### User Interface

#### Main View (`src/view/mod.rs`)
- [x] **Header**
    - [x] Color input field with validation
    - [x] Add buttons: Color, Code, Text
    - [x] Clipboard listening toggle
    - [x] Filter input
- [x] **Tab Filter**
    - [x] All / Colors / Code / Text tabs
    - [x] Filter by snippet kind
    - [x] Combined with text search
- [x] **Snippet List**
    - [x] Scrollable list of snippet cards
    - [x] Click-to-select with visual highlight
- [x] **Status Bar**
    - [x] Snippet count (filtered/total)
    - [x] Last action message

#### Color Features
- [x] **Color Card** (`color_card.rs`)
    - [x] 72x72 color swatch with transparency support
    - [x] Hex display
    - [x] Copy buttons: Hex, RGB, HSL, OKLCH
    - [x] Edit and delete buttons
- [x] **Color Picker Modal** (`color_picker.rs`)
    - [x] HSL mode with hue bar and SL box
    - [x] OKLCH mode with hue bar and CL box
    - [x] Alpha slider
    - [x] Label input
    - [x] Create new or edit existing colors

#### Code Features
- [x] **Code Card** (`code_card.rs`)
    - [x] Code icon
    - [x] Language badge
    - [x] 2-line preview
    - [x] Line count
    - [x] Copy, edit, delete buttons
- [x] **Code Editor Modal** (`code_editor.rs`)
    - [x] Multi-line text editor
    - [x] Language input
    - [x] Label input
    - [x] Create new or edit existing code

#### Text Features
- [x] **Text Card** (`text_card.rs`)
    - [x] Text icon
    - [x] 2-line preview
    - [x] Character/line count
    - [x] Copy, edit, delete buttons
- [x] **Text Editor Modal** (`text_editor.rs`)
    - [x] Multi-line text editor
    - [x] Label input
    - [x] Create new or edit existing text

### Clipboard Integration
- [x] **Smart Detection**
    - [x] Toggle to enable/disable listening
    - [x] Auto-detect snippet type from clipboard
    - [x] Auto-add colors, code, or text snippets
- [x] **Copy Actions**
    - [x] Copy any snippet content
    - [x] Copy colors in multiple formats

### Keyboard Shortcuts
- [x] Ctrl+V to paste/add from clipboard
- [x] Ctrl+N to focus color input
- [x] Escape to close modals / clear filter / deselect
- [x] Delete to remove selected snippet

### Custom Widgets (`src/widgets/`)
- [x] **ColorSwatch** - Renders color with checkerboard for transparency
- [x] **HueBar** - Interactive hue spectrum slider
- [x] **SaturationLightnessBox** - 2D HSL picker
- [x] **ChromaLightnessBox** - 2D OKLCH picker
- [x] **AlphaBar** - Transparency slider

## Future Improvements

### UI Enhancements
- [x] **Palettes/Categories**
    - [x] Create/rename/delete palettes
    - [x] Snippets can belong to multiple palettes
    - [x] Filter by palette
- [x] **Syntax Highlighting**
    - [x] Highlight code in editor (using iced's built-in highlighter)
    - [x] Language-specific colors (Base16Mocha theme)

### Export/Import
- [x] Export snippets as JSON
- [x] Import snippets from file

### External Editor Integration
- [x] **Open in External Editor**
    - [x] Settings for editor command (vscode, helix, nvim, etc.)
    - [x] "Open in Editor" button on code/text cards
    - [x] Create temp file with snippet content
    - [x] Launch editor with temp file path
    - [x] Update snippet when editor closes
- [x] **Editor Presets**
    - [x] VSCode: `code --wait {file}`
    - [x] Helix: `hx {file}`
    - [x] Neovim: `nvim {file}`
    - [x] Vim: `vim {file}`
    - [x] Custom command with `{file}` placeholder
- [x] **Settings UI**
    - [x] Gear icon in header opens settings modal
    - [x] Radio-style preset selection
    - [x] Custom command input
    - [x] Config saved to `~/.config/shard/config.toml`

### Pinned Snippet Windows (Multi-Window)
- [x] **Architecture Migration**
    - [x] Migrate from `iced::application` to `iced::daemon`
    - [x] Add `window::Id` tracking with `BTreeMap<window::Id, WindowKind>`
    - [x] Open main window on startup via `window::open()`
    - [x] Modify `view()`, `title()`, `theme()` to accept `window::Id`
    - [x] Handle `window::close_events()` subscription
- [x] **Pin Feature**
    - [x] Add "Pin" button to snippet cards (all types)
    - [x] `Message::PinSnippet(snippet_id)` opens new always-on-top window
    - [x] Track pinned windows: `BTreeMap<window::Id, WindowKind>`
    - [x] `Message::UnpinSnippet(window_id)` closes pinned window
- [x] **Pinned Window UI**
    - [x] Always on top (`level: Level::AlwaysOnTop`)
    - [x] Small fixed size (300x150)
    - [x] Non-resizable
    - [x] Minimal content view (color swatch / code preview / text preview)
    - [x] Close/unpin button
    - [x] Copy button for snippet content

### Advanced Features
- [ ] Color harmony suggestions (complementary, triadic, etc.)
- [ ] Color contrast checker (WCAG accessibility)
- [ ] Undo/redo for operations
- [ ] Dark/light theme toggle
- [ ] Snippet tags/labels

### Performance
- [ ] Cache color swatches
- [ ] Lazy loading for large lists
- [ ] Virtualized scrolling

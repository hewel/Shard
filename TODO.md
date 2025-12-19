# Shard Project Plan & Todo

## Current Version: Color Palette Manager

The app has been refactored from a text editor into a color palette manager.

## Completed Features

### Core Infrastructure
- [x] **Application Structure**
    - [x] Define `Shard` struct with color palette state
    - [x] Implement `iced::application` entry point
    - [x] Create `Message` enum for all user interactions

### Color Parsing (`src/color.rs`)
- [x] **Multi-format Support**
    - [x] Parse hex colors: `#RGB`, `#RRGGBB`, `#RRGGBBAA`
    - [x] Parse RGB/RGBA: `rgb(r, g, b)`, `rgba(r, g, b, a)`
    - [x] Parse HSL/HSLA: `hsl(h, s%, l%)`, `hsla(h, s%, l%, a)`
- [x] **Format Conversion**
    - [x] Convert to hex string
    - [x] Convert to rgb/rgba string
    - [x] Convert to hsl/hsla string
- [x] **Color Extraction**
    - [x] Extract all colors from arbitrary text (for clipboard detection)

### Database Persistence (`src/db.rs`)
- [x] **SQLite Storage**
    - [x] Store colors with RGBA values, label, and position
    - [x] Auto-create database in user data directory
    - [x] Load colors on startup
- [x] **CRUD Operations**
    - [x] Insert new colors
    - [x] Update color labels
    - [x] Delete colors
    - [x] Move color to top (for duplicate handling)

### User Interface (`src/main.rs`)
- [x] **Color Input**
    - [x] Text input field for entering color values
    - [x] Real-time validation with error feedback (red border)
    - [x] Add button to save colors
- [x] **Color Palette Display**
    - [x] Scrollable list of color cards
    - [x] Visual color swatch with checkerboard for transparency
    - [x] Display hex value
    - [x] Editable labels (click to edit)
- [x] **Copy Actions**
    - [x] Copy as Hex
    - [x] Copy as RGB
    - [x] Copy as HSL
- [x] **Clipboard Listening**
    - [x] Toggle to enable/disable
    - [x] Auto-detect colors from clipboard content
    - [x] Auto-add detected colors to palette
- [x] **Status Bar**
    - [x] Show color count
    - [x] Show last action/status message

### Behavior
- [x] **Duplicate Handling**
    - [x] Detect duplicate colors by RGBA values
    - [x] Move existing color to top instead of creating duplicate
- [x] **Transparency Support**
    - [x] Full RGBA/HSLA alpha channel support
    - [x] Checkerboard background in swatches to visualize transparency

## Future Improvements

### UI Enhancements
- [ ] Color picker widget (visual color selection)
- [ ] Drag-and-drop to reorder colors
- [ ] Group colors into palettes/categories
- [ ] Search/filter colors by label or value
- [ ] Keyboard shortcuts (Ctrl+V to add from clipboard, etc.)

### Export/Import
- [ ] Export palette as JSON
- [ ] Export palette as CSS variables
- [ ] Export palette as SCSS/SASS variables
- [ ] Import colors from file
- [ ] Import from image (color extraction)

### Advanced Features
- [ ] Color harmony suggestions (complementary, triadic, etc.)
- [ ] Color contrast checker (WCAG accessibility)
- [ ] Color naming (auto-suggest names like "Ocean Blue")
- [ ] Undo/redo for color operations
- [ ] Dark/light theme toggle

### Performance
- [ ] Cache color swatches to avoid redrawing
- [ ] Lazy loading for large palettes

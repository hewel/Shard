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
- [x] **Search/Filter**
    - [x] Filter input field in header row
    - [x] Filter colors by label or hex value
    - [x] Clear filter button (×)
    - [x] Status bar shows filtered count (X / Y colors)
- [x] **Keyboard Shortcuts**
    - [x] Ctrl+V to paste/add color from clipboard
    - [x] Ctrl+N to focus color input
    - [x] Escape to cancel editing / clear filter / deselect
    - [x] Delete to remove selected color
    - [x] Click-to-select colors with visual highlight

### Behavior
- [x] **Duplicate Handling**
    - [x] Detect duplicate colors by RGBA values
    - [x] Move existing color to top instead of creating duplicate
- [x] **Transparency Support**
    - [x] Full RGBA/HSLA alpha channel support
    - [x] Checkerboard background in swatches to visualize transparency

## Future Improvements

### UI Enhancements (Planned Phases)
- [x] **Phase 3: Color Picker Modal** (~3-4 hours)
    - [x] Full HSL/RGB color picker with sliders
    - [x] Add new colors via picker
    - [x] Edit existing colors
- [ ] **Phase 4: Palettes/Categories** (~4-5 hours)
    - [ ] Create/rename/delete palettes
    - [ ] Colors can belong to multiple palettes
    - [ ] Filter by palette
    - [ ] Show palette badges on colors
- [ ] **Phase 5: Drag-and-Drop Reordering** (~4-5 hours)
    - [ ] Drag colors to reorder
    - [ ] Visual feedback during drag
    - [ ] Persist order to database

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

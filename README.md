# Shard

A lightweight desktop application for managing colors, code snippets, and text snippets. Built with Rust and the Iced GUI framework.

## Features

- **Smart Editor**: Multi-line text editor with syntax highlighting for code (Rust, JSON, Python, etc.)
- **Color Management**: Detect and preview color values (hex codes like `#FF5733`, RGB values)
- **Snippet Management**: Pin, load, delete, and copy text snippets with a single click
- **Regex Processing**: Batch find-and-replace with full regex pattern support
- **Clipboard Monitoring**: Optional clipboard listening to auto-capture snippets
- **Keyboard Shortcuts**: Configurable shortcuts with recording support
- **Persistent Storage**: Snippets stored in a local SQLite database

## Installation

### Prerequisites

- Rust 1.92.0+
- Cargo package manager

### Build from Source

```bash
# Clone the repository
git clone <repository-url>
cd Shard

# Build in debug mode
cargo build

# Build in release mode (optimized)
cargo build --release
```

### Run

```bash
# Development
cargo run

# Release
cargo run --release
```

## Usage

1. **Adding Snippets**: Use the add menu to create color, code, or text snippets
2. **Color Input**: Enter hex colors (e.g., `#FF5733`) or use the color picker
3. **Code Snippets**: Paste code and it will be syntax highlighted automatically
4. **Clipboard Listening**: Toggle clipboard monitoring to auto-capture copied text
5. **Filtering**: Filter snippets by type or search text
6. **Keyboard Shortcuts**: Configure shortcuts in the settings panel

## Development

```bash
# Run tests
cargo test

# Run linter
cargo clippy -- -D warnings

# Format code
cargo fmt --all
```

## Project Structure

```
src/
├── snippet/       # Snippet types (code, color, text)
├── view/          # UI components (editors, cards, pickers)
├── widgets/       # Custom widgets (color picker components)
├── config.rs      # Configuration and keyboard shortcuts
├── db.rs          # SQLite database operations
├── icons.rs       # Icon and font definitions
├── main.rs        # Application entry point
├── message.rs     # Message definitions (Elm architecture)
├── theme.rs       # Theme definitions
└── update.rs      # State update logic
```

## Architecture

Shard follows the Elm architecture pattern:

1. **State**: Application data stored in the `Shard` struct
2. **View**: Renders the current state as UI widgets
3. **Message**: Events triggered by user interactions
4. **Update**: Modifies state in response to messages

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| iced | 0.14 | GUI framework |
| regex | 1.12 | Pattern matching |
| arboard | 3.6 | Clipboard access |
| tokio | 1.48 | Async runtime |
| rusqlite | 0.31 | SQLite database |
| serde | 1.0 | Serialization |
| toml | 0.8 | Config parsing |
| directories | 5.0 | Platform directories |
| nanoid | 0.4.0 | Unique ID generation |

## License

MIT License - see [LICENSE-MIT](LICENSE-MIT) for details.

## Contributing

Contributions are welcome. Please open an issue or submit a pull request.

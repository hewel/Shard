## Build & Verification

- **Build**: `cargo build`
- **Run**: `cargo run`
- **Test**: `cargo test` (Single: `cargo test <test_name>`)
- **Lint**: `cargo clippy -- -D warnings`
- **Format**: `cargo fmt --all`
- **Install library**: `cargo add <crate>` **DO NOT** edit `Cargo.toml` manually.

## Code Standards

- **Framework**: Use `iced` (master). **MUST** use `iced::application` entry point.
- **Async**: Use `Task` for async operations (not `Command`).
- **Style**: Strictly follow `rustfmt` and standard Rust idioms.
- **Naming**: `snake_case` for locals/functions, `PascalCase` for types.
- **Imports**: Group `std`, external crates (`iced`, `regex`), and local modules.
- **Errors**: Handle errors explicitly with `Result/Option`. Avoid `unwrap()`.
- **Architecture**: Follow Elm Architecture (State -> View -> Message -> Update).

## Context

- See `@doc/requirements.md` for feature specs (Smart Editor, Snippets).
- See `@doc/best-practices.md` for Iced-specific implementation details.

## Icons

Uses Phosphor-Light font (`fonts/Phosphor-Light.ttf`). Icon functions in `src/icons.rs`.

Available icons (see `fonts/style.css` for full list):
- `pencil`, `x`, `copy`, `trash`, `plus`, `check`
- `magnifying_glass`, `floppy_disk`, `clipboard`, `eye`
- `funnel`, `arrow_clockwise`, `x_circle`, `code`, `text_icon`
- `arrow_square_out`, `gear`, `palette`, `tag`, `folder`

To add new icons: find unicode in `fonts/style.css` (e.g., `.ph-light.ph-icon-name:before { content: "\eXXX"; }`), add function to `icons.rs`.

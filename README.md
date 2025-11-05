# BBoard - Breadboarding TUI Tool

A terminal-based implementation of Basecamp's breadboarding methodology for designing user interface flows.

## What is Breadboarding?

Breadboarding is a method for sketching user interface flows using places, affordances, and connections without getting bogged down in visual design. It focuses on the topology and user journey rather than visual aesthetics.

## Features

- **Text-based interface** - Focus on content over visuals
- **Modal editing** - Navigate mode and Edit mode for precise control
- **Quick navigation** - Arrow keys, Tab, and connection following
- **Connection management** - Visual links between places and affordances
- **Collapsed/Expanded views** - Overview vs detailed view
- **Filtering** - Show only connected places
- **TOML save/load** - Human-readable file format
- **International keyboard support** - Compatible with non-US layouts

## Controls

### Navigation Mode
- `↑/↓` - Navigate between places (places are the main screens/pages)
- `Tab` - Navigate into affordances (actions/buttons within a place)
- `Shift+Tab` - Navigate back to parent place
- `Enter` - Follow connections (on affordances)
- `e` - Enter edit mode to edit selected place/affordance
- `Delete` - Delete selected place or affordance
- `Backspace/Esc` - Go back in navigation trail or cancel edit
- `Ctrl+Q` - Quit

### Edit Mode
- `e` (on selected item) - Enter edit mode
- Type text - Edit the selected place/affordance name
- `Backspace` - Delete characters
- `Enter` - Save changes and exit edit mode
- `Esc` - Cancel edit and exit edit mode
- `Delete` - Delete selected place or affordance (from navigation mode)

**Navigation Pattern:**
1. Use `↑/↓` to select a place (like "Invoice", "Setup Autopay")
2. Press `Tab` to go into that place's affordances (actions like "Turn on Autopay")
3. Use `Tab` to navigate between affordances within the place (stays at last affordance)
4. Use `Shift+Tab` to go back to the parent place level
5. Press `Enter` on an affordance with a connection (→) to follow it
6. Press `e` to edit the selected place or affordance name
7. Use `↑/↓` to navigate between places at any level

### Creation
- `Ctrl+N` - New place
- `Ctrl+A` - New affordance (on selected place)
- `Ctrl+C` - Enter connection mode (from selected affordance)
- `Ctrl+R` - Remove connection from selected affordance

### Connection Mode
When in connection mode (selected affordance + Ctrl+C):
- **"Remove connection"** (first option) - Remove existing connection
- Type characters - Search/filter places by name
- `↑/↓` - Navigate search results (including remove option)
- `Enter` - Create connection or remove connection (if selected)
- `Esc` - Cancel connection mode

### Views
- `c` - Toggle collapsed/expanded view
- `Ctrl+F` - Filter to show only connected places

### File Operations
- `Ctrl+S` - Save breadboard
- `Ctrl+O` - Open breadboard

### Edit Mode
- `Enter` - Save changes
- `Esc` - Cancel edit
- `Backspace` - Delete character
- Text input - Edit place/affordance names

## Data Format

Breadboards are saved as TOML files:

```toml
name = "My Breadboard"
created = "2025-01-15T10:00:00Z"

[[places]]
name = "Invoice"

[[places.affordances]]
name = "Turn on Autopay"
connects_to = "place-uuid"

[[places]]
name = "Setup Autopay"

[[places.affordances]]
name = "CC Fields"
connects_to = "place-uuid"
```

## Examples

### 90s Personal Website Example
The project includes a fun, extensive example of a 90s-style personal website with:
- 25+ places covering all the classic 90s web elements
- Animated GIFs, MIDI music, guestbooks, and webrings
- Browser compatibility warnings and "Best viewed in Netscape"
- Download sections for WinAmp skins and desktop themes
- Chat rooms, photo galleries, and personal hobby pages

Load it with: `Ctrl+O` → `90s-personal-website.toml`

This example demonstrates complex user flows with multiple navigation paths, perfect for exploring the tool's filtering and connection-following features. It's a nostalgic journey through web design history!

## Installation

```bash
cargo build --release
```

## Usage

```bash
cargo run
```

The app starts with sample data demonstrating the Autopay flow from Basecamp's breadboarding guide.

### First Steps:
1. **Navigate**: Use `↑/↓` to see different places (Invoice, Setup Autopay, Confirm)
2. **Explore**: Press `Tab` on "Invoice" to see its affordances
3. **Follow Connections**: Press `Enter` on "Turn on Autopay → Setup" to jump to the Setup place
4. **Edit Items**: Select any place/affordance and press `e` to edit its name
5. **Create Connections**: Select an affordance and press `Ctrl+C`, then type to search for places
6. **Remove Connections**: Select an affordance with a connection and press `Ctrl+R` to remove it
7. **Delete Items**: Select any place/affordance and press `Delete` to remove it
8. **Navigate Back**: Press `Backspace` to return to the previous place
9. **Filter**: Press `Ctrl+F` to see only places connected to your current selection
10. **Try the 90s Example**: Press `Ctrl+O` and load `90s-personal-website.toml`

### Understanding the Display:
- **Places** are shown as headers: `┌─ Invoice`
- **Affordances** are listed under places: `├─ Turn on Autopay → Setup`
- **Connections** are shown with arrows: `→ Destination`
- **Incoming connections** show: `(← 2 sources)` in collapsed view

## Testing

The project includes comprehensive automated tests:

```bash
# Run all tests
cargo test

# Run tests in release mode
cargo test --release

# Run specific test module
cargo test models::tests

# Run with output
cargo test -- --nocapture
```

### Test Coverage

- **Models**: Data structures, serialization, connections
- **File Operations**: Save/load, error handling, TOML parsing
- **Application Logic**: Navigation, state management, UI interactions

## Development

### Code Quality

- Zero warnings (`cargo check` produces no output)
- Comprehensive test suite (26 tests)
- Clippy-approved code style
- Memory-safe Rust implementation

### Project Structure

```
├── src/
│   ├── main.rs         # Entry point and event loop
│   ├── app.rs          # Application state and business logic
│   ├── models.rs       # Data structures with tests
│   ├── ui.rs           # TUI rendering
│   ├── input.rs        # Keyboard handling
│   └── file.rs         # File I/O with tests
├── tests/              # Integration tests
├── Cargo.toml          # Dependencies
└── README.md           # This file
```

## License

MIT
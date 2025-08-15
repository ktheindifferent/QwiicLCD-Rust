# QwiicLCD-Rust Project Overview

## Codebase Description

This is a Rust library for controlling QwiicLCD screens using I2C from Linux systems. The library primarily targets ARM devices such as Raspberry Pi and FriendlyARM's NanoPi Neo, but works on any Linux distribution with I2C bus access.

### Key Features
- I2C communication with QwiicLCD displays
- Support for various LCD sizes (default 4x20, configurable)
- Backlight color control (RGB)
- Cursor positioning and text display
- Clear screen and cursor control functionality
- Comprehensive unit tests with edge case handling

## Codebase Structure

```
/root/repo/
├── Cargo.toml           # Rust package manifest
├── LICENSE              # Licensing information
├── README.md            # User documentation and examples
├── overview.md          # Additional project overview
├── project_description.md # Project description
├── todo.md              # Development tasks
└── src/
    └── lib.rs           # Main library implementation
```

### Core Components

#### `src/lib.rs`
The main library file containing:
- **Enums**: Command definitions for LCD control (ClearDisplay, SetRGB, etc.)
- **ScreenConfig**: Configuration struct for LCD dimensions and settings
- **Screen**: Main struct for interacting with the LCD display
- **Public API Methods**:
  - `new()`: Initialize screen with config
  - `clear()`: Clear the display
  - `move_cursor()`: Position cursor at specific row/column
  - `print()`: Display text on screen
  - `change_backlight()`: Set RGB backlight colors
  - `set_contrast()`: Adjust display contrast
  - `create_character()`: Define custom characters
  - Various display control methods (cursor visibility, blink, etc.)

## Dependencies

### Direct Dependencies (from Cargo.toml)
- **i2cdev** (0.4.4): Linux I2C device interface for hardware communication
- **enum_primitive** (0.1.1): Utilities for working with primitive enum types

### Development Information
- **Language**: Rust (Edition 2021)
- **Version**: 0.1.11
- **License**: Dual licensed under MIT OR Apache-2.0
- **Repository**: https://github.com/PixelCoda/QwiicLCD-Rust
- **Documentation**: https://docs.rs/qwiic-lcd-rs

## Testing

The codebase includes comprehensive unit tests covering:
- Screen initialization and configuration
- Cursor movement and boundary checks
- Text printing and display operations
- Backlight control
- Custom character creation
- Edge cases and error handling
- Map function for value scaling

Tests can be run with:
```bash
cargo test
```

## Recent Updates

### Latest Changes (as of current branch: terragon/update-claude-md)
- Added comprehensive unit tests for all major functionality
- Fixed edge cases in map function
- Improved code quality and error handling
- Enhanced documentation and code organization
- Bug fixes in ScreenConfig and display control methods

## Usage Example

```rust
use qwiic_lcd_rs::*;

fn main() {
    let mut config = ScreenConfig::default(); // 4x20 LCD
    let mut screen = Screen::new(config, "/dev/i2c-1", 0x72).unwrap();
    
    screen.change_backlight(255, 255, 255).unwrap(); // White backlight
    screen.clear().unwrap();
    screen.move_cursor(0, 0).unwrap();
    screen.print("Hello, World!").unwrap();
}
```

## Development Notes

- Primary testing has been done with 20x4 LCD displays
- The library uses Linux I2C device files (typically `/dev/i2c-1`)
- Default I2C address is 0x72 (configurable)
- Thread delays are used for command processing timing
- The library follows Rust idioms with Result types for error handling
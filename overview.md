# QwiicLCD-Rust Project Overview

## Project Description
A Rust library for controlling QwiicLCD screens using I2C communication on Linux systems, primarily targeting ARM devices like Raspberry Pi and NanoPi Neo.

## Architecture

### Core Components

1. **Command Enums** - Define LCD control commands and display modes
   - `Command`: LCD control commands (Clear, Home, SetRGB, etc.)
   - Display control enums: `EntryMode`, `DisplayStatus`, `CursorState`, `BlinkState`
   - Movement enums: `MoveType`, `MoveDirection`

2. **Configuration Structs**
   - `ScreenConfig`: Stores LCD dimensions (rows/columns)
   - `DisplayState`: Tracks current display status, cursor, and blink states

3. **Main Controller**
   - `Screen`: Primary struct managing I2C communication and LCD operations
   - Uses Linux I2C device for hardware communication
   - Provides high-level methods for LCD control

### Key Features
- RGB backlight control
- Cursor positioning and visibility control
- Text display capabilities
- Display clearing and homing
- Configurable screen dimensions (default 4x20)

### I2C Communication
- Default address: 0x72
- Uses smbus protocol for byte and block data transmission
- Implements timing delays for LCD responsiveness

### Testing
- Integration test included that exercises all major functionality
- Tests backlight color changes, text display, and cursor control

## Dependencies
- `i2cdev`: Linux I2C device communication
- `enum_primitive`: Enum conversions (legacy dependency)

## Usage Pattern
1. Create ScreenConfig (or use default)
2. Initialize Screen with config, I2C bus, and address
3. Call init() to set up display
4. Use methods to control display (print, clear, move_cursor, etc.)
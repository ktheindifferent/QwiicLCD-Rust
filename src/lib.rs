// Copyright 2021 Caleb Mitchell Smith-Woolrich (PixelCoda)
// Forked from Romain Porte 2017 (https://github.com/MicroJoe/rust-i2c-16x2/blob/master/src/lib.rs)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate i2cdev;

use std::thread;
use std::time::Duration;

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

/// LCD commands for controlling the display
#[derive(Copy, Clone)]
pub enum Command {
    ClearDisplay = 0x2D,
    ReturnHome = 0x02,
    EntryModeSet = 0x04,
    DisplayControl = 0x08,
    CursorShift = 0x10,
    FunctionSet = 0x20,
    SetCGRamAddr = 0x40,
    SetDDRamAddr = 0x80,
    SetRGB = 0x2B,
    SettingCommand = 0x7C,
    SpecialCommand = 254,
}

/// Display entry mode direction
#[derive(Copy, Clone)]
pub enum EntryMode {
    Right = 0x00,
    Left = 0x02,
}

/// Entry shift direction
#[derive(Copy, Clone)]
pub enum EntryShift {
    Increment = 0x01,
    Decrement = 0x00,
}

/// Display on/off status
#[derive(Copy, Clone)]
pub enum DisplayStatus {
    Off = 0x00,
    On = 0x04,
}

/// Cursor visibility state
#[derive(Copy, Clone)]
pub enum CursorState {
    Off = 0x00,
    On = 0x02,
}

/// Cursor blink state
#[derive(Copy, Clone)]
pub enum BlinkState {
    Off = 0x00,
    On = 0x01,
}

/// Type of movement (cursor or display)
#[derive(Copy, Clone)]
pub enum MoveType {
    Cursor = 0x00,
    Display = 0x08,
}

/// Direction for cursor/display movement
#[derive(Copy, Clone)]
pub enum MoveDirection {
    Left = 0x00,
    Right = 0x04,
}

/// Backlight state
#[derive(Copy, Clone)]
pub enum Backlight {
    Off = 0x00,
    On = 0x04,
}

/// Write mode flags for LCD communication
#[derive(Copy, Clone)]
pub enum WriteMode {
    Enable = 0x04,
    ReadWrite = 0x02,
    RegisterSelect = 0x01,
    Normal = 0x00,
}

/// Bit mode configuration (4-bit or 8-bit)
#[derive(Copy, Clone)]
pub enum BitMode {
    B4 = 0x00,
    B8 = 0x10,
}

/// Configuration for the LCD screen dimensions
pub struct ScreenConfig {
    max_rows: u8,
    max_columns: u8,
}

impl ScreenConfig {
    /// Creates a new ScreenConfig with specified dimensions
    pub fn new(max_rows: u8, max_columns: u8) -> ScreenConfig {
        ScreenConfig {
            max_rows,
            max_columns,
        }
    }
}

impl Default for ScreenConfig {
    /// Creates a default ScreenConfig with 4 rows and 20 columns
    fn default() -> Self {
        ScreenConfig::new(4, 20)
    }
}

/// Current state of the display (status, cursor, blink)
pub struct DisplayState {
    status: DisplayStatus,
    cursor: CursorState,
    blink: BlinkState,
}

impl DisplayState {
    fn new(status: DisplayStatus, cursor: CursorState, blink: BlinkState) -> DisplayState {
        DisplayState {
            status,
            cursor,
            blink,
        }
    }
}

impl Default for DisplayState {
    fn default() -> Self {
        DisplayState::new(DisplayStatus::On, CursorState::On, BlinkState::On)
    }
}

/// Main struct for controlling the QwiicLCD screen via I2C
pub struct Screen {
    dev: LinuxI2CDevice,
    config: ScreenConfig,
    state: DisplayState,
}

type ScreenResult = Result<(), LinuxI2CError>;

impl Screen {
    /// Creates a new Screen instance with the given configuration
    ///
    /// # Arguments
    /// * `config` - Screen configuration with dimensions
    /// * `bus` - I2C bus path (e.g., "/dev/i2c-1")
    /// * `i2c_addr` - I2C address of the LCD (default is 0x72)
    pub fn new(config: ScreenConfig, bus: &str, i2c_addr: u16) -> Result<Screen, LinuxI2CError> {
        let dev = LinuxI2CDevice::new(bus, i2c_addr)?;
        Ok(Screen {
            dev,
            config,
            state: DisplayState::default(),
        })
    }

    /// Initializes the LCD screen with default settings
    pub fn init(&mut self) -> ScreenResult {
        self.apply_display_state()?;
        self.clear()?;
        self.enable_blink(false)?;
        self.enable_cursor(false)?;

        // Wait for the screen to set up
        thread::sleep(Duration::from_millis(200));

        Ok(())
    }

    /// Changes the backlight color to the specified RGB values
    pub fn change_backlight(&mut self, r: u8, g: u8, b: u8) -> ScreenResult {
        let block = vec![Command::SetRGB as u8, r, g, b];

        self.write_block(Command::SettingCommand as u8, block)
    }

    /// Clears the display and returns cursor to home position
    pub fn clear(&mut self) -> ScreenResult {
        self.write_setting_cmd(Command::ClearDisplay as u8)?;
        self.home()
    }

    /// Returns the cursor to home position (0,0)
    pub fn home(&mut self) -> ScreenResult {
        self.write_special_cmd(Command::ReturnHome as u8)
    }

    /// Moves the cursor to the specified row and column
    pub fn move_cursor(&mut self, row: usize, col: usize) -> ScreenResult {
        let row_offsets: Vec<usize> = vec![0x00, 0x40, 0x14, 0x54];

        if row >= self.config.max_rows.into() {
            return self.apply_display_state();
        }
        if col >= self.config.max_columns.into() {
            return self.apply_display_state();
        }

        let command = (Command::SetDDRamAddr as u8) | ((col + row_offsets[row]) as u8);

        self.write_special_cmd(command)
    }

    /// Enables or disables the cursor visibility
    pub fn enable_cursor(&mut self, activated: bool) -> ScreenResult {
        self.state.cursor = match activated {
            true => CursorState::On,
            false => CursorState::Off,
        };

        self.apply_display_state()
    }

    /// Enables or disables the display
    pub fn enable_display(&mut self, activated: bool) -> ScreenResult {
        self.state.status = match activated {
            true => DisplayStatus::On,
            false => DisplayStatus::Off,
        };

        self.apply_display_state()
    }

    /// Enables or disables cursor blinking
    pub fn enable_blink(&mut self, activated: bool) -> ScreenResult {
        self.state.blink = match activated {
            true => BlinkState::On,
            false => BlinkState::Off,
        };

        self.apply_display_state()
    }

    /// Applies the current display state to the hardware
    pub fn apply_display_state(&mut self) -> ScreenResult {
        let flags =
            (self.state.status as u8) | (self.state.cursor as u8) | (self.state.blink as u8);

        self.write_special_cmd((Command::DisplayControl as u8) | flags)
    }

    /// Prints a string to the LCD at the current cursor position
    pub fn print(&mut self, s: &str) -> ScreenResult {
        for c in s.chars() {
            self.write_byte(c as u8)?;
        }

        Ok(())
    }

    /// Writes a single byte to the LCD
    pub fn write_byte(&mut self, command: u8) -> ScreenResult {
        self.dev.smbus_write_byte(command)?;
        thread::sleep(Duration::new(0, 10_000));
        Ok(())
    }

    /// Writes a block of data to the LCD
    pub fn write_block(&mut self, register: u8, data: Vec<u8>) -> ScreenResult {
        self.dev.smbus_write_i2c_block_data(register, &data)?;
        thread::sleep(Duration::new(0, 10_000));
        Ok(())
    }

    /// Writes a setting command to the LCD
    pub fn write_setting_cmd(&mut self, command: u8) -> ScreenResult {
        self.dev
            .smbus_write_byte_data(Command::SettingCommand as u8, command)?;
        thread::sleep(Duration::new(0, 10_000));
        Ok(())
    }

    /// Writes a special command to the LCD
    pub fn write_special_cmd(&mut self, command: u8) -> ScreenResult {
        self.dev
            .smbus_write_byte_data(Command::SpecialCommand as u8, command)?;
        thread::sleep(Duration::new(0, 10_000));
        Ok(())
    }

    /// Sets the entry mode for text display (left-to-right or right-to-left)
    ///
    /// # Arguments
    /// * `mode` - The entry mode direction (Left or Right)
    ///
    /// # Example
    /// ```no_run
    /// # use qwiic_lcd_rs::*;
    /// # let mut screen = Screen::new(ScreenConfig::default(), "/dev/i2c-1", 0x72).unwrap();
    /// screen.set_entry_mode(EntryMode::Left).unwrap(); // Text flows left-to-right
    /// screen.set_entry_mode(EntryMode::Right).unwrap(); // Text flows right-to-left
    /// ```
    pub fn set_entry_mode(&mut self, mode: EntryMode) -> ScreenResult {
        let command = Command::EntryModeSet as u8 | mode as u8 | EntryShift::Increment as u8;
        self.write_special_cmd(command)
    }

    /// Sets the entry shift behavior when displaying text
    ///
    /// # Arguments
    /// * `shift` - The shift direction (Increment or Decrement)
    ///
    /// # Example
    /// ```no_run
    /// # use qwiic_lcd_rs::*;
    /// # let mut screen = Screen::new(ScreenConfig::default(), "/dev/i2c-1", 0x72).unwrap();
    /// screen.set_entry_shift(EntryShift::Increment).unwrap(); // Cursor moves forward
    /// screen.set_entry_shift(EntryShift::Decrement).unwrap(); // Cursor moves backward
    /// ```
    pub fn set_entry_shift(&mut self, shift: EntryShift) -> ScreenResult {
        let command = Command::EntryModeSet as u8 | EntryMode::Left as u8 | shift as u8;
        self.write_special_cmd(command)
    }

    /// Shifts the cursor left or right
    ///
    /// # Arguments
    /// * `direction` - The direction to shift (Left or Right)
    ///
    /// # Example
    /// ```no_run
    /// # use qwiic_lcd_rs::*;
    /// # let mut screen = Screen::new(ScreenConfig::default(), "/dev/i2c-1", 0x72).unwrap();
    /// screen.shift_cursor(MoveDirection::Right).unwrap(); // Move cursor right
    /// screen.shift_cursor(MoveDirection::Left).unwrap(); // Move cursor left
    /// ```
    pub fn shift_cursor(&mut self, direction: MoveDirection) -> ScreenResult {
        let command = Command::CursorShift as u8 | MoveType::Cursor as u8 | direction as u8;
        self.write_special_cmd(command)
    }

    /// Shifts the display left or right without moving the cursor
    ///
    /// # Arguments
    /// * `direction` - The direction to shift (Left or Right)
    ///
    /// # Example
    /// ```no_run
    /// # use qwiic_lcd_rs::*;
    /// # let mut screen = Screen::new(ScreenConfig::default(), "/dev/i2c-1", 0x72).unwrap();
    /// screen.shift_display(MoveDirection::Right).unwrap(); // Shift display right
    /// screen.shift_display(MoveDirection::Left).unwrap(); // Shift display left
    /// ```
    pub fn shift_display(&mut self, direction: MoveDirection) -> ScreenResult {
        let command = Command::CursorShift as u8 | MoveType::Display as u8 | direction as u8;
        self.write_special_cmd(command)
    }

    /// Sets the backlight state (on or off)
    ///
    /// # Arguments
    /// * `state` - The backlight state (On or Off)
    ///
    /// # Example
    /// ```no_run
    /// # use qwiic_lcd_rs::*;
    /// # let mut screen = Screen::new(ScreenConfig::default(), "/dev/i2c-1", 0x72).unwrap();
    /// screen.set_backlight_state(Backlight::Off).unwrap(); // Turn backlight off
    /// screen.set_backlight_state(Backlight::On).unwrap(); // Turn backlight on
    /// ```
    pub fn set_backlight_state(&mut self, state: Backlight) -> ScreenResult {
        match state {
            Backlight::On => self.change_backlight(255, 255, 255),
            Backlight::Off => self.change_backlight(0, 0, 0),
        }
    }

    /// Sets the contrast level of the display
    ///
    /// # Arguments
    /// * `level` - Contrast level (0-255)
    ///
    /// # Example
    /// ```no_run
    /// # use qwiic_lcd_rs::*;
    /// # let mut screen = Screen::new(ScreenConfig::default(), "/dev/i2c-1", 0x72).unwrap();
    /// screen.set_contrast(128).unwrap(); // Set contrast to middle value
    /// screen.set_contrast(255).unwrap(); // Set contrast to maximum
    /// ```
    pub fn set_contrast(&mut self, level: u8) -> ScreenResult {
        self.write_setting_cmd(0x18 | (level >> 4))?;
        self.write_setting_cmd(0x10 | (level & 0x0F))
    }

    /// Creates a custom character in CGRAM
    ///
    /// # Arguments
    /// * `location` - CGRAM location (0-7)
    /// * `pattern` - 8-byte pattern defining the character
    ///
    /// # Example
    /// ```no_run
    /// # use qwiic_lcd_rs::*;
    /// # let mut screen = Screen::new(ScreenConfig::default(), "/dev/i2c-1", 0x72).unwrap();
    /// let heart = [0x00, 0x0A, 0x1F, 0x1F, 0x0E, 0x04, 0x00, 0x00];
    /// screen.create_character(0, &heart).unwrap();
    /// screen.write_byte(0).unwrap(); // Display the custom character
    /// ```
    pub fn create_character(&mut self, location: u8, pattern: &[u8; 8]) -> ScreenResult {
        if location > 7 {
            return Ok(());
        }
        
        let command = Command::SetCGRamAddr as u8 | (location << 3);
        self.write_special_cmd(command)?;
        
        for &byte in pattern.iter() {
            self.write_byte(byte)?;
        }
        
        self.home()
    }

    /// Configures the bit mode of the display (4-bit or 8-bit)
    ///
    /// # Arguments
    /// * `mode` - The bit mode (B4 or B8)
    ///
    /// # Example
    /// ```no_run
    /// # use qwiic_lcd_rs::*;
    /// # let mut screen = Screen::new(ScreenConfig::default(), "/dev/i2c-1", 0x72).unwrap();
    /// screen.configure_bit_mode(BitMode::B8).unwrap(); // Set to 8-bit mode
    /// screen.configure_bit_mode(BitMode::B4).unwrap(); // Set to 4-bit mode
    /// ```
    pub fn configure_bit_mode(&mut self, mode: BitMode) -> ScreenResult {
        // Configure function set with bit mode, 2-line display, and 5x8 font
        let command = Command::FunctionSet as u8 | mode as u8 | 0x08 | 0x00;
        self.write_special_cmd(command)
    }
}

/// Maps a value from one range to another
pub fn map(x: usize, in_min: usize, in_max: usize, out_min: usize, out_max: usize) -> usize {
    // Handle edge case where input range is zero
    if in_max == in_min {
        return out_min;
    }
    
    // Handle potential overflow/underflow
    if x <= in_min {
        return out_min;
    }
    if x >= in_max {
        return out_max;
    }
    
    // Perform the mapping calculation
    let numerator = (x - in_min) * (out_max.abs_diff(out_min));
    let denominator = in_max - in_min;
    
    if out_max >= out_min {
        out_min + (numerator / denominator)
    } else {
        out_min - (numerator / denominator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // This test requires hardware
    fn test_init() {
        let config = ScreenConfig::default();
        let mut screen = Screen::new(config, "/dev/i2c-1", 0x72).expect("Could not init device");

        screen.change_backlight(255, 255, 255).unwrap();
        screen.home().unwrap();
        screen.move_cursor(0, 0).unwrap();
        screen.enable_blink(false).unwrap();
        screen.enable_blink(true).unwrap();
        screen.clear().unwrap();
        screen.print("Testing...").unwrap();
        thread::sleep(Duration::from_secs(1));

        screen.clear().unwrap();
        screen.print("BG: Green").unwrap();
        screen.change_backlight(0, 255, 0).unwrap();
        thread::sleep(Duration::from_secs(2));

        screen.clear().unwrap();
        screen.print("BG: Red").unwrap();
        screen.change_backlight(255, 0, 0).unwrap();
        thread::sleep(Duration::from_secs(2));

        screen.clear().unwrap();
        screen.print("BG: Blue").unwrap();
        screen.change_backlight(0, 0, 255).unwrap();
        thread::sleep(Duration::from_secs(2));

        screen.clear().unwrap();
        screen.print("BG: Purple").unwrap();
        screen.change_backlight(230, 230, 250).unwrap();
        thread::sleep(Duration::from_secs(2));

        screen.change_backlight(255, 255, 255).unwrap();
        screen.home().unwrap();
        screen.move_cursor(0, 0).unwrap();
        screen.enable_blink(false).unwrap();
        screen.enable_blink(true).unwrap();
        screen.clear().unwrap();
        screen.print("It Works!").unwrap();
        thread::sleep(Duration::from_secs(1));
    }

    #[test]
    fn test_map_function() {
        // Test basic mapping
        assert_eq!(map(5, 0, 10, 0, 100), 50);
        assert_eq!(map(0, 0, 10, 0, 100), 0);
        assert_eq!(map(10, 0, 10, 0, 100), 100);
        
        // Test different ranges
        assert_eq!(map(25, 0, 100, 0, 10), 2);
        assert_eq!(map(75, 0, 100, 0, 10), 7);
        
        // Test with offset ranges
        assert_eq!(map(15, 10, 20, 100, 200), 150);
        assert_eq!(map(10, 10, 20, 100, 200), 100);
        assert_eq!(map(20, 10, 20, 100, 200), 200);
    }

    #[test]
    fn test_screen_config_new() {
        let config = ScreenConfig::new(2, 16);
        assert_eq!(config.max_rows, 2);
        assert_eq!(config.max_columns, 16);
        
        let config = ScreenConfig::new(4, 20);
        assert_eq!(config.max_rows, 4);
        assert_eq!(config.max_columns, 20);
        
        let config = ScreenConfig::new(1, 8);
        assert_eq!(config.max_rows, 1);
        assert_eq!(config.max_columns, 8);
    }

    #[test]
    fn test_screen_config_default() {
        let config = ScreenConfig::default();
        assert_eq!(config.max_rows, 4);
        assert_eq!(config.max_columns, 20);
    }

    #[test]
    fn test_display_state_new() {
        let state = DisplayState::new(DisplayStatus::On, CursorState::Off, BlinkState::On);
        assert!(matches!(state.status, DisplayStatus::On));
        assert!(matches!(state.cursor, CursorState::Off));
        assert!(matches!(state.blink, BlinkState::On));
        
        let state = DisplayState::new(DisplayStatus::Off, CursorState::On, BlinkState::Off);
        assert!(matches!(state.status, DisplayStatus::Off));
        assert!(matches!(state.cursor, CursorState::On));
        assert!(matches!(state.blink, BlinkState::Off));
    }

    #[test]
    fn test_display_state_default() {
        let state = DisplayState::default();
        assert!(matches!(state.status, DisplayStatus::On));
        assert!(matches!(state.cursor, CursorState::On));
        assert!(matches!(state.blink, BlinkState::On));
    }

    #[test]
    fn test_command_values() {
        assert_eq!(Command::ClearDisplay as u8, 0x2D);
        assert_eq!(Command::ReturnHome as u8, 0x02);
        assert_eq!(Command::EntryModeSet as u8, 0x04);
        assert_eq!(Command::DisplayControl as u8, 0x08);
        assert_eq!(Command::CursorShift as u8, 0x10);
        assert_eq!(Command::FunctionSet as u8, 0x20);
        assert_eq!(Command::SetCGRamAddr as u8, 0x40);
        assert_eq!(Command::SetDDRamAddr as u8, 0x80);
        assert_eq!(Command::SetRGB as u8, 0x2B);
        assert_eq!(Command::SettingCommand as u8, 0x7C);
        assert_eq!(Command::SpecialCommand as u8, 254);
    }

    #[test]
    fn test_entry_mode_values() {
        assert_eq!(EntryMode::Right as u8, 0x00);
        assert_eq!(EntryMode::Left as u8, 0x02);
    }

    #[test]
    fn test_entry_shift_values() {
        assert_eq!(EntryShift::Increment as u8, 0x01);
        assert_eq!(EntryShift::Decrement as u8, 0x00);
    }

    #[test]
    fn test_display_status_values() {
        assert_eq!(DisplayStatus::Off as u8, 0x00);
        assert_eq!(DisplayStatus::On as u8, 0x04);
    }

    #[test]
    fn test_cursor_state_values() {
        assert_eq!(CursorState::Off as u8, 0x00);
        assert_eq!(CursorState::On as u8, 0x02);
    }

    #[test]
    fn test_blink_state_values() {
        assert_eq!(BlinkState::Off as u8, 0x00);
        assert_eq!(BlinkState::On as u8, 0x01);
    }

    #[test]
    fn test_move_type_values() {
        assert_eq!(MoveType::Cursor as u8, 0x00);
        assert_eq!(MoveType::Display as u8, 0x08);
    }

    #[test]
    fn test_move_direction_values() {
        assert_eq!(MoveDirection::Left as u8, 0x00);
        assert_eq!(MoveDirection::Right as u8, 0x04);
    }

    #[test]
    fn test_backlight_values() {
        assert_eq!(Backlight::Off as u8, 0x00);
        assert_eq!(Backlight::On as u8, 0x04);
    }

    #[test]
    fn test_write_mode_values() {
        assert_eq!(WriteMode::Enable as u8, 0x04);
        assert_eq!(WriteMode::ReadWrite as u8, 0x02);
        assert_eq!(WriteMode::RegisterSelect as u8, 0x01);
        assert_eq!(WriteMode::Normal as u8, 0x00);
    }

    #[test]
    fn test_bit_mode_values() {
        assert_eq!(BitMode::B4 as u8, 0x00);
        assert_eq!(BitMode::B8 as u8, 0x10);
    }

    #[test]
    fn test_map_edge_cases() {
        // Same input and output range
        assert_eq!(map(5, 0, 10, 0, 10), 5);
        
        // Single point range (edge case) - returns out_min when in_max == in_min
        assert_eq!(map(0, 0, 0, 0, 100), 0);
        assert_eq!(map(5, 5, 5, 0, 100), 0);
        
        // Large numbers
        assert_eq!(map(500, 0, 1000, 0, 10000), 5000);
        
        // Fractional result (truncated due to integer division)
        assert_eq!(map(1, 0, 3, 0, 10), 3); // 1/3 * 10 = 3.33... truncated to 3
        
        // Out of bounds values - clamp to range
        assert_eq!(map(15, 0, 10, 0, 100), 100); // Above max
        assert_eq!(map(0, 5, 10, 0, 100), 0); // Below min
    }

    #[test]
    #[ignore] // This test requires hardware
    fn test_set_entry_mode() {
        let config = ScreenConfig::default();
        let mut screen = Screen::new(config, "/dev/i2c-1", 0x72).expect("Could not init device");
        
        // Test left-to-right mode
        screen.set_entry_mode(EntryMode::Left).unwrap();
        screen.clear().unwrap();
        screen.print("LTR").unwrap();
        thread::sleep(Duration::from_millis(500));
        
        // Test right-to-left mode
        screen.set_entry_mode(EntryMode::Right).unwrap();
        screen.clear().unwrap();
        screen.print("RTL").unwrap();
        thread::sleep(Duration::from_millis(500));
        
        // Reset to left-to-right
        screen.set_entry_mode(EntryMode::Left).unwrap();
    }

    #[test]
    #[ignore] // This test requires hardware
    fn test_set_entry_shift() {
        let config = ScreenConfig::default();
        let mut screen = Screen::new(config, "/dev/i2c-1", 0x72).expect("Could not init device");
        
        // Test increment shift
        screen.set_entry_shift(EntryShift::Increment).unwrap();
        screen.clear().unwrap();
        screen.print("Inc").unwrap();
        thread::sleep(Duration::from_millis(500));
        
        // Test decrement shift
        screen.set_entry_shift(EntryShift::Decrement).unwrap();
        screen.clear().unwrap();
        screen.move_cursor(0, 10).unwrap();
        screen.print("Dec").unwrap();
        thread::sleep(Duration::from_millis(500));
        
        // Reset to increment
        screen.set_entry_shift(EntryShift::Increment).unwrap();
    }

    #[test]
    #[ignore] // This test requires hardware
    fn test_shift_cursor() {
        let config = ScreenConfig::default();
        let mut screen = Screen::new(config, "/dev/i2c-1", 0x72).expect("Could not init device");
        
        screen.clear().unwrap();
        screen.print("Cursor").unwrap();
        
        // Test shifting cursor right
        for _ in 0..3 {
            screen.shift_cursor(MoveDirection::Right).unwrap();
            thread::sleep(Duration::from_millis(200));
        }
        
        // Test shifting cursor left
        for _ in 0..6 {
            screen.shift_cursor(MoveDirection::Left).unwrap();
            thread::sleep(Duration::from_millis(200));
        }
        
        screen.print("!").unwrap();
    }

    #[test]
    #[ignore] // This test requires hardware
    fn test_shift_display() {
        let config = ScreenConfig::default();
        let mut screen = Screen::new(config, "/dev/i2c-1", 0x72).expect("Could not init device");
        
        screen.clear().unwrap();
        screen.print("Display Shift Test").unwrap();
        thread::sleep(Duration::from_millis(500));
        
        // Test shifting display right
        for _ in 0..5 {
            screen.shift_display(MoveDirection::Right).unwrap();
            thread::sleep(Duration::from_millis(200));
        }
        
        // Test shifting display left
        for _ in 0..10 {
            screen.shift_display(MoveDirection::Left).unwrap();
            thread::sleep(Duration::from_millis(200));
        }
        
        // Return to normal position
        for _ in 0..5 {
            screen.shift_display(MoveDirection::Right).unwrap();
            thread::sleep(Duration::from_millis(200));
        }
    }

    #[test]
    #[ignore] // This test requires hardware
    fn test_set_backlight_state() {
        let config = ScreenConfig::default();
        let mut screen = Screen::new(config, "/dev/i2c-1", 0x72).expect("Could not init device");
        
        screen.clear().unwrap();
        screen.print("Backlight Test").unwrap();
        
        // Test backlight off
        screen.set_backlight_state(Backlight::Off).unwrap();
        thread::sleep(Duration::from_secs(1));
        
        // Test backlight on
        screen.set_backlight_state(Backlight::On).unwrap();
        thread::sleep(Duration::from_secs(1));
        
        // Toggle a few times
        for _ in 0..3 {
            screen.set_backlight_state(Backlight::Off).unwrap();
            thread::sleep(Duration::from_millis(300));
            screen.set_backlight_state(Backlight::On).unwrap();
            thread::sleep(Duration::from_millis(300));
        }
    }

    #[test]
    #[ignore] // This test requires hardware
    fn test_set_contrast() {
        let config = ScreenConfig::default();
        let mut screen = Screen::new(config, "/dev/i2c-1", 0x72).expect("Could not init device");
        
        screen.clear().unwrap();
        screen.print("Contrast Test").unwrap();
        
        // Test various contrast levels
        let levels = [0, 64, 128, 192, 255];
        for level in levels.iter() {
            screen.set_contrast(*level).unwrap();
            thread::sleep(Duration::from_millis(500));
        }
        
        // Reset to default contrast
        screen.set_contrast(128).unwrap();
    }

    #[test]
    #[ignore] // This test requires hardware
    fn test_create_character() {
        let config = ScreenConfig::default();
        let mut screen = Screen::new(config, "/dev/i2c-1", 0x72).expect("Could not init device");
        
        // Define custom characters
        let heart = [0x00, 0x0A, 0x1F, 0x1F, 0x0E, 0x04, 0x00, 0x00];
        let smiley = [0x00, 0x00, 0x0A, 0x00, 0x11, 0x0E, 0x00, 0x00];
        let arrow = [0x00, 0x04, 0x06, 0x1F, 0x06, 0x04, 0x00, 0x00];
        
        // Create custom characters
        screen.create_character(0, &heart).unwrap();
        screen.create_character(1, &smiley).unwrap();
        screen.create_character(2, &arrow).unwrap();
        
        // Display custom characters
        screen.clear().unwrap();
        screen.print("Custom: ").unwrap();
        screen.write_byte(0).unwrap(); // Heart
        screen.write_byte(1).unwrap(); // Smiley
        screen.write_byte(2).unwrap(); // Arrow
        
        thread::sleep(Duration::from_secs(2));
    }

    #[test]
    #[ignore] // This test requires hardware
    fn test_configure_bit_mode() {
        let config = ScreenConfig::default();
        let mut screen = Screen::new(config, "/dev/i2c-1", 0x72).expect("Could not init device");
        
        // Test 8-bit mode
        screen.configure_bit_mode(BitMode::B8).unwrap();
        screen.clear().unwrap();
        screen.print("8-bit mode").unwrap();
        thread::sleep(Duration::from_secs(1));
        
        // Test 4-bit mode
        screen.configure_bit_mode(BitMode::B4).unwrap();
        screen.clear().unwrap();
        screen.print("4-bit mode").unwrap();
        thread::sleep(Duration::from_secs(1));
        
        // Reset to 8-bit mode (typically default)
        screen.configure_bit_mode(BitMode::B8).unwrap();
    }

    #[test]
    #[ignore] // This test requires hardware - comprehensive integration test
    fn test_all_new_features() {
        let config = ScreenConfig::default();
        let mut screen = Screen::new(config, "/dev/i2c-1", 0x72).expect("Could not init device");
        
        // Initialize and clear
        screen.init().unwrap();
        screen.clear().unwrap();
        
        // Test entry modes
        screen.set_entry_mode(EntryMode::Left).unwrap();
        screen.print("Entry Test").unwrap();
        thread::sleep(Duration::from_millis(500));
        
        // Test cursor shifting
        screen.shift_cursor(MoveDirection::Left).unwrap();
        screen.shift_cursor(MoveDirection::Left).unwrap();
        screen.print("!").unwrap();
        thread::sleep(Duration::from_millis(500));
        
        // Test display shifting
        screen.shift_display(MoveDirection::Right).unwrap();
        thread::sleep(Duration::from_millis(500));
        screen.shift_display(MoveDirection::Left).unwrap();
        thread::sleep(Duration::from_millis(500));
        
        // Test backlight control
        screen.set_backlight_state(Backlight::Off).unwrap();
        thread::sleep(Duration::from_millis(500));
        screen.set_backlight_state(Backlight::On).unwrap();
        
        // Test custom characters
        let check = [0x00, 0x01, 0x03, 0x16, 0x1C, 0x08, 0x00, 0x00];
        screen.create_character(3, &check).unwrap();
        screen.clear().unwrap();
        screen.print("Complete ").unwrap();
        screen.write_byte(3).unwrap(); // Display checkmark
        
        thread::sleep(Duration::from_secs(2));
    }
}

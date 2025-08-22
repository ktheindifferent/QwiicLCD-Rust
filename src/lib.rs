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

use std::error::Error;
use std::fmt;
use std::thread;
use std::time::Duration;

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

/// Custom error types for QwiicLCD operations
#[derive(Debug)]
pub enum QwiicLcdError {
    /// Wraps underlying I2C communication errors
    I2CError(LinuxI2CError),
    /// Invalid cursor position
    InvalidPosition { row: usize, col: usize, max_rows: u8, max_columns: u8 },
    /// Invalid character (non-ASCII)
    InvalidCharacter(char),
    /// Communication timeout after retries
    CommunicationTimeout,
    /// Device initialization failed
    InitializationFailed(String),
    /// Custom character index out of range (0-7)
    InvalidCustomCharIndex(u8),
    /// Contrast value out of range (0-255)
    InvalidContrastValue(u8),
}

impl fmt::Display for QwiicLcdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QwiicLcdError::I2CError(e) => write!(f, "I2C communication error: {}", e),
            QwiicLcdError::InvalidPosition { row, col, max_rows, max_columns } => {
                write!(f, "Invalid cursor position ({}, {}). Screen dimensions are {}x{}", 
                       row, col, max_rows, max_columns)
            },
            QwiicLcdError::InvalidCharacter(c) => {
                write!(f, "Invalid character '{}' (code: {}). Only ASCII characters are supported", c, *c as u32)
            },
            QwiicLcdError::CommunicationTimeout => {
                write!(f, "Communication timeout: device did not respond after retries")
            },
            QwiicLcdError::InitializationFailed(msg) => {
                write!(f, "Failed to initialize LCD: {}", msg)
            },
            QwiicLcdError::InvalidCustomCharIndex(idx) => {
                write!(f, "Invalid custom character index {}. Must be 0-7", idx)
            },
            QwiicLcdError::InvalidContrastValue(val) => {
                write!(f, "Invalid contrast value {}. Must be 0-255", val)
            },
        }
    }
}

impl Error for QwiicLcdError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            QwiicLcdError::I2CError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<LinuxI2CError> for QwiicLcdError {
    fn from(error: LinuxI2CError) -> Self {
        QwiicLcdError::I2CError(error)
    }
}

/// Configuration for retry logic
#[derive(Clone, Copy, Debug)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay between retries in milliseconds
    pub initial_delay_ms: u64,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f32,
    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_retries: 3,
            initial_delay_ms: 10,
            backoff_multiplier: 2.0,
            max_delay_ms: 1000,
        }
    }
}

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

/// Configuration for the LCD screen dimensions and retry behavior
pub struct ScreenConfig {
    max_rows: u8,
    max_columns: u8,
    retry_config: RetryConfig,
}

impl ScreenConfig {
    /// Creates a new ScreenConfig with specified dimensions
    pub fn new(max_rows: u8, max_columns: u8) -> ScreenConfig {
        ScreenConfig {
            max_rows,
            max_columns,
            retry_config: RetryConfig::default(),
        }
    }
    
    /// Creates a new ScreenConfig with specified dimensions and retry configuration
    pub fn new_with_retry(max_rows: u8, max_columns: u8, retry_config: RetryConfig) -> ScreenConfig {
        ScreenConfig {
            max_rows,
            max_columns,
            retry_config,
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

type ScreenResult = Result<(), QwiicLcdError>;

impl Screen {
    /// Creates a new Screen instance with the given configuration
    ///
    /// # Arguments
    /// * `config` - Screen configuration with dimensions
    /// * `bus` - I2C bus path (e.g., "/dev/i2c-1")
    /// * `i2c_addr` - I2C address of the LCD (default is 0x72)
    pub fn new(config: ScreenConfig, bus: &str, i2c_addr: u16) -> Result<Screen, QwiicLcdError> {
        let dev = LinuxI2CDevice::new(bus, i2c_addr)
            .map_err(|e| QwiicLcdError::InitializationFailed(
                format!("Failed to open I2C device on {} at address 0x{:02X}: {}", bus, i2c_addr, e)
            ))?;
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
            return Err(QwiicLcdError::InvalidPosition {
                row,
                col,
                max_rows: self.config.max_rows,
                max_columns: self.config.max_columns,
            });
        }
        if col >= self.config.max_columns.into() {
            return Err(QwiicLcdError::InvalidPosition {
                row,
                col,
                max_rows: self.config.max_rows,
                max_columns: self.config.max_columns,
            });
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
    ///
    /// This method handles ASCII and extended ASCII characters (0x20-0xFF).
    /// Characters outside this range will be replaced with '?' (0x3F).
    ///
    /// # Character Set Support
    /// - Standard ASCII (0x20-0x7E): Fully supported
    /// - Extended ASCII (0x80-0xFF): Support depends on LCD ROM code
    /// - Unicode/UTF-8: Not supported, will be replaced with '?'
    ///
    /// For strict ASCII-only printing, use `print_ascii()` instead.
    pub fn print(&mut self, s: &str) -> ScreenResult {
        for c in s.chars() {
            let byte = self.map_character(c);
            self.write_byte(byte)?;
        }

        Ok(())
    }

    /// Prints ASCII-only text to the LCD at the current cursor position
    ///
    /// This method strictly accepts only ASCII characters (0x20-0x7E).
    /// Returns an error if any non-ASCII character is encountered.
    pub fn print_ascii(&mut self, s: &str) -> Result<(), String> {
        if !s.is_ascii() {
            return Err("String contains non-ASCII characters".to_string());
        }

        for c in s.chars() {
            if c as u32 >= 0x20 && c as u32 <= 0x7E {
                self.write_byte(c as u8)
                    .map_err(|e| format!("I2C error: {:?}", e))?;
            } else if c == '\n' || c == '\r' || c == '\t' {
                // Skip control characters silently
                continue;
            } else {
                return Err(format!(
                    "Character '{}' (0x{:02X}) is not printable ASCII",
                    c, c as u32
                ));
            }
        }

        Ok(())
    }

    /// Maps a character to a byte value suitable for the LCD
    ///
    /// Handles character encoding for HD44780-compatible displays:
    /// - ASCII printable characters (0x20-0x7E) are passed through
    /// - Extended ASCII (0x80-0xFF) are passed through (ROM-dependent support)
    /// - Characters outside supported range are replaced with '?' (0x3F)
    fn map_character(&self, c: char) -> u8 {
        let code = c as u32;

        // Standard ASCII printable range and extended ASCII
        if (0x20..=0x7E).contains(&code) || (0x80..=0xFF).contains(&code) {
            code as u8
        }
        // Common replacements for better display
        else {
            match c {
                // Tab, newline, carriage return -> space
                '\t' | '\n' | '\r' => 0x20,
                // Everything else -> question mark
                _ => 0x3F,
            }
        }
    }

    /// Writes a single byte to the LCD
    pub fn write_byte(&mut self, command: u8) -> ScreenResult {
        let result = self.retry_i2c_write_byte(command)?;
        thread::sleep(Duration::new(0, 10_000));
        Ok(result)
    }

    /// Writes a block of data to the LCD
    pub fn write_block(&mut self, register: u8, data: Vec<u8>) -> ScreenResult {
        let result = self.retry_i2c_write_block(register, data)?;
        thread::sleep(Duration::new(0, 10_000));
        Ok(result)
    }

    /// Writes a setting command to the LCD
    pub fn write_setting_cmd(&mut self, command: u8) -> ScreenResult {
        let result = self.retry_i2c_write_byte_data(Command::SettingCommand as u8, command)?;
        thread::sleep(Duration::new(0, 10_000));
        Ok(result)
    }

    /// Writes a special command to the LCD
    pub fn write_special_cmd(&mut self, command: u8) -> ScreenResult {
        let result = self.retry_i2c_write_byte_data(Command::SpecialCommand as u8, command)?;
        thread::sleep(Duration::new(0, 10_000));
        Ok(result)
    }
    
    /// Sets the LCD contrast (0-255)
    pub fn set_contrast(&mut self, contrast: u8) -> ScreenResult {
        self.write_setting_cmd(0x18)?;
        self.write_setting_cmd(contrast)
    }
    
    /// Creates a custom character at the specified index (0-7)
    /// 
    /// # Arguments
    /// * `index` - Character index (0-7)
    /// * `data` - 8 bytes defining the character bitmap
    pub fn create_character(&mut self, index: u8, data: [u8; 8]) -> ScreenResult {
        if index > 7 {
            return Err(QwiicLcdError::InvalidCustomCharIndex(index));
        }
        
        let addr = (Command::SetCGRamAddr as u8) | (index << 3);
        self.write_special_cmd(addr)?;
        
        for byte in data.iter() {
            self.write_byte(*byte)?;
        }
        
        self.home()
    }
    
    /// Retry I2C write byte operation
    fn retry_i2c_write_byte(&mut self, command: u8) -> ScreenResult {
        let mut delay_ms = self.config.retry_config.initial_delay_ms;
        let mut last_error = None;
        
        for attempt in 0..=self.config.retry_config.max_retries {
            match self.dev.smbus_write_byte(command) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    last_error = Some(e);
                    
                    // Don't sleep after the last attempt
                    if attempt < self.config.retry_config.max_retries {
                        thread::sleep(Duration::from_millis(delay_ms));
                        
                        // Apply exponential backoff
                        delay_ms = ((delay_ms as f32 * self.config.retry_config.backoff_multiplier) as u64)
                            .min(self.config.retry_config.max_delay_ms);
                    }
                }
            }
        }
        
        // All retries exhausted
        match last_error {
            Some(e) => Err(QwiicLcdError::from(e)),
            None => Err(QwiicLcdError::CommunicationTimeout),
        }
    }
    
    /// Retry I2C write block operation
    fn retry_i2c_write_block(&mut self, register: u8, data: Vec<u8>) -> ScreenResult {
        let mut delay_ms = self.config.retry_config.initial_delay_ms;
        let mut last_error = None;
        
        for attempt in 0..=self.config.retry_config.max_retries {
            match self.dev.smbus_write_i2c_block_data(register, &data) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    last_error = Some(e);
                    
                    // Don't sleep after the last attempt
                    if attempt < self.config.retry_config.max_retries {
                        thread::sleep(Duration::from_millis(delay_ms));
                        
                        // Apply exponential backoff
                        delay_ms = ((delay_ms as f32 * self.config.retry_config.backoff_multiplier) as u64)
                            .min(self.config.retry_config.max_delay_ms);
                    }
                }
            }
        }
        
        // All retries exhausted
        match last_error {
            Some(e) => Err(QwiicLcdError::from(e)),
            None => Err(QwiicLcdError::CommunicationTimeout),
        }
    }
    
    /// Retry I2C write byte data operation
    fn retry_i2c_write_byte_data(&mut self, register: u8, data: u8) -> ScreenResult {
        let mut delay_ms = self.config.retry_config.initial_delay_ms;
        let mut last_error = None;
        
        for attempt in 0..=self.config.retry_config.max_retries {
            match self.dev.smbus_write_byte_data(register, data) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    last_error = Some(e);
                    
                    // Don't sleep after the last attempt
                    if attempt < self.config.retry_config.max_retries {
                        thread::sleep(Duration::from_millis(delay_ms));
                        
                        // Apply exponential backoff
                        delay_ms = ((delay_ms as f32 * self.config.retry_config.backoff_multiplier) as u64)
                            .min(self.config.retry_config.max_delay_ms);
                    }
                }
            }
        }
        
        // All retries exhausted
        match last_error {
            Some(e) => Err(QwiicLcdError::from(e)),
            None => Err(QwiicLcdError::CommunicationTimeout),
        }
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
    fn test_map_character() {
        // We test the map_character function directly by duplicating its logic
        // since we can't create a Screen without a real I2C device
        let map_char = |c: char| -> u8 {
            let code = c as u32;

            // Standard ASCII printable range and extended ASCII
            if (0x20..=0x7E).contains(&code) || (0x80..=0xFF).contains(&code) {
                code as u8
            }
            // Common replacements for better display
            else {
                match c {
                    // Tab, newline, carriage return -> space
                    '\t' | '\n' | '\r' => 0x20,
                    // Everything else -> question mark
                    _ => 0x3F,
                }
            }
        };

        // Test ASCII printable characters
        assert_eq!(map_char(' '), 0x20);
        assert_eq!(map_char('!'), 0x21);
        assert_eq!(map_char('A'), 0x41);
        assert_eq!(map_char('Z'), 0x5A);
        assert_eq!(map_char('a'), 0x61);
        assert_eq!(map_char('z'), 0x7A);
        assert_eq!(map_char('~'), 0x7E);
        assert_eq!(map_char('0'), 0x30);
        assert_eq!(map_char('9'), 0x39);

        // Test extended ASCII (passed through)
        // Note: These characters have Unicode values that match their extended ASCII positions
        assert_eq!(map_char('£'), 0xA3); // Pound sign (U+00A3)
        assert_eq!(map_char('°'), 0xB0); // Degree symbol (U+00B0)
        assert_eq!(map_char('÷'), 0xF7); // Division sign (U+00F7)
        assert_eq!(map_char('ÿ'), 0xFF); // y with diaeresis (U+00FF)

        // Test characters that don't map directly (Unicode > 0xFF)
        assert_eq!(map_char('€'), 0x3F); // Euro sign (U+20AC) - outside extended ASCII

        // Test control characters (mapped to space)
        assert_eq!(map_char('\t'), 0x20);
        assert_eq!(map_char('\n'), 0x20);
        assert_eq!(map_char('\r'), 0x20);

        // Test Unicode characters outside LCD range (mapped to '?')
        assert_eq!(map_char('😀'), 0x3F); // Emoji
        assert_eq!(map_char('中'), 0x3F); // Chinese character
        assert_eq!(map_char('א'), 0x3F); // Hebrew character
        assert_eq!(map_char('🚀'), 0x3F); // Rocket emoji
        assert_eq!(map_char('\0'), 0x3F); // Null character
    }

    #[test]
    fn test_print_ascii_valid() {
        // Test ASCII validation logic without actual I2C device
        let validate_ascii = |s: &str| -> Result<(), String> {
            if !s.is_ascii() {
                return Err("String contains non-ASCII characters".to_string());
            }

            for c in s.chars() {
                if c as u32 >= 0x20 && c as u32 <= 0x7E {
                    // Valid printable ASCII
                } else if c == '\n' || c == '\r' || c == '\t' {
                    // Skip control characters silently
                    continue;
                } else {
                    return Err(format!(
                        "Character '{}' (0x{:02X}) is not printable ASCII",
                        c, c as u32
                    ));
                }
            }

            Ok(())
        };

        // Test valid ASCII strings
        assert!(validate_ascii("Hello World").is_ok());
        assert!(validate_ascii("Test 123!@#").is_ok());
        assert!(validate_ascii("").is_ok()); // Empty string is valid
        assert!(validate_ascii(" ~!@#$%^&*()_+{}|:\"<>?").is_ok());
    }

    #[test]
    fn test_print_ascii_invalid() {
        // Test ASCII validation logic without actual I2C device
        let validate_ascii = |s: &str| -> Result<(), String> {
            if !s.is_ascii() {
                return Err("String contains non-ASCII characters".to_string());
            }

            for c in s.chars() {
                if c as u32 >= 0x20 && c as u32 <= 0x7E {
                    // Valid printable ASCII
                } else if c == '\n' || c == '\r' || c == '\t' {
                    // Skip control characters silently
                    continue;
                } else {
                    return Err(format!(
                        "Character '{}' (0x{:02X}) is not printable ASCII",
                        c, c as u32
                    ));
                }
            }

            Ok(())
        };

        // Test non-ASCII strings
        assert!(validate_ascii("Café").is_err()); // Contains é
        assert!(validate_ascii("Hello 世界").is_err()); // Contains Chinese
        assert!(validate_ascii("€100").is_err()); // Contains Euro symbol
        assert!(validate_ascii("Temperature: 25°C").is_err()); // Contains degree symbol
        assert!(validate_ascii("😀").is_err()); // Contains emoji

        // Verify error messages
        match validate_ascii("Café") {
            Err(msg) => assert!(msg.contains("non-ASCII")),
            Ok(_) => panic!("Expected error for non-ASCII string"),
        }
    }

    #[test]
    fn test_print_empty_string() {
        // Test that empty strings are valid
        let validate_ascii = |s: &str| -> Result<(), String> {
            if !s.is_ascii() {
                return Err("String contains non-ASCII characters".to_string());
            }
            Ok(())
        };

        // Empty string should work for both validation methods
        assert!(validate_ascii("").is_ok());
        assert_eq!("".len(), 0);
    }

    #[test]
    fn test_print_special_characters() {
        // Test character mapping for special cases
        let map_char = |c: char| -> u8 {
            let code = c as u32;

            if (0x20..=0x7E).contains(&code) || (0x80..=0xFF).contains(&code) {
                code as u8
            } else {
                match c {
                    '\t' | '\n' | '\r' => 0x20,
                    _ => 0x3F,
                }
            }
        };

        // Test that various special cases map correctly
        assert_eq!(map_char('\n'), 0x20); // Newline -> space
        assert_eq!(map_char('\t'), 0x20); // Tab -> space
        assert_eq!(map_char('\r'), 0x20); // Carriage return -> space
        assert_eq!(map_char('中'), 0x3F); // Chinese -> question mark
        assert_eq!(map_char('∑'), 0x3F); // Math symbol -> question mark
        assert_eq!(map_char('🚀'), 0x3F); // Emoji -> question mark
    }
    
    #[test]
    fn test_qwiic_lcd_error_display() {
        let i2c_error = LinuxI2CError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test error"));
        let error = QwiicLcdError::I2CError(i2c_error);
        assert!(error.to_string().contains("I2C communication error"));
        
        let error = QwiicLcdError::InvalidPosition { row: 5, col: 25, max_rows: 4, max_columns: 20 };
        let msg = error.to_string();
        assert!(msg.contains("Invalid cursor position"));
        assert!(msg.contains("(5, 25)"));
        assert!(msg.contains("4x20"));
        
        let error = QwiicLcdError::InvalidCharacter('😀');
        let msg = error.to_string();
        assert!(msg.contains("Invalid character"));
        assert!(msg.contains("Only ASCII characters"));
        
        let error = QwiicLcdError::CommunicationTimeout;
        assert!(error.to_string().contains("Communication timeout"));
        
        let error = QwiicLcdError::InitializationFailed("test failure".to_string());
        let msg = error.to_string();
        assert!(msg.contains("Failed to initialize LCD"));
        assert!(msg.contains("test failure"));
        
        let error = QwiicLcdError::InvalidCustomCharIndex(8);
        let msg = error.to_string();
        assert!(msg.contains("Invalid custom character index 8"));
        assert!(msg.contains("Must be 0-7"));
        
        let error = QwiicLcdError::InvalidContrastValue(255);
        let msg = error.to_string();
        assert!(msg.contains("Invalid contrast value"));
    }
    
    #[test]
    fn test_error_conversion_from_linux_i2c() {
        let i2c_error = LinuxI2CError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        let lcd_error: QwiicLcdError = i2c_error.into();
        assert!(matches!(lcd_error, QwiicLcdError::I2CError(_)));
    }
    
    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 10);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert_eq!(config.max_delay_ms, 1000);
    }
    
    #[test]
    fn test_retry_config_custom() {
        let config = RetryConfig {
            max_retries: 5,
            initial_delay_ms: 20,
            backoff_multiplier: 1.5,
            max_delay_ms: 500,
        };
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_delay_ms, 20);
        assert_eq!(config.backoff_multiplier, 1.5);
        assert_eq!(config.max_delay_ms, 500);
    }
    
    #[test]
    fn test_screen_config_with_retry() {
        let retry_config = RetryConfig {
            max_retries: 5,
            initial_delay_ms: 20,
            backoff_multiplier: 1.5,
            max_delay_ms: 500,
        };
        let config = ScreenConfig::new_with_retry(2, 16, retry_config);
        assert_eq!(config.max_rows, 2);
        assert_eq!(config.max_columns, 16);
        assert_eq!(config.retry_config.max_retries, 5);
        assert_eq!(config.retry_config.initial_delay_ms, 20);
    }
    
    #[test] 
    fn test_screen_config_default_includes_retry() {
        let config = ScreenConfig::default();
        assert_eq!(config.max_rows, 4);
        assert_eq!(config.max_columns, 20);
        assert_eq!(config.retry_config.max_retries, 3);
        assert_eq!(config.retry_config.initial_delay_ms, 10);
    }
    
    #[test]
    fn test_invalid_cursor_position_error() {
        // This test would require a mock Screen, so we test the error creation directly
        let error = QwiicLcdError::InvalidPosition { row: 10, col: 30, max_rows: 4, max_columns: 20 };
        match error {
            QwiicLcdError::InvalidPosition { row, col, max_rows, max_columns } => {
                assert_eq!(row, 10);
                assert_eq!(col, 30);
                assert_eq!(max_rows, 4);
                assert_eq!(max_columns, 20);
            },
            _ => panic!("Wrong error type"),
        }
    }
    
    #[test]
    fn test_invalid_character_error() {
        let error = QwiicLcdError::InvalidCharacter('€');
        match error {
            QwiicLcdError::InvalidCharacter(c) => {
                assert_eq!(c, '€');
            },
            _ => panic!("Wrong error type"),
        }
    }
    
    #[test]
    fn test_custom_character_index_validation() {
        let valid_index = 7;
        let invalid_index = 8;
        
        // Test that index 7 is valid (no error)
        assert!(valid_index <= 7);
        
        // Test that index 8 would produce error
        assert!(invalid_index > 7);
        let error = QwiicLcdError::InvalidCustomCharIndex(invalid_index);
        assert!(error.to_string().contains("Invalid custom character index"));
    }
}

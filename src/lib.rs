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


// commands
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
    SpecialCommand = 254
}

// Display entry mode
#[derive(Copy, Clone)]
pub enum EntryMode {
    Right = 0x00,
    Left = 0x02,
}

#[derive(Copy, Clone)]
pub enum EntryShift {
    Increment = 0x01,
    Decrement = 0x00,
}

// Flags for display on/off control

#[derive(Copy, Clone)]
pub enum DisplayStatus {
    Off = 0x00,
    On = 0x04,
}

#[derive(Copy, Clone)]
pub enum CursorState {
    Off = 0x00,
    On = 0x02,
}

#[derive(Copy, Clone)]
pub enum BlinkState {
    Off = 0x00,
    On = 0x01,
}

// Flags for display/cursor shift

#[derive(Copy, Clone)]
pub enum MoveType {
    Cursor = 0x00,
    Display = 0x08,
}

#[derive(Copy, Clone)]
pub enum MoveDirection {
    Left = 0x00,
    Right = 0x04,
}

#[derive(Copy, Clone)]
pub enum Backlight {
    Off = 0x00,
    On = 0x04,
}

// Specific flags
#[derive(Copy, Clone)]
pub enum WriteMode {
    Enable = 0x04,
    ReadWrite = 0x02,
    RegisterSelect = 0x01,
    Normal = 0x00,
}

// Configuration

#[derive(Copy, Clone)]
pub enum BitMode {
    B4 = 0x00,
    B8 = 0x10,
}

pub struct ScreenConfig {
    max_rows: u8,
    max_columns: u8
}

impl ScreenConfig {
    pub fn new(max_rows: u8, max_columns: u8) -> ScreenConfig {
        ScreenConfig {
            max_rows: 4,
            max_columns: 20
        }
    }

    pub fn default() -> ScreenConfig {
        ScreenConfig::new(4, 20)
    }
}

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

    fn default() -> DisplayState {
        DisplayState::new(DisplayStatus::On, CursorState::On, BlinkState::On)
    }
}

// Screen
pub struct Screen {
    dev: LinuxI2CDevice,
    config: ScreenConfig,
    state: DisplayState,
}

type ScreenResult = Result<(), LinuxI2CError>;

impl Screen {
    pub fn new(config: ScreenConfig, bus: &str, i2c_addr: u16) -> Result<Screen, LinuxI2CError> {
        let dev = LinuxI2CDevice::new(bus, i2c_addr)?;
        Ok(Screen {
               dev,
               config,
               state: DisplayState::default(),
           })
    }

    pub fn init(&mut self) -> ScreenResult {
        self.apply_display_state()?;
        self.clear()?;
        self.enable_blink(false)?;
        self.enable_cursor(false)?;
  
        // Wait for the screen to set up
        thread::sleep(Duration::from_millis(200));

        Ok(())
    }

    pub fn change_backlight(&mut self, r: u8, g: u8, b: u8) -> ScreenResult {
        let mut block = vec![0, 1, 2, 3];

        block[0] = Command::SetRGB as u8;
        block[1] = r;
        block[2] = g;
        block[3] = b;

        self.write_block((Command::SettingCommand as u8), block)
    }

    pub fn clear(&mut self) -> ScreenResult {
        self.write_setting_cmd(Command::ClearDisplay as u8);
        self.home()
    }

    pub fn home(&mut self) -> ScreenResult {
        self.write_special_cmd(Command::ReturnHome as u8)
    }

    pub fn move_cursor(&mut self, row: usize, col: usize) -> ScreenResult {
        let row_offsets: Vec<usize> = vec![0x00, 0x40, 0x14, 0x54];

        if row > self.config.max_rows.into() || row < 0 {
            return self.apply_display_state();
        }
        if col > self.config.max_columns.into() || col < 0 {
            return self.apply_display_state();
        }

        let command = ((Command::SetDDRamAddr as u8) | ((col + row_offsets[row]) as u8));

        self.write_special_cmd(command as u8)
    }

    pub fn enable_cursor(&mut self, activated: bool) -> ScreenResult {
        self.state.cursor = match activated {
            true => CursorState::On,
            false => CursorState::Off,
        };

        self.apply_display_state()
    }

    pub fn enable_display(&mut self, activated: bool) -> ScreenResult {
        self.state.status = match activated {
            true => DisplayStatus::On,
            false => DisplayStatus::Off,
        };

        self.apply_display_state()
    }

    pub fn enable_blink(&mut self, activated: bool) -> ScreenResult {
        self.state.blink = match activated {
            true => BlinkState::On,
            false => BlinkState::Off,
        };

        self.apply_display_state()
    }

    pub fn apply_display_state(&mut self) -> ScreenResult {
        let mut flags = 0;

        flags = flags | (self.state.status as u8);
        flags = flags | (self.state.cursor as u8);
        flags = flags | (self.state.blink as u8);

        self.write_special_cmd((Command::DisplayControl as u8) | flags)
    }

    pub fn print(&mut self, s: &str) -> ScreenResult {
        for c in s.chars() {
            self.write_byte(c as u8)?;
        }

        Ok(())
    }

    pub fn write_byte(&mut self, command: u8) -> ScreenResult {
        self.dev.smbus_write_byte(command)?;
        thread::sleep(Duration::new(0, 10_000));
        Ok(())
    }

    pub fn write_block(&mut self, register: u8, data: Vec<u8>) -> ScreenResult {
        self.dev.smbus_write_i2c_block_data(register, &data)?;
        thread::sleep(Duration::new(0, 10_000));
        Ok(())
    }

    pub fn write_setting_cmd(&mut self, command: u8) -> ScreenResult {
        self.dev.smbus_write_byte_data((Command::SettingCommand as u8), command)?;
        thread::sleep(Duration::new(0, 10_000));
        Ok(())
    }
    
    pub fn write_special_cmd(&mut self, command: u8) -> ScreenResult {
        self.dev.smbus_write_byte_data((Command::SpecialCommand as u8), command)?;
        thread::sleep(Duration::new(0, 10_000));
        Ok(())
    }
}

pub fn map(x: usize, in_min: usize, in_max: usize, out_min: usize, out_max: usize) -> usize {
    return usize::from(((x-in_min) * (out_max-out_min) / (in_max-in_min) + out_min));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let _config = ScreenConfig::default();

        let config = ScreenConfig::default();
        let mut screen = Screen::new(config, "/dev/i2c-1", 0x72).expect("Could not init device");
    
        screen.change_backlight(255, 255, 255).unwrap();
        screen.home().unwrap();
        screen.move_cursor(0,0).unwrap();
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
        screen.move_cursor(0,0).unwrap();
        screen.enable_blink(false).unwrap();
        screen.enable_blink(true).unwrap();
        screen.clear().unwrap();
        screen.print("It Works!").unwrap();
        thread::sleep(Duration::from_secs(1));
    }
}



// Copyright 2017 Romain Porte
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

#[derive(Copy, Clone)]
pub enum LineCount {
    L1 = 0x00,
    L2 = 0x08,
}

#[derive(Copy, Clone)]
pub enum MatrixSize {
    M5x8 = 0x00,
    M5x10 = 0x04
}

pub struct ScreenConfig {
    bit_mode: BitMode,
    line_count: LineCount,
    matrix_size: MatrixSize,
    max_rows: u8,
    max_columns: u8
}

impl ScreenConfig {
    pub fn new(bit_mode: BitMode, line_count: LineCount, matrix_size: MatrixSize, max_rows: u8, max_columns: u8) -> ScreenConfig {
        ScreenConfig {
            bit_mode,
            line_count,
            matrix_size,
            max_rows: 4,
            max_columns: 20
        }
    }

    pub fn default() -> ScreenConfig {
        ScreenConfig::new(BitMode::B8, LineCount::L2, MatrixSize::M5x8, 4, 20)
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
        self.write(0x03, WriteMode::Normal)?;
        self.write(0x03, WriteMode::Normal)?;
        self.write(0x03, WriteMode::Normal)?;
        self.write(0x02, WriteMode::Normal)?;

        self.install_function_set()?;

        self.apply_display_state()?;
        self.clear()?;
        // self.set_entry_mode(EntryMode::Left)?; // Allow users to change this?

        // Wait for the screen to set up
        thread::sleep(Duration::from_millis(200));

        Ok(())
    }

    // High-order commands mapped to methods

    pub fn install_function_set(&mut self) -> ScreenResult {
        let mut flags = 0;

        flags = flags | (self.config.bit_mode as u8);
        flags = flags | (self.config.line_count as u8);
        flags = flags | (self.config.matrix_size as u8);

        self.command(Command::FunctionSet, flags)
    }

    pub fn change_backlight(&mut self, r: u8, g: u8, b: u8) -> ScreenResult {
        let mut block = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

        self.set_status(false);

        let mut flags = 0;
        flags = flags | (self.state.status as u8);
        flags = flags | (self.state.cursor as u8);
        flags = flags | (self.state.blink as u8);

        let red = 128 + map(r.into(), 0, 255, 0, 29) as u8;
        let green = 128 + map(g.into(), 0, 255, 0, 29) as u8;
        let blue = 188 + map(b.into(), 0, 255, 0, 29) as u8;
     
        block[0] = Command::SpecialCommand as u8;
        block[1] = ((Command::DisplayControl as u8) | flags);
        block[2] = Command::SettingCommand as u8;
        block[3] = red;
        block[4] = Command::SettingCommand as u8;
        block[5] = green;
        block[6] = Command::SettingCommand as u8;
        block[7] = blue;
     
        block[8] = Command::SpecialCommand as u8;
        block[9] = ((Command::DisplayControl as u8) | flags);
        self.write_block((Command::SettingCommand as u8), block);
        self.set_status(true)
    }


    // Working
    pub fn clear(&mut self) -> ScreenResult {
        self.write_setting_cmd(Command::ClearDisplay as u8)
    }

    // Working
    pub fn home(&mut self) -> ScreenResult {
        self.write_special_cmd(Command::ReturnHome as u8)
    }

    // pub fn set_entry_mode(&mut self, entry_mode: EntryMode) -> ScreenResult {
    //     self.command(Command::EntryModeSet, entry_mode as u8)
    // }

    // Working
    // TODO: Patch min/max barrier
    pub fn move_cursor(&mut self, row: usize, col: usize) -> ScreenResult {
        // self.state.cursor = match activated {
        //     true => CursorState::On,
        //     false => CursorState::Off,
        // };

        let row_offsets: Vec<usize> = vec![0x00, 0x40, 0x14, 0x54];

    
        // row = row_offsets.iter().max().unwrap();
        // row = row_offsets.iter().min().unwrap();
        let command = ((Command::SetDDRamAddr as u8) | ((col + row_offsets[row]) as u8));

        self.write_special_cmd(command as u8)
    }

    // Working
    pub fn set_cursor(&mut self, activated: bool) -> ScreenResult {
        self.state.cursor = match activated {
            true => CursorState::On,
            false => CursorState::Off,
        };

        self.apply_display_state()
    }

    // Working
    pub fn set_status(&mut self, activated: bool) -> ScreenResult {
        self.state.status = match activated {
            true => DisplayStatus::On,
            false => DisplayStatus::Off,
        };

        self.apply_display_state()
    }

    // Working
    pub fn set_blink(&mut self, activated: bool) -> ScreenResult {
        self.state.blink = match activated {
            true => BlinkState::On,
            false => BlinkState::Off,
        };

        self.apply_display_state()
    }

    // Working
    // Stateless so every argument is needed
    pub fn apply_display_state(&mut self) -> ScreenResult {
        let mut flags = 0;

        flags = flags | (self.state.status as u8);
        flags = flags | (self.state.cursor as u8);
        flags = flags | (self.state.blink as u8);

        self.write_special_cmd((Command::DisplayControl as u8) | flags)
    }

    // Other methods that are not commands

    pub fn set_backlight(&mut self, backlight: bool) -> ScreenResult {
        if backlight {
            self.write_cmd(Backlight::On as u8)
        } else {
            self.write_cmd(Backlight::Off as u8)
        }
    }

    pub fn display(&mut self, s: &str, line: u8, col: u8) -> ScreenResult {
        // let pos = match line {
        //     1 => 0x00 + col,
        //     2 => 0x40 + col,
        //     3 => 0x14 + col,
        //     4 => 0x54 + col,
        //     _ => col,
        // };
        // self.write_cmd(pos)?;

        for c in s.chars() {
            self.write_byte(c as u8)?;
        }

        Ok(())
    }

    pub fn print(&mut self, s: &str) -> ScreenResult {
        // let pos = match line {
        //     1 => 0x00 + col,
        //     2 => 0x40 + col,
        //     3 => 0x14 + col,
        //     4 => 0x54 + col,
        //     _ => col,
        // };
        // self.write_cmd(pos)?;

        for c in s.chars() {
            self.write_byte(c as u8)?;
        }

        Ok(())
    }

    // Lower-level methods that write commands to device, ordered from higher
    // to lower level of abstraction

    pub fn command(&mut self, command: Command, data: u8) -> ScreenResult {
        self.write((command as u8), WriteMode::Normal)


    }

    pub fn write_char(&mut self, ch: u8) -> ScreenResult {
        self.write(ch, WriteMode::RegisterSelect)
    }

    pub fn write_four_bytes(&mut self, data: u8) -> ScreenResult {
        self.write_screen(data)?;
        self.strobe(data)?;
        Ok(())
    }

    pub fn write(&mut self, command: u8, mode: WriteMode) -> ScreenResult {
        // match self.config.bit_mode {
        //     BitMode::B4 => {
        //         self.write_four_bytes((mode as u8) | (command & 0xF0))?;
        //         self.write_four_bytes((mode as u8) | ((command << 4) & 0xF0))?;
        //         Ok(())
        //     },
        //     BitMode::B8 => {
                self.write_screen(command)?; // Not sure here for mode
                Ok(())
        //     }
        // }
    }

    pub fn strobe(&mut self, data: u8) -> ScreenResult {
        // Set enable bit
        self.write_screen(data | (WriteMode::Enable as u8))?;
        thread::sleep(Duration::new(0, 50_000));

        // Unset enable bit
        self.write_screen(data & !(WriteMode::Enable as u8))?;
        thread::sleep(Duration::new(0, 10_000));

        Ok(())
    }

    pub fn write_screen(&mut self, command: u8) -> ScreenResult {
        self.write_cmd(command)
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

    pub fn write_cmd(&mut self, command: u8) -> ScreenResult {
        self.dev.smbus_write_byte(command)?;
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
    return usize::from(((((x - in_min)*(out_max - out_min))/(in_max - in_min)) + out_min));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let _config = ScreenConfig::default();
    }
}



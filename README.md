# QwiicLCD I2C screen library for Rust

## Description

This library aims at controlling QwiicLCD screens using I2C from Linux. It
primary target is ARM devices such as RaspberryPi or FriendlyARM's NanoPi Neo.
It should nonetheless work on other Linux distributions with access to an I2C
bus.

## How to use library

Add the following line to your cargo.toml:
```
qwiic-lcd-rs = { git = "https://github.com/PixelCoda/QwiicLCD-Rust.git", version = "*" }
```

Example: 
```rust
extern crate qwiic_lcd_rs;

use qwiic_lcd_rs::*;
use std::thread;
use std::time::Duration;

fn main() {
    // Default LCDSize is 4x20
    let mut config = ScreenConfig::default();

    // Uncomment and modify the values below to use different screen sizes
    // config.max_rows = 2;
    // config.max_columns = 16;

    // Default Qwiic address is 0x72
    let mut screen = Screen::new(config, "/dev/i2c-1", 0x72).expect("Could not init device");

    // Set backlight to bright white
    screen.change_backlight(255, 255, 255).unwrap();

    // Clear the screen
    screen.clear().unwrap();
    
    // Move the cursor to 0,0
    screen.move_cursor(0,0).unwrap();

    // Print text
    screen.print("Hello from Rust!").unwrap();

    // Move to the next line
    screen.move_cursor(1,0).unwrap();

    // Print text
    screen.print("It works! :)").unwrap();
}
```

## References

* https://github.com/sparkfun/Qwiic_SerLCD_Py/blob/main/qwiic_serlcd.py
* https://github.com/MicroJoe/rust-i2c-16x2/blob/master/src/lib.rs

## License

Released under Apache 2.0.

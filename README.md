# QwiicLCD I2C screen library for Rust

## Description

This library aims at controlling QwiicLCD screens using I2C from Linux. It
primary target is ARM devices such as RaspberryPi or FriendlyARM's NanoPi Neo.
It should nonetheless work on other Linux distributions with access to an I2C
bus.

Currently I only have access to the 20x4 LCD for testing purposes. If you have issues with other Qwiic LCDs please submit an issue or a pull request.

## How to use library

Add the following line to your cargo.toml:
```
qwiic-lcd-rs = "0.1.11"
```

Or for the most recent commit on the master branch use:
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

    // Set backlight to green and wait 1 second
    screen.change_backlight(0, 255, 0).unwrap();
    thread::sleep(Duration::from_secs(1));

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

# Support and follow my work by:

#### Buying my dope NTFs:
 * https://opensea.io/accounts/PixelCoda

#### Checking out my Github:
 * https://github.com/PixelCoda

#### Following my facebook page:
 * https://www.facebook.com/pixelcoda/

#### Subscribing to my Patreon:
 * https://www.patreon.com/calebsmith_pixelcoda

#### Or donating crypto:
 * ADA: addr1qyp299a45tgvveh83tcxlf7ds3yaeh969yt3v882lvxfkkv4e0f46qvr4wzj8ty5c05jyffzq8a9pfwz9dl6m0raac7s4rac48
 * ALGO: VQ5EK4GA3IUTGSPNGV64UANBUVFAIVBXVL5UUCNZSDH544XIMF7BAHEDM4
 * ATOM: cosmos1wm7lummcealk0fxn3x9tm8hg7xsyuz06ul5fw9
 * BTC: bc1qh5p3rff4vxnv23vg0hw8pf3gmz3qgc029cekxz
 * ETH: 0x7A66beaebF7D0d17598d37525e63f524CfD23452
 * ERC20: 0x7A66beaebF7D0d17598d37525e63f524CfD23452
 * XLM: GCJAUMCO2L7PTYMXELQ6GHBTF25MCQKEBNSND2C4QMUPTSVCPEN3LCOG
 * XTZ: tz1SgJppPn56whprsDDGcqR4fxqCr2PXvg1R
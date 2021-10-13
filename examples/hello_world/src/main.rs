

extern crate qwiic_lcd_rs;

use qwiic_lcd_rs::*;
use std::thread;
use std::time::Duration;

fn main() {
    let config = ScreenConfig::default();
    let mut screen = Screen::new(config, "/dev/i2c-1", 0x72).expect("Could not init device");

  
    screen.change_backlight(255, 255, 255).unwrap();
    screen.home().unwrap();
    screen.enable_blink(false).unwrap();
    screen.enable_blink(true).unwrap();
    screen.clear().unwrap();
    
    screen.move_cursor(0,0).unwrap();
    screen.print("Hello Rust!").unwrap();

    screen.move_cursor(1,0).unwrap();
    screen.print("It works! :)").unwrap();
    thread::sleep(Duration::from_secs(2));
}

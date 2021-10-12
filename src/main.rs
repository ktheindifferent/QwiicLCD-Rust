extern crate qwiic_lcd_rs;

use qwiic_lcd_rs::*;
use std::thread;
use std::time::Duration;

// 16x2: 0x3f
// 20x4: 0x27

fn main() {
    let config = ScreenConfig::default();
    let mut screen = Screen::new(config, "/dev/i2c-1", 0x72).expect("Could not init device");

    // println!("init");
    // screen.init().unwrap();


    println!("clear");
    screen.clear().unwrap();
    thread::sleep(Duration::from_secs(5));

    println!("home");
    screen.home().unwrap();
    thread::sleep(Duration::from_secs(5));
    
    println!("move_cursor");
    screen.move_cursor(2,2).unwrap();
    thread::sleep(Duration::from_secs(5));
    

    // println!("off");
    // screen.set_backlight(false).unwrap();
    // thread::sleep(Duration::from_secs(5));
    
    // println!("on");
    // screen.set_backlight(true).unwrap();
    // thread::sleep(Duration::from_secs(5));
    
    println!("show some text");
    screen.display("Hello Rust!", 1, 0).unwrap();
    screen.display("Fuck yeah :)", 2, 0).unwrap();
    thread::sleep(Duration::from_secs(5));
    
    // println!("off");
    // screen.set_backlight(false).unwrap();
    // thread::sleep(Duration::from_secs(1));
}

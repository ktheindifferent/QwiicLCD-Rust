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


    // println!("clear");
    // screen.clear().unwrap();
    // thread::sleep(Duration::from_secs(5));

    // println!("enable_cursor");
    // screen.set_cursor(true).unwrap();
    // thread::sleep(Duration::from_secs(5));
    
    // println!("disable_cursor");
    // screen.set_cursor(false).unwrap();
    // thread::sleep(Duration::from_secs(5));

    // println!("enable_cursor");
    // screen.set_cursor(true).unwrap();
    // thread::sleep(Duration::from_secs(5));

    // println!("enable_blink");
    // screen.set_blink(true).unwrap();
    // thread::sleep(Duration::from_secs(5));

    // println!("display_off");
    // screen.set_status(false).unwrap();
    // thread::sleep(Duration::from_secs(5));

    // println!("display_on");
    // screen.set_status(true).unwrap();
    // thread::sleep(Duration::from_secs(5));

    // println!("home");
    // screen.home().unwrap();
    // thread::sleep(Duration::from_secs(5));
    
    // println!("move_cursor");
    // screen.move_cursor(2,2).unwrap();
    // thread::sleep(Duration::from_secs(5));
    

    screen.home().unwrap();
    screen.move_cursor(0,0).unwrap();
    screen.set_blink(false).unwrap();
    screen.set_cursor(false).unwrap();
    screen.clear().unwrap();

    // println!("off");
    // screen.set_backlight(false).unwrap();
    // thread::sleep(Duration::from_secs(5));
    
    // println!("on");
    // screen.set_backlight(true).unwrap();
    // thread::sleep(Duration::from_secs(5));
    
    println!("show some text");
    screen.display("Hello Rust!", 1, 0).unwrap();
    screen.display("Fuck yeah :)", 3, 0).unwrap();
    thread::sleep(Duration::from_secs(5));
    
    // println!("off");
    // screen.set_backlight(false).unwrap();
    // thread::sleep(Duration::from_secs(1));
}

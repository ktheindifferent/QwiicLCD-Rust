// Copyright 2021 Caleb Mitchell Smith-Woolrich (PixelCoda)
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
    
    screen.change_backlight(255, 255, 255).unwrap();
    screen.home().unwrap();
    screen.move_cursor(0,0).unwrap();
    screen.enable_blink(false).unwrap();
    screen.enable_blink(true).unwrap();
    screen.clear().unwrap();

    // println!("off");
    // screen.set_backlight(false).unwrap();
    // thread::sleep(Duration::from_secs(5));
    
    // println!("on");
    // screen.set_backlight(true).unwrap();
    // thread::sleep(Duration::from_secs(5));
    

    screen.print("Hello Rust!").unwrap();
    screen.move_cursor(1,0).unwrap();
    screen.print("It works! :)").unwrap();
    thread::sleep(Duration::from_secs(5));

    screen.change_backlight(255, 0, 255).unwrap();
    thread::sleep(Duration::from_secs(5));
    screen.change_backlight(0, 255, 255).unwrap();
    thread::sleep(Duration::from_secs(5));
    screen.change_backlight(255, 255, 0).unwrap();
    // println!("off");
    // screen.set_backlight(false).unwrap();
    // thread::sleep(Duration::from_secs(1));
}

# QwiicLCD I2C screen library for Rust

References used:

https://github.com/sparkfun/Qwiic_SerLCD_Py/blob/main/qwiic_serlcd.py

## Description

This library aims at controlling QwiicLCD screens using I2C from Linux. It
primary target is ARM devices such as RaspberryPi or FriendlyARM's NanoPi Neo.
It should nonetheless work on other Linux distributions with access to an I2C
bus.

Currently I only have access to the 20x4 LCD for testing purposes. If you have issues with other LCD sizes please submit an issue or a pull request.

## Building for Raspberry Pi

First setup your Rust cross compilation using the
[rust-cross](https://github.com/japaric/rust-cross) guide.

If you are using Archlinux like me you want to install
[arm-linux-gnueabihf-gcc](https://aur.archlinux.org/packages/arm-linux-gnueabihf-gcc/)
from AUR.

Then you should be good with the following commands

    $ cargo build --target=arm-unknown-linux-gnueabihf
    $ scp target/arm-unknown-linux-gnueabihf/debug/i2c-16x2 pi@raspberrypi.local:screen
    $ ssh pi@raspberrypi.local
    pi@raspberry$ ./screen

## License

Released under Apache 2.0.

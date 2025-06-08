#![no_std]
#![no_main]

use radian_core::prelude::*;

pub fn driver_main() {
    kprint!("hello driver world!\r\n");
    loop {}
}

#![no_std]
#![no_main]

use core::arch::asm;

use radian_core::prelude::*;

pub fn driver_main() {
    kprint!("hello driver world!");
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

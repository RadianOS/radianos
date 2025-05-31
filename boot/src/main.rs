#![no_main]
#![no_std]

//use core::panic::PanicInfo;

use consts::VERSION;
use log::info;
use uefi::prelude::*;

pub mod consts;


#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    info!("RadianOS Bootloader v{}", VERSION);

    Status::SUCCESS

}

//#[panic_handler]
//fn panic(info: &PanicInfo) -> ! {
//    loop {}
//}

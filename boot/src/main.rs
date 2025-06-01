#![no_main]
#![no_std]

extern crate alloc;

use core::arch::asm;

use log::info;
use uefi::{
    boot::{get_handle_for_protocol, open_protocol_exclusive},
    prelude::*,
    proto::console::
        text::Output
    ,
};

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    let handle = get_handle_for_protocol::<Output>().unwrap();
    let mut output = open_protocol_exclusive::<Output>(handle).unwrap();
    output.clear().expect("Failed to clear screen");

    info!("Booting Radian OS...");
    

    boot_system();

    Status::SUCCESS
}


pub fn boot_system() {
    // This will fail because we don't have a kernel yet lol
    // let (entry_point, kernel_entry) = load_kernel("\\EFI\\BOOT\\radiankernel");

    // info!("Kernel entry point: 0x{:x}", kernel_entry as usize);

    // You could use UEFI simple text output protocol here for debugging

    // info!("Jumping to kernel entry point at 0x{:x}", entry_point);

    // unsafe {
    //     kernel_entry();
    // }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
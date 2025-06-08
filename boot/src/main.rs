#![no_main]
#![no_std]
extern crate alloc;

use uefi::{
    boot::{get_handle_for_protocol, open_protocol_exclusive},
    prelude::*,
    proto::console::text::Output
};

#[macro_use]
mod serial;
mod fs;
mod kernel;

use kernel::load_kernel;

#[entry]
fn boot_efi_main() -> Status {
    uefi::helpers::init().unwrap();
    let handle = get_handle_for_protocol::<Output>().unwrap();
    let mut output = open_protocol_exclusive::<Output>(handle).unwrap();
    output.clear().expect("Failed to clear screen");
    boot_print!("Booting Radian OS...\r\n");
    // This will fail because we don't have a kernel yet lol
    let (entry_point, kernel_entry) = load_kernel("\\EFI\\BOOT\\KERNEL");
    boot_print!("Kernel entry point: 0x{:x}\r\n", kernel_entry as usize);
    // You could use UEFI simple text output protocol here for debugging
    boot_print!("Jumping to kernel entry point at 0x{:x}\r\n", entry_point);
    unsafe {
        kernel_entry();
    }
}

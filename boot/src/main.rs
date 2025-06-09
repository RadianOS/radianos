#![no_main]
#![no_std]
extern crate alloc;

use uefi::{
    boot::{MemoryAttribute, MemoryType, get_handle_for_protocol, open_protocol_exclusive},
    mem::memory_map::MemoryMap,
    prelude::*,
    proto::console::text::Output,
};

#[macro_use]
mod serial;
mod fs;
mod kernel;
use kernel::load_kernel;

#[repr(C)]
struct MemoryEntry {
    virt: u64,
    phys: u64,
    page_count: u64,
    attribute: u64,
    type_: u32,
}

#[entry]
fn boot_efi_main() -> Status {
    uefi::helpers::init().unwrap();
    let handle = get_handle_for_protocol::<Output>().unwrap();
    let mut output = open_protocol_exclusive::<Output>(handle).unwrap();
    output.clear().expect("Failed to clear screen");
    boot_print!("Booting \x1b[31mRadian OS\x1b[0m \x1b[32mv0.0.5\x1b[0m...\r\n");
    // This will fail because we don't have a kernel yet lol
    let (entry_point, kernel_entry) = load_kernel("\\EFI\\BOOT\\KERNEL");
    boot_print!("Kernel entry point: 0x{:x}\r\n", kernel_entry as usize);
    // You could use UEFI simple text output protocol here for debugging
    boot_print!("Jumping to kernel entry point at 0x{:x}\r\n", entry_point);

    let memory_map = uefi::boot::memory_map(MemoryType::LOADER_DATA).unwrap();

    let table =
        uefi::boot::allocate_pages(boot::AllocateType::AnyPages, MemoryType::LOADER_DATA, 1)
            .unwrap()
            .as_ptr();
    for (i, e) in memory_map.entries().enumerate() {
        unsafe {
            (table as *mut MemoryEntry).add(i).write(MemoryEntry {
                virt: e.virt_start,
                phys: e.phys_start,
                page_count: e.page_count,
                type_: core::mem::transmute::<MemoryType, u32>(e.ty),
                attribute: core::mem::transmute::<MemoryAttribute, u64>(e.att),
            });
        }
    }
    unsafe {
        core::arch::asm!(
            "call {0}",
            in(reg) kernel_entry,
            in("rsi") (table as *const MemoryEntry) as u64,
            in("rdi") (memory_map.entries().len()) as u64,
            options(noreturn)
        )
    }
}

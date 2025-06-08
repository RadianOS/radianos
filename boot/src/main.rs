#![no_main]
#![no_std]
extern crate alloc;

use core::fmt::{self, Write};
use uefi::{
    boot::{get_handle_for_protocol, open_protocol_exclusive},
    prelude::*,
    proto::console::text::Output
};
use uefi::boot::{self, AllocateType, MemoryType};
use xmas_elf::{program, sections::{self, SectionData}, ElfFile};
use alloc::vec::Vec;
use uefi::{boot::{ScopedProtocol}, fs::{FileSystem, FileSystemResult}, proto::media::fs::SimpleFileSystem, CString16};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// QEMU uses the standard COM1 serial port at 0x3F8
const SERIAL_PORT: u16 = 0x3F8;

/// Write a byte to the serial port.
pub fn serial_write_byte(byte: u8) {
    unsafe {
        //let mut line_status = Port::<u8>::new(SERIAL_PORT + 5);
        //while (line_status.read() & 0x20) == 0 {} // Wait until empty

        let mut line_status = 0u8;
        while line_status & 0x20 == 0 {
            core::arch::asm!(
                "in al, dx",
                out("al") line_status,
                in("dx") (SERIAL_PORT + 5)
            );
        }
        core::arch::asm!(
            "out dx, al",
            in("al") byte,
            in("dx") SERIAL_PORT
        );
    }
}

/// Write a string to the serial port.
pub fn serial_write_str(s: &str) {
    for byte in s.bytes() {
        serial_write_byte(byte);
    }
}

pub fn read_file(path: &str) -> FileSystemResult<Vec<u8>> {
    let path: CString16 = CString16::try_from(path).unwrap();
    let fs: ScopedProtocol<SimpleFileSystem> = boot::get_image_file_system(boot::image_handle()).unwrap();
    let mut fs = FileSystem::new(fs);
    fs.read(path.as_ref())
}

struct SerialWriter;
impl Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        serial_write_str(s);
        Ok(())
    }
}

pub fn log_write_fmt(args: fmt::Arguments) {
    let _ = SerialWriter.write_fmt(args);
}

#[macro_export]
macro_rules! boot_print {
    ($($arg:tt)*) => {{
        $crate::log_write_fmt(core::format_args!($($arg)*));
    }};
}

pub const KERNEL_BASE: u64 = 0;

type KernelFn = unsafe extern "C" fn() -> !;
pub fn load_kernel(file_path: &str) -> (usize, KernelFn) {
    let bytes = read_file(file_path).unwrap();
    let elf = ElfFile::new(&bytes).expect("Failed to parse ELF file");

    // for sh in elf.section_iter() {
    //     match sh.get_type().unwrap() {
    //         sections::ShType::Rela => {
    //             match sh.get_data(&elf).unwrap() {
    //                 SectionData::Rela64(rela) => {
    //                     for rela in rela.iter() {
    //                         match rela.get_type() {
    //                         }
    //                     }
    //                 }
    //                 _ => {}
    //             }
    //         }
    //         sections::ShType::Rel => boot_print!("rel"),
    //         _ => {}
    //     }
    // }

    for ph in elf.program_iter() {
        if ph.get_type().unwrap() == program::Type::Dynamic {
            boot_print!("Skipping dynamic segment");
        }
        if ph.get_type().expect("Failed to get header type") != program::Type::Load {
            continue;
        }

        let file_offset = ph.offset() as usize;
        let file_size = ph.file_size() as usize;
        let mem_size = ph.mem_size() as usize;
        let virt_addr = ph.virtual_addr() as usize;

        let aligned_virt_addr = virt_addr & !0xFFF;
        let page_offset = virt_addr - aligned_virt_addr;
        let total_size = page_offset + mem_size;
        let num_pages = total_size.div_ceil(0x1000);

        let mem_type = if ph.flags().is_execute() {
            MemoryType::LOADER_CODE
        } else {
            MemoryType::LOADER_DATA
        };
        boot_print!("Using {num_pages} pages, addr = {KERNEL_BASE:0x} + {virt_addr:0x}, align {aligned_virt_addr:0x} with type {:?}, {:0x}\r\n", mem_type, ph.physical_addr());
        let dest_ptr = boot::allocate_pages(
            AllocateType::Address(KERNEL_BASE + u64::try_from(aligned_virt_addr).unwrap()),
            mem_type,
            num_pages,
        )
        .expect("Failed to allocate pages")
        .as_ptr();
        unsafe {
            core::ptr::copy_nonoverlapping(
                bytes[file_offset..].as_ptr(),
                dest_ptr.add(page_offset),
                file_size,
            );
            if mem_size > file_size {
                core::ptr::write_bytes(dest_ptr.add(page_offset + file_size), 0, mem_size - file_size);
            }
        }
    }

    let entry_point = (KERNEL_BASE + elf.header.pt2.entry_point()) as usize;
    let kernel_entry: KernelFn = unsafe { core::mem::transmute(entry_point) };

    (entry_point, kernel_entry)
}

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

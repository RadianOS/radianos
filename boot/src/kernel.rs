// Kernel loading logic
use xmas_elf::{program, ElfFile};
use uefi::boot::{self, AllocateType, MemoryType};
use crate::fs::read_file;
use crate::{boot_print, MemoryEntry};

pub const KERNEL_BASE: u64 = 0;

pub type KernelFn = unsafe extern "C" fn() -> !;

pub fn load_kernel(file_path: &str) -> (usize, KernelFn) {
    let bytes = read_file(file_path).unwrap();
    let elf = ElfFile::new(&bytes).expect("Failed to parse ELF file");

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

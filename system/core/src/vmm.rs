use crate::{db, kprint, pmm};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page(u64);
impl Page {
    pub const FLAG_MASK: u64 = 0xfff;

    pub const PRESENT: u64 = 0x01;
    pub const READ_WRITE: u64 = 0x02;
    pub const USER_SUPERVISOR: u64 = 0x04; //shared
    pub const WRITE_THROUGH: u64 = 0x08;

    pub fn is_present(self) -> bool {
        self.0 & Page::PRESENT != 0
    }
    pub fn get_physaddr(self) -> u64 {
        self.0 & !Page::FLAG_MASK
    }
    /// Clear old flags and override with new ones
    pub fn override_flags(self, flags: u64) -> Self {
        Self((self.0 & !Page::FLAG_MASK) | flags)
    }
    pub fn contains_flags(self, flags: u64) -> bool {
        self.0 & Page::FLAG_MASK == flags
    }
}

const NUM_ENTRIES: usize = 512;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AddressSpaceHandle(u16);
impl AddressSpaceHandle {
    /// Kernel address space is always #1
    pub fn get_kernel() -> Self {
        Self(1)
    }
}

pub struct Manager;
impl Manager {
    pub fn init(_: &mut db::Database) {
        //db.aspaces[i] = pmm::Manager::alloc_page();
    }

    fn get_current_cr3() -> u64 {
        let r;
        unsafe {
            core::arch::asm!(
                "mov {}, cr3",
                out(reg) r
            );
        }
        r
    }

    pub fn new_address_space(db: &mut db::Database, pgtable: pmm::Handle) -> AddressSpaceHandle {
        db.aspaces.push(pgtable);
        let aspace = AddressSpaceHandle((db.aspaces.len() - 1) as u16);
        // We live in the fucking lower half, congrats -- now we get to pay the consequences
        unsafe extern "C" {
            unsafe static KERNEL_START: u8;
            unsafe static KERNEL_END: u8;
        }
        let kernel_start = &raw const KERNEL_START as u64;
        let kernel_end = &raw const KERNEL_END as u64;
        let kernel_pages = (kernel_end - kernel_start).div_ceil(pmm::PAGE_SIZE as u64) as usize;
        Self::map(
            db,
            aspace,
            kernel_start,
            kernel_start,
            kernel_pages,
            Page::PRESENT | Page::READ_WRITE,
        );
        aspace
    }

    pub fn traverse_page_table<F>(
        db: &db::Database,
        aspace: AddressSpaceHandle,
        vaddr: u64,
        mut f: F,
    )
    where
        F: FnMut(&Page),
    {
        let index = [
            (vaddr >> 39) as usize % NUM_ENTRIES,
            (vaddr >> 30) as usize % NUM_ENTRIES,
            (vaddr >> 21) as usize % NUM_ENTRIES,
            (vaddr >> 12) as usize % NUM_ENTRIES,
        ];
        let mut table = db.aspaces[aspace.0 as usize].get() as *const Page;
        for i in 0..index.len() {
            //kprint!("[vmm] walker {:0x}\r\n", index[i]);
            unsafe {
                let entry = table.add(index[i]);
                if i == index.len() - 1 {
                    f(entry.as_ref().unwrap());
                } else {
                    f(entry.as_ref().unwrap());
                    table = if (*entry).is_present() {
                        ((*entry).0 & !Page::FLAG_MASK) as *const Page
                    } else {
                        break;
                    };
                }
            }
        }
    }

    /// Maps a single page, if the page already exists it will simply go into the next level
    /// if the flags of the page differ, the flags will be updated but the TLB will not be flushed
    /// so the changes wont be reflected globally
    pub fn map_single(
        db: &mut db::Database,
        aspace: AddressSpaceHandle,
        paddr: u64,
        vaddr: u64,
        flags: u64,
    ) {
        //kprint!("Mapping {paddr:0x} => {vaddr:0x}\r\n",);
        let index = [
            (vaddr >> 39) as usize % NUM_ENTRIES,
            (vaddr >> 30) as usize % NUM_ENTRIES,
            (vaddr >> 21) as usize % NUM_ENTRIES,
            (vaddr >> 12) as usize % NUM_ENTRIES,
        ];
        let mut table = db.aspaces[aspace.0 as usize].get_mut() as *mut Page;
        for i in 0..index.len() {
            //kprint!("[vmm] walker {:0x}\r\n", index[i]);
            unsafe {
                let entry = table.add(index[i]);
                if i == index.len() - 1 {
                    *entry = Page((paddr & !Page::FLAG_MASK) | flags);
                } else {
                    table = if (*entry).is_present() {
                        // Always override flags
                        if !(*entry).contains_flags(flags) {
                            *entry = (*entry).override_flags(flags);
                        }
                        ((*entry).get_physaddr()) as *mut Page
                    } else {
                        let pd_addr = pmm::Manager::alloc_page_zeroed().get() as u64;
                        //kprint!("[vmm] alloc new addr {pd_addr:016x}\r\n");
                        assert_eq!(pd_addr & Page::FLAG_MASK, 0); //page aligned please
                        *entry = Page(pd_addr | flags);
                        pd_addr as *mut Page
                    };
                }
            }
        }
    }

    pub fn map(
        db: &mut db::Database,
        aspace: AddressSpaceHandle,
        mut paddr: u64,
        mut vaddr: u64,
        count: usize,
        flags: u64,
    ) {
        for _ in 0..count {
            Self::map_single(db, aspace, paddr, vaddr, flags);
            paddr += pmm::PAGE_SIZE as u64;
            vaddr += pmm::PAGE_SIZE as u64;
        }
    }

    pub fn has_mapping_present(
        db: &db::Database,
        aspace: AddressSpaceHandle,
        vaddr: u64,
    ) -> bool {
        let index = [
            (vaddr >> 39) as usize % NUM_ENTRIES,
            (vaddr >> 30) as usize % NUM_ENTRIES,
            (vaddr >> 21) as usize % NUM_ENTRIES,
            (vaddr >> 12) as usize % NUM_ENTRIES,
        ];
        let mut table = db.aspaces[aspace.0 as usize].get() as *const Page;
        unsafe {
            for i in 0..(index.len() - 1) {
                let entry = table.add(index[i]);
                table = if (*entry).is_present() {
                    ((*entry).get_physaddr()) as *const Page
                } else {
                    return false;
                };
            }
            let entry = table.add(index[index.len() - 1]);
            (*entry).is_present()
        }
    }

    pub fn invalidate_single(addr: u64) {
        unsafe {
            core::arch::asm!(
                "invlpg [{0}]",
                in(reg) addr,
                options(nostack)
            )
        }
    }

    /// Reloads entire TLB because fuck you
    pub fn reload_cr3(db: &db::Database, aspace: AddressSpaceHandle) {
        let table = db.aspaces[aspace.0 as usize].get_mut() as u64;
        kprint!("[vmm] loading table at {table:016x}\r\n");
        unsafe {
            core::arch::asm!(
                "mov cr3, {}",
                in(reg) table,
                options(nostack)
            )
        }
    }
}

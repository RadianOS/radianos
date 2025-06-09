use crate::{db, kprint, pmm};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page(u64);
impl Page {
    pub const FLAG_MASK: u64 = 0x3ff;

    pub const PRESENT: u64 = 0x01;
    pub const READ_WRITE: u64 = 0x02;
    pub const USER_SUPERVISOR: u64 = 0x04; //shared
    pub const WRITE_THROUGH: u64 = 0x08;

    pub fn is_present(self) -> bool {
        self.0 & Page::PRESENT != 0
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AddressSpaceHandle(u16);

pub struct Manager;
impl Manager {
    pub fn init(_: &mut db::Database) {
        //db.aspaces[i] = pmm::Manager::alloc_page();
    }

    #[allow(dead_code)]
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
        //let _start_addr = unsafe { (&KERNEL_START) as *const _ as u64 };
        //let _end_addr = unsafe { (&KERNEL_END) as *const _ as u64 };
        // We live in the fucking lower half, congrats -- now we get to pay the consequences
        Self::map(
            db,
            aspace,
            0,
            0,
            512,
            Page::PRESENT | Page::READ_WRITE,
        );
        aspace
    }

    pub fn map_single(
        db: &mut db::Database,
        aspace: AddressSpaceHandle,
        paddr: u64,
        vaddr: u64,
        flags: u64,
    ) {
        let index = [
            ((vaddr >> 39) % 512) as usize,
            ((vaddr >> 30) % 512) as usize,
            ((vaddr >> 21) % 512) as usize,
            ((vaddr >> 12) % 512) as usize,
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
                        ((*entry).0 & !Page::FLAG_MASK) as *mut Page
                    } else {
                        let pd_addr = pmm::Manager::alloc_page_zeroed().get() as u64;
                        kprint!("[vmm] alloc new addr {pd_addr:016x}\r\n");
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

    fn invalidate_single(addr: u64) {
        unsafe {
            core::arch::asm!(
                "invlpg ({})",
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

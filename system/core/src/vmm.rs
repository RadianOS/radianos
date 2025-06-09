use crate::{db, kprint, pmm, smp};

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
        AddressSpaceHandle((db.aspaces.len() - 1) as u16)
    }

    pub fn map(db: &mut db::Database, aspace: AddressSpaceHandle, mut paddr: u64, mut vaddr: u64, count: usize, flags: u64) {
        for _ in 0..count {
            let index = [
                ((vaddr >> 39) & Page::FLAG_MASK) as usize,
                ((vaddr >> 30) & Page::FLAG_MASK) as usize,
                ((vaddr >> 21) & Page::FLAG_MASK) as usize,
                ((vaddr >> 12) & Page::FLAG_MASK) as usize
            ];
            //kprint!("\r\nv={vaddr:016x}::p={paddr:016x}");
            let mut table = db.aspaces[aspace.0 as usize].get_mut() as *mut Page;
            for i in 0..index.len() {
                //kprint!("walk #{:04x} ", index[i]);
                unsafe {
                    let entry = table.add(index[i] as usize);
                    if i == index.len() - 1 {
                        *entry = Page((paddr & !Page::FLAG_MASK) | flags);
                    } else {
                        table = if (*entry).is_present() {
                            ((*entry).0 & !Page::FLAG_MASK) as *mut Page
                        } else {
                            let pd_addr = pmm::Manager::alloc_page_zeroed().get() as u64;
                            //kprint!("alloc new addr {pd_addr:016x} ");
                            assert_eq!(pd_addr & Page::FLAG_MASK, 0); //page aligned please
                            *entry = Page(pd_addr | flags);
                            pd_addr as *mut Page
                        };
                    }
                }
            }
            paddr += 4096;
            vaddr += 4096;
        }
    }

    /// Reloads entire TLB because fuck you
    pub fn evil_function_do_not_call(db: &db::Database, aspace: AddressSpaceHandle) {
        let table = db.aspaces[aspace.0 as usize].get_mut() as u64;
        kprint!("loading table at {table}\r\n");
        unsafe{core::arch::asm!(
            "mov cr3, {}",
            in(reg) table,
            options(nostack)
        )}
    }
}

use crate::{db, pmm, smp};

struct Page(u32);

pub struct Manager;
impl Manager {
    pub fn init(db: &mut db::Database) {
        // lapic id
        for i in 0..smp::Manager::get_core_count() {
            db.page_directory[i] = pmm::Manager::alloc_page();
        }
    }

    pub fn map(smp_id: usize, paddr: u64, vaddr: u64, flags: u32) {
        let index = [
            (vaddr >> 22),
            (vaddr >> 12) & 0x3ff
        ];
        
    }
}

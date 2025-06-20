use crate::db;
use crate::kprint;
use crate::pmm;
use crate::vmm;
use crate::containers::StaticVec;

#[derive(Debug)]
pub struct Task {
    gpr: [u64; 16],
    stack_page: pmm::Handle,
}
impl Task {
    pub fn new() -> Self {
        Self::default()
    }
}
impl Default for Task {
    fn default() -> Self {
        Self {
            gpr: core::array::from_fn(|_| 0),
            stack_page: pmm::Handle::default(),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskHandle(u8);

#[derive(Debug)]
pub struct Worker {
    aspace: vmm::AddressSpaceHandle,
    entry_point: u64,
    tasks: StaticVec<Task, 4>,
    flags: u8,
}
impl Worker {
    pub const SLEEP: u8 = 0x40;
    pub const ACTIVE: u8 = 0x80;

    pub fn new(aspace: vmm::AddressSpaceHandle) -> Self {
        Self{
            aspace,
            entry_point: 0,
            tasks: StaticVec::new(),
            flags: 0,
        }
    }
    const fn set_flag<const FLAG: u8>(&mut self, v: bool) {
        self.flags = (self.flags & !FLAG) | [0, FLAG][v as usize];
    }
    pub const fn is_active(&self) -> bool {
        self.flags & Self::ACTIVE != 0
    }
    pub const fn set_active(&mut self, v: bool) {
        self.set_flag::<{Self::ACTIVE}>(v)
    }
}

/// Default stack base
pub const TASK_STACK_BASE: u64 = 0x1100_0000;
pub type EntryFn = unsafe extern "C" fn() -> ();
/// Only used for shit like .bin or a.out
pub const PROGRAM_IMAGE_BASE: u64 = 0x1000_0000;

pub struct Manager;
impl Manager {
    /// Probably already enabled but just to be sure
    fn enable_sysret() {
        unsafe {
            core::arch::asm!(
                "mov rcx, 0xc0000082",
                "wrmsr",
                "mov rcx, 0xc0000080",
                "rdmsr",
                "or eax, 1",
                "wrmsr",
                "mov rcx, 0xc0000081",
                "rdmsr",
                "mov edx, 0x00180008",
                "wrmsr",
            )
        }
    }

    pub fn init(db: &mut db::Database) {
        Self::enable_sysret();
    }

    pub fn new_worker(db: &mut db::Database, aspace: vmm::AddressSpaceHandle) -> db::ObjectHandle {
        let worker = Worker::new(aspace);
        db.workers.push(worker); //kernel worker
        db::ObjectHandle::new::<{db::ObjectHandle::WORKER}>((db.workers.len() - 1) as u16)
    }

    fn get_worker_aspace(db: &db::Database, id: db::ObjectHandle) -> vmm::AddressSpaceHandle {
        db.workers.get(id.get_id() as usize).map(|w| w.aspace).unwrap_or_default()
    }

    pub fn new_task(db: &mut db::Database, id: db::ObjectHandle) -> Option<TaskHandle> {
        if let Some(worker) = db.workers.get_mut(id.get_id() as usize) {
            worker.tasks.push(Task::new());
            let task_id = TaskHandle((worker.tasks.len() - 1) as u8);
            // Map the stack (default)
            let aspace = Self::get_worker_aspace(db, id);
            let stack_page = pmm::Manager::alloc_page_zeroed();
            vmm::Manager::map(db, aspace, stack_page.get() as u64, TASK_STACK_BASE, 1, vmm::Page::PRESENT | vmm::Page::READ_WRITE);
            Some(task_id)
        } else {
            None
        }
    }

    #[unsafe(no_mangle)]
    pub fn switch_to_usermode(next_rip: u64) {
        unsafe {
            core::arch::asm!(
                "mov r11, 0x202",
                "sysretq",
                in("rcx") next_rip,
                options(nostack),
                options(noreturn),
            );
        }
    }

    pub fn load_elf_into_worker(db: &mut db::Database, id: db::ObjectHandle, bytes: &[u8], main: bool) {
        use xmas_elf::{program, ElfFile};
        let elf = ElfFile::new(&bytes).expect("Failed to parse ELF file");
        let aspace = Self::get_worker_aspace(db, id);
        kprint!("[task] using aspace = {:?}\r\n", aspace);

        for ph in elf.program_iter() {
            if ph.get_type().unwrap() == program::Type::Dynamic {
                kprint!("[task] Skipping dynamic segment\r\n");
            }
            if ph.get_type().expect("Failed to get header type") != program::Type::Load {
                continue;
            }

            let mem_size = ph.mem_size() as usize;
            let virt_addr = ph.virtual_addr() as usize;

            let aligned_virt_addr = virt_addr & !0xFFF;
            let page_offset = virt_addr - aligned_virt_addr;
            let total_size = page_offset + mem_size;
            let num_pages = total_size.div_ceil(0x1000);
            kprint!("[task] Using {num_pages} pages, addr = {virt_addr:0x}, align {aligned_virt_addr:0x} with type {:0x}\r\n", ph.physical_addr());
            for i in 0..num_pages {
                let handle = pmm::Manager::alloc_page();
                let ptr = handle.get_mut();
                let file_offset = (ph.offset() as usize - i * pmm::PAGE_SIZE).min(pmm::PAGE_SIZE);
                let file_size = (ph.file_size() as usize - i * pmm::PAGE_SIZE).min(pmm::PAGE_SIZE);
                let mem_size = (ph.mem_size() as usize - i * pmm::PAGE_SIZE).min(pmm::PAGE_SIZE);
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        bytes[file_offset..].as_ptr(),
                        ptr.add(page_offset),
                        file_size
                    );
                    if mem_size > file_size {
                        core::ptr::write_bytes(
                            ptr.add(page_offset + file_size),
                            0,
                            mem_size - file_size
                        );
                    }
                }
                kprint!("[task] {:016x} => {virt_addr:016x}; file_size={file_size}, file_offset={file_offset}, mem_size={mem_size}\r\n", ptr as u64);
                vmm::Manager::map_single(db, aspace, ptr as u64, (virt_addr + i * pmm::PAGE_SIZE) as u64, vmm::Page::PRESENT | vmm::Page::READ_WRITE | vmm::Page::USER_SUPERVISOR);
            }
        }
        //let entry_function: EntryFn = unsafe { core::mem::transmute(entry_point) };
        if main {
            if let Some(worker) = db.workers.get_mut(id.get_id() as usize) {
                worker.entry_point = elf.header.pt2.entry_point();
                kprint!("[task] entry point at {:016x}\r\n", worker.entry_point);
            }
        }
    }

    /// Simple round robin
    pub fn scheduler_tick(db: &mut db::Database) -> db::ObjectHandle {
        assert!(db.workers.len() > 0); //cant schedule without workers
        let mut active_index = 0;
        for i in 0..db.workers.len() {
            if db.workers[i].is_active() {
                active_index = i;
                break;
            }
        }
        let next_index = (active_index + 1) % db.workers.len();
        if next_index != active_index {
            db.workers[active_index].set_active(false);
            db.workers[next_index].set_active(true);
        }
        db::ObjectHandle::new::<{db::ObjectHandle::WORKER}>(next_index as u16)
    }
}

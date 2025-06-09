use crate::db;
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
    tasks: StaticVec<Task, 4>,
}
impl Worker {
    pub fn new(aspace: vmm::AddressSpaceHandle) -> Self {
        Self{
            aspace,
            tasks: StaticVec::new(),
        }
    }
}

/// Default stack base
pub const TASK_STACK_BASE: u64 = 0xC000_0000;

pub struct Manager;
impl Manager {
    pub fn init(db: &mut db::Database) {

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
}

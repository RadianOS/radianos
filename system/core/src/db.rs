use crate::{policy, pmm, vfs};

/// "Fat pointer" - only use if you absolutely dont know the source of id
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectHandle {
    id: u16,
    type_: u16,
}
impl ObjectHandle {
    pub const NONE: u16 = 0;
    pub const WORKER: u16 = 1;
    pub const ACTOR: u16 = 2;
}

pub struct Worker {
    gpr: [u64; 16],
    stack_page: pmm::Handle,
}
impl Worker {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Worker {
    fn default() -> Self {
        Self {
            gpr: core::array::from_fn(|_| 0),
            stack_page: pmm::Handle::default(),
        }
    }
}

crate::dense_soa_generic!(
    struct Database;
    workers: Worker,
    policy_rule: policy::PolicyRule,
    vfs_nodes: vfs::Node,
    vfs_providers: vfs::Provider,
);
static mut GLOBAL_DATABASE: [u8; core::mem::size_of::<Database>()] = [0u8; core::mem::size_of::<Database>()];

impl Database {
    pub fn init() {
        
    }

    /// Assumed lock
    pub fn get() -> &'static Self {
        unsafe {
            #[allow(static_mut_refs)]
            (GLOBAL_DATABASE.as_ptr() as *const Self).as_ref().unwrap()
        }
    }

    pub fn get_mut() -> &'static mut Self {
        unsafe {
            #[allow(static_mut_refs)]
            (GLOBAL_DATABASE.as_mut_ptr() as *mut Self).as_mut().unwrap()
        }
    }

    pub fn find_from_str(&self, s: &str) -> Option<ObjectHandle> {
        if s.starts_with("worker_") {
            let offset = s.strip_prefix("worker_").unwrap().parse::<usize>().unwrap_or_default();
            if offset < self.workers.len() {
                Some(ObjectHandle{
                    id: offset as u16,
                    type_: ObjectHandle::WORKER,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

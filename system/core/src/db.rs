use crate::{policy, pmm};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Handle {
    id: u8,
    type_: u8,
}
impl Handle {
    pub const NONE: u8 = 0;
    pub const WORKER: u8 = 1;
    pub const ACTOR: u8 = 2;
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

pub struct Transaction {
    data: pmm::Handle,
}

pub struct Provider {
    
}

crate::dense_soa_generic!(
    struct Database;
    workers: Worker,
    logged_transactions: Transaction,
    pending_transactions: Transaction,
    policy_rule: policy::PolicyRule,
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

    pub fn find_from_str(&self, s: &str) -> Option<Handle> {
        if s.starts_with("worker_") {
            let offset = s.strip_prefix("worker_").unwrap().parse::<usize>().unwrap_or_default();
            if offset < self.workers.len() {
                Some(Handle{
                    id: offset as u8,
                    type_: Handle::WORKER,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

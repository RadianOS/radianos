use crate::{containers::StaticVec, pmm, policy, vfs, task};

/// "Fat pointer" - only use if you absolutely dont know the source of id
/// or if the object does not have a handle of its own, in such case, you're more than
/// welcome to use the fat id
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectHandle {
    id: u16,
    type_: u16,
}
impl ObjectHandle {
    pub const NONE: u16 = 0;
    pub const WORKER: u16 = 1;
    pub const ACTOR: u16 = 2;
    //???
    pub const ADDRESS_SPACE: u16 = 3;
    pub const VFS_NODE: u16 = 4;
    pub const fn new<const TYPE: u16>(id: u16) -> Self {
        Self{
            id,
            type_: TYPE,
        }
    }
    pub const fn get_id(self) -> u16 {
        self.id
    }
}

pub struct Database {
    pub workers: StaticVec<task::Worker, 64>,
    pub policy_rule: StaticVec<policy::PolicyRule, 128>,
    pub vfs_nodes: StaticVec<vfs::Node, 128>,
    pub vfs_providers: StaticVec<vfs::Provider, 32>,
    pub aspaces: StaticVec<pmm::Handle, 64>,
}
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

static DEFAULT_PATHBUF: PathBuf<'static> = PathBuf::from_str(".");
pub struct PathBuf<'a> {
    inner: &'a str,
}
impl<'a> PathBuf<'a> {
    pub const fn from_str(inner: &'a str) -> Self {
        Self{ inner }
    }
    pub fn path(&'a self) -> Path<'a> {
        Path{ inner: &self }
    }
}

pub struct Path<'a> {
    inner: &'a PathBuf<'a>, //смлв
}
impl<'a> Path<'a> {
    pub fn new() -> Self {
        Self{ inner: &DEFAULT_PATHBUF, }
    }
    pub fn components(&self) -> core::str::Split<'a, &'a str> {
        self.inner.inner.split("/")
    }
    pub fn file_name(&'a self) -> Option<&'a str> {
        self.components().last().map(|p| p.split(".").next()).unwrap_or(None)
    }
    pub fn extension(&'a self) -> Option<&'a str> {
        self.components().last().map(|p| p.split(".").last()).unwrap_or(None)
    }
}

use core::str;

use crate::{db, kprint, policy};

#[derive(Default, Debug)]
pub enum Error {
    #[default]
    Unknown,
    Policy,
    Custom(u32),
}
pub type Result = core::result::Result<usize, Error>;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProviderHandle(u16);
pub struct Provider {
    write: fn(db: &mut db::Database, actor: db::ObjectHandle, data: &[u8]) -> Result,
    #[allow(dead_code)]
    read: fn(db: &mut db::Database, actor: db::ObjectHandle, data: &mut [u8]) -> Result,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeHandle(u16);
impl NodeHandle {
    pub fn is_root(self) -> bool {
        self.0 == 0
    }
}
pub struct Node {
    name: [u8; 24],
    parent: NodeHandle,
    provider: ProviderHandle,
}
impl Node {
    pub fn get_name(&self) -> &str {
        let len = self
            .name
            .iter()
            .enumerate()
            .find(|&(_, c)| *c == 0)
            .map(|(i, _)| i)
            .unwrap_or(self.name.len());
        unsafe { str::from_raw_parts(self.name.as_ptr(), len) }
    }
    /// You fucking better dont clone/copy this around without understanding its consequences
    pub fn get_parent(&self) -> &NodeHandle {
        &self.parent
    }
    pub fn get_provider(&self) -> &ProviderHandle {
        &self.provider
    }
}

pub struct Manager;
impl Manager {
    pub fn init(db: &mut db::Database) {
        let root_handle = NodeHandle::default();
        Self::new_node(db, "", root_handle); //root node do not touch ignore please
        Self::new_provider(
            db,
            Provider {
                write: |_, _, _| Err(Error::Unknown),
                read: |_, _, _| Err(Error::Unknown),
            },
        ); //default provider

        let log_provider = Self::new_provider(
            db,
            Provider {
                write: |db, actor, data| {
                    let caps = policy::Capability::default().with(policy::Capability::WRITE_LOG);
                    if policy::PolicyEngine::check_capability(db, actor, caps) {
                        let s = unsafe { str::from_raw_parts(data.as_ptr(), data.len()) };
                        kprint!("{}", s);
                        Ok(data.len())
                    } else {
                        Err(Error::Policy)
                    }
                },
                read: |_db, _actor, _data| Err(Error::Policy),
            },
        );

        Self::new_node(db, "binary", root_handle);
        let boot_dir = Self::new_node(db, "boot", root_handle);
        Self::new_node(db, "x86_64", boot_dir);

        Self::new_node(db, "devices", root_handle);
        Self::new_node(db, "mount", root_handle);

        let mutable_dir = Self::new_node(db, "mutable", root_handle);
        Self::new_node(db, "spool", mutable_dir);
        Self::new_node(db, "cache", mutable_dir);
        Self::new_node(db, "runtime", mutable_dir);
        let mutable_logs_dir = Self::new_node(db, "logs", mutable_dir);
        Self::new_node_with_provider(db, "radian_core.log", mutable_logs_dir, log_provider);

        let systen_dir = Self::new_node(db, "system", root_handle);
        Self::new_node(db, "include", systen_dir);
        Self::new_node(db, "lib", systen_dir);
        Self::new_node(db, "opt", systen_dir);
        Self::new_node(db, "run", systen_dir);

        Self::new_node(db, "temp", root_handle);

        let user_dir = Self::new_node(db, "user", root_handle);
        Self::new_node(db, "home", user_dir);
        Self::new_node(db, "binary", user_dir);

        Self::new_node(db, "misc", root_handle);
        Self::new_node(db, "opt", root_handle);
    }
    pub fn new_provider(db: &mut db::Database, provider: Provider) -> ProviderHandle {
        db.vfs_providers.push(provider);
        ProviderHandle((db.vfs_providers.len() - 1) as u16)
    }
    #[inline]
    pub fn new_node(db: &mut db::Database, name: &str, parent: NodeHandle) -> NodeHandle {
        Self::new_node_with_provider(db, name, parent, ProviderHandle::default())
    }
    pub fn new_node_with_provider(
        db: &mut db::Database,
        name: &str,
        parent: NodeHandle,
        provider: ProviderHandle,
    ) -> NodeHandle {
        let bytes = core::array::from_fn(|i| name.as_bytes().get(i).copied().unwrap_or(0));
        db.vfs_nodes.push(Node {
            name: bytes,
            parent,
            provider,
        });
        NodeHandle((db.vfs_nodes.len() - 1) as u16)
    }
    pub fn for_each_children<F: FnMut(NodeHandle)>(db: &db::Database, which: NodeHandle, mut f: F) {
        for i in 1..db.vfs_nodes.len() {
            let node = &db.vfs_nodes[i];
            if node.parent == which {
                f(NodeHandle(i as u16));
            }
        }
    }
    pub fn find_children(db: &db::Database, from: NodeHandle, name: &str) -> Option<NodeHandle> {
        for i in 1..db.vfs_nodes.len() {
            let node = &db.vfs_nodes[i];
            if node.parent == from && node.get_name() == name {
                return Some(NodeHandle(i as u16));
            }
        }
        None
    }
    pub fn get_node(db: &db::Database, handle: NodeHandle) -> &Node {
        &db.vfs_nodes[handle.0 as usize]
    }
    pub fn get_node_mut(db: &mut db::Database, handle: NodeHandle) -> &mut Node {
        db.vfs_nodes.get_mut(handle.0 as usize).unwrap()
    }
    pub fn invoke_provider_write(
        db: &mut db::Database,
        handle: ProviderHandle,
        actor: db::ObjectHandle,
        data: &[u8],
    ) -> Result {
        let f = db.vfs_providers[handle.0 as usize].write;
        f(db, actor, data)
    }
    pub fn invoke_provider_read(
        db: &mut db::Database,
        handle: ProviderHandle,
        actor: db::ObjectHandle,
        data: &mut [u8],
    ) -> Result {
        let f = db.vfs_providers[handle.0 as usize].write;
        f(db, actor, data)
    }
}

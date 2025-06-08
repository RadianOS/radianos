use core::str;

use crate::db;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeHandle(u16);
impl NodeHandle {
    pub fn is_root(self) -> bool {
        self.0 == 0
    }
}

pub struct Node {
    name: [u8; 8],
    parent: NodeHandle,
}
impl Node {
    pub fn get_name<'a>(&'a self) -> &'a str {
        let len = self.name.iter().enumerate().find(|&(i, c)| *c == 0).map(|(i, _)| i).unwrap_or(self.name.len());
        unsafe {
            str::from_raw_parts(self.name.as_ptr(), len)
        }
    }
    /// You fucking better dont clone/copy this around without understanding its consequences
    pub fn get_parent<'a>(&'a self) -> &'a NodeHandle {
        &self.parent
    }
}

pub struct Manager;
impl Manager {
    pub fn init(db: &mut db::Database) {
        let root_handle = NodeHandle::default();
        Self::new_node(db, "", root_handle); //root node do not touch ignore please

        Self::new_node(db, "binary", root_handle);
        let boot_dir = Self::new_node(db, "boot", root_handle);
        Self::new_node(db, "x86_64", boot_dir);

        Self::new_node(db, "devices", root_handle);
        Self::new_node(db, "mount", root_handle);

        let mutable_dir = Self::new_node(db, "mutable", root_handle);
        Self::new_node(db, "logs", mutable_dir);
        Self::new_node(db, "spool", mutable_dir);
        Self::new_node(db, "cache", mutable_dir);
        Self::new_node(db, "runtime", mutable_dir);

        Self::new_node(db, "system", root_handle);
        Self::new_node(db, "temp", root_handle);

        let user_dir = Self::new_node(db, "user", root_handle);
        Self::new_node(db, "home", user_dir);
        Self::new_node(db, "binary", user_dir);

        Self::new_node(db, "opt", root_handle);
    }
    pub fn new_node(db: &mut db::Database, name: &str, parent: NodeHandle) -> NodeHandle {
        let bytes = core::array::from_fn(|i| name.bytes().nth(i).unwrap_or(0));
        db.vfs_nodes.push(Node{
            name: bytes,
            parent,
        });
        NodeHandle((db.vfs_nodes.len() - 1) as u16)
    }
    pub fn for_each_children<F: Fn(NodeHandle)>(db: &db::Database, which: NodeHandle, f: F) {
        for i in 1..db.vfs_nodes.len() {
            let node = db.vfs_nodes.get(i).unwrap();
            if node.parent == which {
                f(NodeHandle(i as u16));
            }
        }
    }
    pub fn find_children(db: &db::Database, from: NodeHandle, name: &str) -> Option<NodeHandle> {
        for i in 1..db.vfs_nodes.len() {
            let node = db.vfs_nodes.get(i).unwrap();
            if node.parent == from && node.get_name() == name {
                return Some(NodeHandle(i as u16));
            }
        }
        None
    }
    pub fn get_node<'a>(db: &'a db::Database, handle: NodeHandle) -> &'a Node {
        db.vfs_nodes.get(handle.0 as usize).unwrap()
    }
    pub fn get_node_mut<'a>(db: &'a mut db::Database, handle: NodeHandle) -> &'a mut Node {
        db.vfs_nodes.get_mut(handle.0 as usize).unwrap()
    }
}

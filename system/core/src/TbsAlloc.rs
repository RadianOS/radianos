/// Thread-Bucket-S type allocator
/// 
/// The allocator will bucket "arenas" corresponding to a given thread
/// the buckets will have a fixed size of 2MiB but can dynamically grow
/// 
/// For large allocations the max allowed allocation size is the spacing
/// between arenas, for this reason only arenas that are used are initialized

use core::alloc::{AllocError, Allocator, GlobalAlloc, Layout};
use core::ptr::NonNull;

use crate::containers::{FlexibleArray, StaticVec};
use crate::{db, kprint, pmm, vmm};

pub const CACHE_LINE_SIZE: usize = 64;

/// Max number of arenas total, set it to thread count
pub const MAX_ARENAS: usize = 8;
pub const ARENA_DEFAULT_SIZE: usize = 2097152; // Size of a given arena
pub const ARENA_MAX_SIZE: usize = 1 << 48; // Max size supported by allocator
pub const ARENA_DEFAULT_BASE: usize = 0x1000_0000; //Base of allocations
pub const ARENA_DEFAULT_SPACING: usize = 0x1000_0000; //1 GiB from each other

#[derive(Default, Debug, Clone, Copy)]
struct IntrusiveIntervalNode {
    base: usize,
    length: usize,
    is_free: bool,
    left: usize,
    right: usize,
    height: i8,
}
impl IntrusiveIntervalNode {
    #[inline]
    fn is_present(&self) -> bool {
        self.base != 0
    }
    fn get_height(&self) -> i8 {
        if self.is_present() {
            self.height
        } else {
            0
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct IntrusiveIntervalTree {
    extent: usize,
    root: usize,
    nodes: FlexibleArray<IntrusiveIntervalNode>,
}
impl IntrusiveIntervalTree {
    const NODES_OFFSET: usize = core::mem::size_of::<IntrusiveIntervalTree>();
    fn init(&mut self, root_base: usize, root_length: usize) {
        self.root = 0;
        self.extent = 0;
        // Create the null node
        self.nodes[0] = IntrusiveIntervalNode::default();
        self.extent += 1;
        let root = self.alloc_node();
        self.nodes[root].base = root_base;
        self.nodes[root].length = root_length;
        self.nodes[root].is_free = true;
        self.root = root;
    }
    #[inline] fn get_node<'a>(&'a self, index: usize) -> &'a IntrusiveIntervalNode {
        &self.nodes[index]
    }
    #[inline] fn get_node_mut<'a>(&'a mut self, index: usize) -> &'a mut IntrusiveIntervalNode {
        &mut self.nodes[index]
    }
    fn alloc_node(&mut self) -> usize {
        for i in 1..self.extent {
            if !self.nodes[i].is_present() {
                return i;
            }
        }
        self.extent += 1;
        // This is horrible but i don't give a fuck
        unsafe {
            // EVIL NON-DETERMINISM IF YOU DONT DO THIS
            self.nodes[self.extent] = IntrusiveIntervalNode::default();
            let vaddr = &raw const self.nodes[self.extent] as u64;
            let db = db::Database::get_mut();
            let aspace =vmm::AddressSpaceHandle::get_kernel();
            if !vmm::Manager::has_mapping_present(db, aspace, (vaddr & !0xfff) as u64) {
                let handle = pmm::Manager::alloc_page();
                vmm::Manager::map(db, aspace, handle.get() as u64, (vaddr & !0xfff) as u64, 1, vmm::Page::PRESENT | vmm::Page::READ_WRITE);
            }
        }
        self.extent - 1
    }
    fn max_height(&self, index: usize) -> i8 {
        let h1 = self.nodes[self.nodes[index].left].get_height();
        let h2 = self.nodes[self.nodes[index].right].get_height();
        h1.max(h2)
    }
    fn left_rotate(&mut self, x: usize) -> usize {
        let y = self.nodes[x].right;
        let tmp = self.nodes[y].left;
        self.nodes[y].left = x;
        self.nodes[x].right = tmp;
        self.nodes[x].height = 1 + self.max_height(x);
        self.nodes[y].height = 1 + self.max_height(y);
        y
    }
    fn right_rotate(&mut self, y: usize) -> usize {
        let x = self.nodes[y].left;
        let tmp = self.nodes[x].right;
        self.nodes[x].right = y;
        self.nodes[y].left = tmp;
        self.nodes[y].height = 1 + self.max_height(y);
        self.nodes[x].height = 1 + self.max_height(x);
        x
    }
    fn get_balance(&self, index: usize) -> i8 {
        if self.nodes[index].is_present() {
            self.nodes[self.nodes[index].left].get_height() - self.nodes[self.nodes[index].right].get_height()
        } else {
            0
        }
    }
    fn insert(&mut self, index: usize, base: usize, length: usize, is_free: bool) -> usize {
        if self.nodes[index].is_present() {
            if base < self.nodes[index].base {
                self.nodes[index].left = self.insert(self.nodes[index].left, base, length, is_free);
            } else if base > self.nodes[index].base {
                self.nodes[index].right = self.insert(self.nodes[index].right, base, length, is_free);
            }
            self.nodes[index].height = 1 + self.max_height(index);
            let balance = self.get_balance(index);
            if balance > 1 && self.get_balance(self.nodes[index].left) >= 0 { //LL
                return self.right_rotate(index);
            } else if balance > 1 && self.get_balance(self.nodes[index].left) < 0 { //LR
                self.nodes[index].left = self.left_rotate(self.nodes[index].left);
                return self.right_rotate(index);
            } else if balance < -1 && self.get_balance(self.nodes[index].right) <= 0 { //RR
                return self.left_rotate(index);
            } else if balance < -1 && self.get_balance(self.nodes[index].right) > 0 { //RL
                self.nodes[index].right = self.right_rotate(self.nodes[index].right);
                return self.left_rotate(index);
            }
            index
        } else {
            let new_node = self.alloc_node();
            self.nodes[new_node].base = base;
            self.nodes[new_node].length = length;
            self.nodes[new_node].is_free = is_free;
            new_node
        }
    }
    fn find_free(&mut self, index: usize, length: usize) -> Option<usize> {
        if self.nodes[index].is_present() {
            if self.nodes[index].length >= length && self.nodes[index].is_free == true {
                return Some(index);
            } else if let Some(left) = self.find_free(self.nodes[index].left, length) {
                return Some(left);
            } else if let Some(right) = self.find_free(self.nodes[index].right, length) {
                return Some(right);
            } else {
                None
            }
        } else {
            None
        }
    }

    fn print_debug(&self, index: usize, level: usize) {
        let tree_print_node = |index: usize, level: usize| {
            if level > 0 {
                for i in 0..level - 1 {
                    kprint!("│   ");
                }
                kprint!("└── ");
            }
            kprint!("{:0x}:{:0x}:{}\r\n", self.nodes[index].base, self.nodes[index].length,
                ['U','F'][self.nodes[index].is_free as usize]);
        };
        if self.nodes[index].is_present() {
            tree_print_node(index, level);
            self.print_debug(self.nodes[index].left, level + 1);
            self.print_debug(self.nodes[index].right, level + 1);
        } else {

        }
    }
}

pub fn test_self() {
    unsafe {
        let mut buffer = [0u8; 1024];
        let tree = buffer.as_mut_ptr().byte_add(4 - (buffer.as_ptr() as usize) % 4) as *mut IntrusiveIntervalTree;
        IntrusiveIntervalTree::init(tree.as_mut().unwrap(), 64, 64);
        kprint!("[tbs] root={}\r\n", (*tree).root);
        for i in 0..4 {
            let new_root = tree.as_mut().unwrap().insert((*tree).root, 65535 - 1024 * i, 512 * i, true);
            (*tree).root = new_root;
            kprint!("[tbs] insert={new_root}\r\n",);
        }
    }
}
pub fn print_debug() {
    unsafe {
        let arenas = &raw mut TBS_ALLOCATOR.arenas;
        for i in 0..MAX_ARENAS {
            if (*arenas)[i].is_present() {
                kprint!("==>Arena#{i}\r\n");
                let tree = ((*arenas)[i].get_base_mut() as *mut IntrusiveIntervalTree)
                    .as_mut().unwrap();
                tree.print_debug(tree.root, 0);
            }
        }
    }
}

#[derive(Default, Debug)]
struct Arena {
    base: usize,
    length: usize,
    lock: core::sync::atomic::AtomicBool,
}
impl Arena {
    pub const fn new(base: usize, length: usize) -> Self {
        //assert_eq!(base % CACHE_LINE_SIZE, 0);
        //assert_eq!(length % CACHE_LINE_SIZE, 0);
        Self{
            base,
            length,
            lock: core::sync::atomic::AtomicBool::new(false),
        }
    }
    #[inline]
    pub fn get_base_ptr<T>(&self) -> *const T {
        self.base as *const T
    }
    #[inline]
    pub fn get_base_mut<T>(&mut self) -> *mut T {
        self.base as *mut T
    }
    pub fn is_present(&self) -> bool {
        self.base != 0
    }
}
pub struct TbsAllocator {
    arenas: [Arena; MAX_ARENAS]
}
impl TbsAllocator {
    pub const fn new() -> Self {
        Self {
            // CORE ARRAY FROM FN IS NOT CONST YOU FUCKING KIDDING ME?
            arenas: [
                Arena::new(0, 0),
                Arena::new(0, 0),
                Arena::new(0, 0),
                Arena::new(0, 0),
                Arena::new(0, 0),
                Arena::new(0, 0),
                Arena::new(0, 0),
                Arena::new(0, 0),
            ],
        }
    }
    pub fn init(db: &mut db::Database, aspace: vmm::AddressSpaceHandle) {
        unsafe {
            // Initialize first arena
            (*&raw mut TBS_ALLOCATOR).arenas[0] = Arena::new(ARENA_DEFAULT_BASE, ARENA_DEFAULT_SIZE);
            // Create first page for tree span
            let handle = pmm::Manager::alloc_page();
            vmm::Manager::map(db, aspace, handle.get() as u64, ARENA_DEFAULT_BASE as u64, 1, vmm::Page::PRESENT | vmm::Page::READ_WRITE);
            let tree = (*&raw mut TBS_ALLOCATOR).arenas[0].get_base_mut() as *mut IntrusiveIntervalTree;
            (*tree).init(ARENA_DEFAULT_BASE, ARENA_DEFAULT_SIZE);
        }
    }
}
unsafe impl GlobalAlloc for TbsAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() == 0 {
            return core::ptr::null_mut();
        }
        assert!(layout.align() <= CACHE_LINE_SIZE);
        let aligned_size = layout.size().div_ceil(CACHE_LINE_SIZE) * CACHE_LINE_SIZE;
        unsafe {
            let arenas = &raw mut TBS_ALLOCATOR.arenas;
            for i in 0..MAX_ARENAS {
                if (*arenas)[i].is_present() {
                    let tree = ((*arenas)[i].get_base_mut() as *mut IntrusiveIntervalTree)
                        .as_mut().unwrap();
                    if let Some(free) = tree.find_free(tree.root, aligned_size) {
                        let new_ptr;
                        if tree.nodes[free].length == aligned_size {
                            new_ptr = tree.nodes[free].base;
                            tree.nodes[free].is_free = false;
                        } else {
                            new_ptr = tree.nodes[free].base + tree.nodes[free].length - aligned_size;
                            tree.nodes[free].length -= aligned_size;
                            tree.root = tree.insert(tree.root, new_ptr, aligned_size, false);
                        }
                        // Map if not already
                        let aspace = vmm::AddressSpaceHandle::get_kernel();
                        let db = db::Database::get_mut();
                        if !vmm::Manager::has_mapping_present(db, aspace, (new_ptr & !0xfff) as u64) {
                            let handle = pmm::Manager::alloc_page();
                            vmm::Manager::map(db, aspace, handle.get() as u64, (new_ptr & !0xfff) as u64, 1, vmm::Page::PRESENT | vmm::Page::READ_WRITE);
                        }
                        return new_ptr as *mut u8;
                    } else {
                        tree.print_debug(tree.root, 0);
                        unreachable!();
                    }
                }
            }
        }
        unreachable!()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // uh lmao
    }
}
#[global_allocator]
static mut TBS_ALLOCATOR: TbsAllocator = TbsAllocator::new();

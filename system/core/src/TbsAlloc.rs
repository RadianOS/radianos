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
use crate::{kprint, pmm, vmm};

pub const CACHE_LINE_SIZE: usize = 64;

/// Max number of arenas total, set it to thread count
pub const MAX_ARENAS: usize = 8;
pub const ARENA_DEFAULT_SIZE: usize = 2097152; // Size of a given arena
pub const ARENA_DEFAULT_BASE: usize = 0x1000_0000; //Base of allocations
pub const ARENA_DEFAULT_SPACING: usize = 0x1000_0000; //1 GiB from each other

#[derive(Default, Debug, Clone, Copy)]
struct HeapNode {
    base: usize,
    length: usize,
    left: usize,
    right: usize,
    height: i8,
}
impl HeapNode {
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

pub struct HeapNodeAccessor<'a> {
    tree: &'a mut HeapTree,
}

#[derive(Default, Debug, Clone, Copy)]
struct HeapTree {
    extent: usize,
    root: usize,
    nodes: FlexibleArray<HeapNode>,
}
impl HeapTree {
    const NODES_OFFSET: usize = core::mem::size_of::<HeapTree>();
    fn init(&mut self, root_base: usize, root_length: usize) {
        self.root = 0;
        self.extent = 0;
        // Create the null node
        self.nodes[0] = HeapNode::default();
        self.extent += 1;
        let root = self.alloc_node();
        self.nodes[root].base = root_base;
        self.nodes[root].length = root_length;
        self.root = root;
    }
    #[inline] fn get_node<'a>(&'a self, index: usize) -> &'a HeapNode {
        &self.nodes[index]
    }
    #[inline] fn get_node_mut<'a>(&'a mut self, index: usize) -> &'a mut HeapNode {
        &mut self.nodes[index]
    }
    fn alloc_node(&mut self) -> usize {
        for i in 1..self.extent {
            if !self.nodes[i].is_present() {
                return i;
            }
        }
        self.extent += 1;
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
    fn insert(&mut self, index: usize, base: usize, length: usize) -> usize {
        if self.nodes[index].is_present() {
            if base < self.nodes[index].base {
                self.nodes[index].left = self.insert(self.nodes[index].left, base, length);
            } else if base > self.nodes[index].base {
                self.nodes[index].right = self.insert(self.nodes[index].right, base, length);
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
            new_node
        }
    }
}

#[unsafe(no_mangle)]
pub fn test_self() {
    unsafe {
        let mut buffer = [0u8; 1024];
        let tree = buffer.as_mut_ptr().byte_add(4 - (buffer.as_ptr() as usize) % 4) as *mut HeapTree;
        HeapTree::init(tree.as_mut().unwrap(), 64, 64);
        kprint!("[tbs] root={}\r\n", (*tree).root);
        for i in 0..4 {
            let new_root = tree.as_mut().unwrap().insert((*tree).root, 65535 - 1024 * i, 512 * i);
            (*tree).root = new_root;
            kprint!("[tbs] insert={new_root}\r\n",);
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
    pub fn init() {
        unsafe {
            // Initialize first arena
            (*&raw mut TBS_ALLOCATOR).arenas[0] = Arena::new(ARENA_DEFAULT_BASE, ARENA_DEFAULT_SIZE);
            let tree = (*&raw mut TBS_ALLOCATOR).arenas[0].get_base_mut() as *mut HeapTree;
            (*tree).init(ARENA_DEFAULT_BASE, ARENA_DEFAULT_SIZE);
        }
    }
}
unsafe impl GlobalAlloc for TbsAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            let arenas = &raw mut TBS_ALLOCATOR.arenas;
            for i in 0..MAX_ARENAS {
                if (*arenas)[i].is_present() {
                    let tree = (*arenas)[i].get_base_mut() as *mut HeapTree;
                    
                }
            }
        }   
        core::ptr::null_mut()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        
    }
}
#[global_allocator]
static mut TBS_ALLOCATOR: TbsAllocator = TbsAllocator::new();

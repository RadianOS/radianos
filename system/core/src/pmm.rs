use crate::{containers::StaticVec, kprint, weak_typed_enum};

type BitmapEntry = u64;
const BITMAP_BYTES: usize = core::mem::size_of::<BitmapEntry>();
const BITMAP_BITS: usize = BITMAP_BYTES * 8;
pub const PAGE_SIZE: usize = 4096;

#[derive(Default, Debug, Clone, Copy)]
struct RelativeHandle(u16);

#[derive(Default, Debug, Clone, Copy)]
struct Arena {
    /// Must be page aligned
    base: usize,
    /// Must be page aligned
    length: usize,
}
impl Arena {
    pub const fn new(base: usize, length: usize) -> Self {
        Self{
            base,
            length,
        }
    }
    fn reset_heap(&mut self) {
        unsafe {
            self.get_heap_mut().write_bytes(0, self.get_num_pages().div_ceil(BITMAP_BITS));
            // Heap pages needed for heap mark as used
            for i in 0..self.get_num_pages().div_ceil(BITMAP_BITS).div_ceil(PAGE_SIZE) {
                let offs = i / 64;
                let mask = 1 << (i % 64);
                let new_val = self.get_heap().add(offs).read() | mask;
                self.get_heap_mut().add(offs).write(new_val);
            }
        }
    }
    /// Also the number of handles per arena
    #[inline]
    fn get_num_pages(&self) -> usize {
        self.length / PAGE_SIZE
    }
    #[inline]
    fn get_base<T>(&self) -> *const T {
        self.base as *const T
    }
    #[inline]
    fn get_base_mut<T>(&mut self) -> *mut T {
        self.base as *mut T
    }
    #[inline]
    fn get_heap(&self) -> *const BitmapEntry {
        self.base as *const BitmapEntry
    }
    #[inline]
    fn get_heap_mut(&mut self) -> *mut BitmapEntry {
        self.base as *mut BitmapEntry
    }
    pub fn alloc_page(&mut self) -> Option<RelativeHandle> {
        let heap = self.get_heap_mut();
        for i in 0..self.get_num_pages().div_ceil(BITMAP_BITS) {
            let value = unsafe { heap.add(i).read() };
            for j in 0..BITMAP_BITS {
                let mask = 1 << j;
                if value & mask == 0 {
                    unsafe { heap.add(i).write(value | mask) };
                    return Some(RelativeHandle((i * BITMAP_BITS + j).try_into().unwrap()));
                }
            }
        }
        None
    }
    pub fn free_page(&mut self, handle: RelativeHandle) {
        let heap = self.get_heap_mut();
        let value = unsafe { heap.add(handle.0 as usize).read() };
        let mask = 1 << (handle.0 as usize % BITMAP_BITS);
        assert_ne!(value & mask, 0);
        unsafe { heap.add(handle.0 as usize).write(value & !mask) };
        unreachable!()
    }
}

struct PhysicalAllocator {
    arenas: StaticVec<Arena, 16>,
}
impl PhysicalAllocator {
    pub const fn new() -> Self {
        Self{
            arenas: StaticVec::new_with_default(Arena::new(0, 0)),
        }
    }
}
static mut PHYSICAL_ALLOCATOR: PhysicalAllocator = PhysicalAllocator::new();

#[repr(C)]
pub struct MemoryEntry {
    virt: u64,
    phys: u64,
    page_count: u64,
    attribute: u64,
    type_: u32,
}

weak_typed_enum!(
pub MemoryType : u32 {
    RESERVED =  0,
    LOADER_CODE =  1,
    LOADER_DATA =  2,
    BOOT_SERVICES_CODE =  3,
    BOOT_SERVICES_DATA =  4,
    RUNTIME_SERVICES_CODE =  5,
    RUNTIME_SERVICES_DATA =  6,
    CONVENTIONAL =  7,
    UNUSABLE =  8,
    ACPI_RECLAIM =  9,
    ACPI_NON_VOLATILE = 10,
    MMIO = 11,
    MMIO_PORT_SPACE = 12,
    PAL_CODE = 13,
    PERSISTENT_MEMORY = 14,
    UNACCEPTED = 15,
    MAX = 16,
});

#[derive(Debug)]
pub struct Manager;
impl Manager {
    pub fn init(entries: *mut MemoryEntry, num_entries: usize) {
        for i in 0..num_entries {
            unsafe {
                let e = entries.add(i).read();
                if e.type_ == MemoryType::CONVENTIONAL {
                    kprint!("[pmm] add memory {:016x} (len = {} bytes)\r\n", e.phys, e.page_count * 4096);
                    let mut arena = Arena{
                        base: e.phys as usize,
                        length: e.page_count as usize * 4096,
                    };
                    // WE CANNOT MAP THE NULL PAGE, FUCK YOU RUST
                    if arena.base == 0 {
                        arena.base += PAGE_SIZE;
                        arena.length -= PAGE_SIZE;
                    }
                    arena.reset_heap();
                    (*&raw mut PHYSICAL_ALLOCATOR).arenas.push(arena);
                }
            }
        }
    }

    pub fn alloc_page() -> Handle {
        let mut first_handle = 0;
        unsafe {
            // Shut the fuck up
            let arenas = (&raw mut PHYSICAL_ALLOCATOR.arenas).as_mut().unwrap();
            for i in 0..(*arenas).len() {
                let last_handle = first_handle + (*arenas)[i].get_num_pages();
                if let Some(rel) = (*arenas)[i].alloc_page() {
                    return Handle(first_handle as u32 + rel.0 as u32);
                }
                first_handle = last_handle;
            }
        }
        unreachable!();
    }

    pub fn alloc_page_zeroed() -> Handle {
        let handle = Self::alloc_page();
        unsafe {
            handle.get_mut().write_bytes(0, PAGE_SIZE);
        }
        handle
    }

    pub fn free_page(handle: Handle) {
        let mut first_handle = 0;
        unsafe {
            // Shut the fuck up
            let arenas = (&raw mut PHYSICAL_ALLOCATOR.arenas).as_mut().unwrap();
            for i in 0..(*arenas).len() {
                let last_handle = first_handle + (*arenas)[i].get_num_pages();
                if handle.0 >= first_handle as u32 && handle.0 < last_handle as u32 {
                    (*arenas)[i].free_page(RelativeHandle((handle.0 as usize - first_handle) as u16));
                    return;
                }
                first_handle = last_handle;
            }
        }
        unreachable!()
    }
}

/// Represents a single page handle use this to quickly create page allocations :)
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Handle(u32);
impl Handle {
    #[inline]
    pub fn get(self) -> *const u8 {
        self.get_mut() as *const u8
    }
    pub fn get_mut(self) -> *mut u8 {
        let mut first_handle = 0;
        unsafe {
            // Shut the fuck up
            let arenas = (&raw mut PHYSICAL_ALLOCATOR.arenas).as_mut().unwrap();
            for i in 0..(*arenas).len() {
                let last_handle = first_handle + (*arenas)[i].get_num_pages();
                if self.0 >= first_handle as u32 && self.0 < last_handle as u32 {
                    let offset = self.0 - first_handle as u32;
                    return (*arenas)[i].get_base_mut::<u8>().add((offset * PAGE_SIZE as u32) as usize);
                }
                first_handle = last_handle;
            }
        }
        unreachable!()
    }
}

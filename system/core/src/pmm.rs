use crate::kprint;

unsafe extern "C" {
    unsafe static mut HEAP_START: u8;
    unsafe static HEAP_END: u8;
}

const BITMAP_BYTES: usize = core::mem::size_of::<u64>();
const BITMAP_BITS: usize = BITMAP_BYTES * 8;
const PAGE_SIZE: usize = 4096;

#[derive(Debug)]
pub struct Manager;
impl Manager {
    #[inline]
    fn get_num_pages() -> usize {
        let bytes = (&raw const HEAP_END) as *const _ as usize
            - (&raw const HEAP_START) as *const _ as usize;
        bytes / PAGE_SIZE
    }

    #[inline]
    fn get_heap() -> *const u64 {
        &raw const HEAP_START as *const u64
    }

    #[inline]
    fn get_heap_mut() -> *mut u64 {
        &raw mut HEAP_START as *mut u64
    }

    pub fn init() {
        assert!(Self::get_num_pages() < u16::MAX as usize);
        unsafe {
            let heap = Self::get_heap_mut();
            let bytes = Self::get_num_pages() / BITMAP_BITS * BITMAP_BYTES;
            kprint!("[pmm] {}, b={}", Self::get_num_pages(), bytes);
            heap.write_bytes(0, bytes);
            heap.add(0).write(1 << 0);
        }
    }

    pub fn alloc_page() -> Handle {
        let heap = Self::get_heap_mut();
        for i in 0..(Self::get_num_pages() / BITMAP_BITS) {
            let value = unsafe { heap.add(i).read() };
            for j in 0..BITMAP_BITS {
                let mask = 1 << j;
                if value & mask == 0 {
                    unsafe { heap.add(i).write(value | mask) };
                    return Handle((i * BITMAP_BITS + j) as u16);
                }
            }
        }
        unreachable!();
    }

    pub fn free_page(handle: Handle) {
        let heap = Self::get_heap_mut();
        let value = unsafe { heap.add(handle.0 as usize).read() };
        let mask = 1 << (handle.0 as usize % BITMAP_BITS);
        assert_ne!(value & mask, 0);
        unsafe { heap.add(handle.0 as usize).write(value & !mask) };
    }
}

/// Represents a single page handle use this to quickly create page allocations :)
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Handle(u16);
impl Handle {
    pub fn get(self) -> *const u8 {
        unsafe {
            Manager::get_heap().byte_add(PAGE_SIZE * self.0 as usize) as *const u8
        }
    }
    pub fn get_mut(self) -> *mut u8 {
        unsafe {
            Manager::get_heap_mut().byte_add(PAGE_SIZE * self.0 as usize) as *mut u8
        }
    }
}

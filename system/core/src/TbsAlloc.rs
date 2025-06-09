use core::alloc::{AllocError, Allocator, Layout};
use core::ptr::NonNull;

/// Max number of arenas
pub const MAX_ARENAS: usize = 64;
pub const CACHE_LINE_SIZE: usize = 64;

#[derive(Default, Debug, Clone, Copy)]
struct Arena {
    /// Must be in units of cache lines and aligned to 64 bytes
    base: usize,
    /// Must be in units of cache lines and aligned to 64 bytes
    length: usize,
}
impl Arena {
    pub fn new(base: *mut u8, length: usize) -> Self {
        assert_eq!((base as usize) % CACHE_LINE_SIZE, 0);
        assert_eq!(length % CACHE_LINE_SIZE, 0);
        Self{
            base: (base as usize) / CACHE_LINE_SIZE,
            length: length / CACHE_LINE_SIZE,
        }
    }
    pub fn get_base_ptr(self) -> *const u8 {
        (self.base * CACHE_LINE_SIZE) as *const u8
    }
    pub fn get_base_mut(self) -> *mut u8 {
        (self.base * CACHE_LINE_SIZE) as *mut u8
    }
}
pub struct PageAllocator {
    arenas: [Arena; MAX_ARENAS]
}
impl PageAllocator {
    pub fn new() -> Self {
        Self {
            arenas: [Arena::default(); MAX_ARENAS],
        }
    }
}

unsafe impl Allocator for PageAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        // if end > memory.buffer.len() {
        //     Err(AllocError)
        // } else {
        //     Ok(NonNull::from(slice))
        // }
        Err(AllocError)
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        // No-op: deallocation is unsupported in a bump allocator.
    }
}

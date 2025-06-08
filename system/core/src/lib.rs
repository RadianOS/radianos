#![no_std]
#![feature(str_from_raw_parts)]
#![feature(lang_items)]
#![feature(c_size_t)]
#![feature(pointer_is_aligned_to)]

use core::str;

pub mod policy;
pub mod pmm;
pub mod containers;
pub mod db;
pub mod vfs;
pub mod prelude;
pub mod vmm;
pub mod smp;

#[macro_export]
macro_rules! dense_bitfield {
    ($name:ident $repr:ident $($cap:ident = $value:expr,)*) => {
        #[repr(C)]
        #[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
        pub struct $name($repr);
        impl $name {
            $(pub const $cap: $repr = $value;)*
            pub fn contains(self, c: Self) -> bool {
                (self.0 & c.0) == c.0
            }
            pub fn with(self, c: $repr) -> Self {
                Self(self.0 | c)
            }
        }
    };
}

#[macro_export]
macro_rules! tagged_dense_bitfield {
    ($name:ident $repr:ident $tag:ident = $tag_mask:expr, $($cap:ident = $value:expr,)*) => {
        #[repr(C)]
        #[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
        pub struct $name($repr);
        impl $name {
            $(pub const $cap: $repr = $value;)*
            const $tag: $repr = $tag_mask;
            const TAG_SHIFT: $repr = 8;
            pub fn contains(self, c: Self) -> bool {
                (self.0 & c.0) == c.0
            }
            pub fn with(self, c: $repr) -> Self {
                Self(self.0 | c)
            }
            pub fn set_tag(self, c: $repr) -> Self {
                Self((self.0 & !Self::$tag) | ((c << Self::TAG_SHIFT) & Self::$tag))
            }
            pub fn get_tag(self) -> $repr {
                (self.0 & Self::$tag) >> Self::TAG_SHIFT
            }
        }
    };
}

#[macro_export]
macro_rules! dense_soa_generic_helper {
    (Monotonic $name:ident $repr:ty) => {
        pub $name: $crate::containers::StaticVec<$repr, 64>,
    }
}

#[macro_export]
macro_rules! kprint {
    ($($args:tt)*) => ({
        use core::fmt::Write;
        let _ = write!($crate::DebugSerial{}, $($args)*);
    });
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    if let Some(loc) = info.location() {
        kprint!("{}:{}: {}\r\n", loc.file(), loc.line(), info.message());
    }
    abort();
}

#[unsafe(no_mangle)]
extern "C" fn abort() -> ! {
    loop {
        unsafe {
            core::arch::asm!("pause");
        }
    }
}

pub struct DebugSerial;
impl core::fmt::Write for DebugSerial {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            Self::put_byte(b);
        }
        Ok(())
    }
}
impl DebugSerial {
    pub fn get_byte() -> u8 {
        let mut byte = 0;
        unsafe {
            core::arch::asm!(
                "in al, dx",
                out("al") byte,
                in("dx") 0x3f8
            );
        }
        byte
    }
    pub fn put_byte(b: u8) {
        unsafe {
            core::arch::asm!(
                "out dx, al",
                in("al") b,
                in("dx") 0x3f8
            );
        }
    }
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(s: *mut core::ffi::c_void, c: core::ffi::c_int, n: core::ffi::c_size_t) -> *mut core::ffi::c_void {
    for i in 0..n {
        *(s as *mut u8).add(i) = c as u8;
    }
    s
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(s1: *mut core::ffi::c_void, s2: *const core::ffi::c_void, n: core::ffi::c_size_t) -> *mut core::ffi::c_void {
    if n != 0 {
        let s1_slice = unsafe { core::slice::from_raw_parts_mut(s1.cast::<core::mem::MaybeUninit<u8>>(), n) };
        let s2_slice = unsafe { core::slice::from_raw_parts(s2.cast::<core::mem::MaybeUninit<u8>>(), n) };
        use core::mem::MaybeUninit;
        let s1_addr = s1.addr();
        let s2_addr = s2.addr();
        // Find the number of similar trailing bits in the two addresses to let
        // us find the largest possible chunk size
        let equal_trailing_bits_count = (s1_addr ^ s2_addr).trailing_zeros();
        let chunk_size = match equal_trailing_bits_count {
            0 => 1,
            1 => 2,
            2 => 4,
            3 => 8,
            _ => 16, // use u128 chunks for any higher alignments
        };
        let chunk_align_offset = s1.align_offset(chunk_size);
        let prefix_len = chunk_align_offset.min(n);

        // Copy "prefix" bytes
        for (s1_elem, s2_elem) in core::iter::zip(&mut s1_slice[..prefix_len], &s2_slice[..prefix_len]) {
            *s1_elem = *s2_elem;
        }

        if chunk_align_offset < n {
            fn copy_chunks_and_remainder<const N: usize, T: Copy>(
                dst: &mut [core::mem::MaybeUninit<u8>],
                src: &[core::mem::MaybeUninit<u8>],
            ) {
                // Check sanity
                assert_eq!(N, core::mem::size_of::<T>());
                assert_eq!(0, N % core::mem::align_of::<T>());
                assert!(dst.as_mut_ptr().is_aligned_to(N));
                assert!(src.as_ptr().is_aligned_to(N));
                // Split into "middle" and "suffix"
                let (dst_chunks, dst_remainder) = dst.as_chunks_mut::<N>();
                let (src_chunks, src_remainder) = src.as_chunks::<N>();
                // Copy "middle"
                for (dst_chunk, src_chunk) in core::iter::zip(dst_chunks, src_chunks) {
                    let dst_chunk_primitive: &mut MaybeUninit<T> =
                        unsafe { &mut *dst_chunk.as_mut_ptr().cast() };
                    let src_chunk_primitive: &MaybeUninit<T> =
                        unsafe { &*src_chunk.as_ptr().cast() };
                    *dst_chunk_primitive = *src_chunk_primitive;
                }
                // Copy "suffix"
                for (dst_elem, src_elem) in core::iter::zip(dst_remainder, src_remainder) {
                    *dst_elem = *src_elem;
                }
            }
            let s1_middle_and_suffix = &mut s1_slice[prefix_len..];
            let s2_middle_and_suffix = &s2_slice[prefix_len..];
            match chunk_size {
                1 => {
                    for (s1_elem, s2_elem) in core::iter::zip(s1_middle_and_suffix, s2_middle_and_suffix) {
                        *s1_elem = *s2_elem;
                    }
                }
                2 => {
                    copy_chunks_and_remainder::<2, u16>(s1_middle_and_suffix, s2_middle_and_suffix)
                }
                4 => {
                    copy_chunks_and_remainder::<4, u32>(s1_middle_and_suffix, s2_middle_and_suffix)
                }
                8 => {
                    copy_chunks_and_remainder::<8, u64>(s1_middle_and_suffix, s2_middle_and_suffix)
                }
                16 => copy_chunks_and_remainder::<16, u128>(
                    s1_middle_and_suffix,
                    s2_middle_and_suffix,
                ),
                _ => unreachable!(),
            }
        }
    }

    s1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(s1: *const core::ffi::c_void, s2: *const core::ffi::c_void, n: usize) -> core::ffi::c_int {
    use core::mem;
    let (div, rem) = (n / mem::size_of::<usize>(), n % mem::size_of::<usize>());
    let mut a = s1 as *const usize;
    let mut b = s2 as *const usize;
    for _ in 0..div {
        if *a != *b {
            for i in 0..mem::size_of::<usize>() {
                let c = *(a as *const u8).add(i);
                let d = *(b as *const u8).add(i);
                if c != d {
                    return c as core::ffi::c_int - d as core::ffi::c_int;
                }
            }
            unreachable!()
        }
        a = a.offset(1);
        b = b.offset(1);
    }

    let mut a = a as *const u8;
    let mut b = b as *const u8;
    for _ in 0..rem {
        if *a != *b {
            return *a as core::ffi::c_int - *b as core::ffi::c_int;
        }
        a = a.offset(1);
        b = b.offset(1);
    }
    0
}

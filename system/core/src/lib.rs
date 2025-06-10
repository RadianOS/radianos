#![allow(internal_features)]
#![no_std]
#![feature(str_from_raw_parts)]
#![feature(lang_items)]
#![feature(c_size_t)]
#![feature(pointer_is_aligned_to)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]

use core::str;
pub mod TbsAlloc;
pub mod containers;
pub mod cpu;
pub mod db;
pub mod pmm;
pub mod policy;
pub mod prelude;
pub mod smp;
pub mod styles;
pub mod task;
pub mod vfs;
pub mod vmm;

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
    ($qual:tt $name:ident : $repr:ident { $tag:ident = $tag_mask:expr, $($cap:ident = $value:expr,)* }) => {
        #[repr(C)]
        #[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
        $qual struct $name($repr);
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

#[macro_export]
macro_rules! const_assert {
    ($x:expr $(,)?) => {
        #[allow(unknown_lints, clippy::eq_op)]
        const _: [(); 0 - !{
            const ASSERT: bool = $x;
            ASSERT
        } as usize] = [];
    };
}

#[cfg(not(test))]
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
    pub fn get_byte() -> Option<u8> {
        #[allow(unused_assignments)]
        let mut byte = 0;
        unsafe {
            core::arch::asm!(
                "in al, dx",
                out("al") byte,
                in("dx") 0x3f8 + 5
            );
            if byte & 0x01 != 0 {
                core::arch::asm!(
                    "in al, dx",
                    out("al") byte,
                    in("dx") 0x3f8
                );
                Some(byte)
            } else {
                None
            }
        }
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
#[cfg(not(test))]
extern "C" fn eh_personality() {}

#[macro_export]
macro_rules! weak_typed_enum {
    ($qual:tt $name:ident : $repr:ty { $($elem:ident = $value:expr,)+ }) => {
        #[derive(Default, Debug, Clone, Copy, Ord, Eq, PartialEq, PartialOrd)]
        $qual struct $name($repr);
        impl $name {
            $(pub const $elem: $repr = $value;)+
        }
        impl From<u32> for $name {
            fn from(t: $repr) -> Self {
                Self(t)
            }
        }
        impl Into<u32> for $name {
            fn into(self) -> $repr {
                self.0
            }
        }
    };
}

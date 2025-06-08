#![no_std]
#![no_main]
#![feature(naked_functions)]

pub mod cap;
pub mod policy;
pub mod pmm;

#[macro_export]
macro_rules! dense_bitfield {
    ($name:ident $repr:ident $($cap:ident = $value:expr,)*) => {
        #[repr(C, packed)]
        #[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
        pub struct $name($repr);
        impl $name {
            $(const $cap: $repr = $value;)*
            pub fn contains(self, c: $repr) -> bool {
                (self.0 & c) == c
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
        #[repr(C, packed)]
        #[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
        pub struct $name($repr);
        impl $name {
            $(const $cap: $repr = $value;)*
            const $tag: $repr = $tag_mask;
            const TAG_SHIFT: $repr = 8;
            pub fn contains(self, c: $repr) -> bool {
                (self.0 & c) == c
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
macro_rules! dense_soa_generic {
    ($name:ident $($f_name:ident: $f_repr:ident)*) => {
        #[repr(C, packed)]
        struct $name {
            $($f_name: DenseBlock<$f_repr>,)*
        }
    }
}

struct DebugSerial;
impl core::fmt::Write for DebugSerial {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            unsafe {
                core::arch::asm!(
                    "out dx, al",
                    in("al") b,
                    in("dx") 0x3f8u16
                );
            }
        }
        Ok(())
    }
}
macro_rules! print {
    ($($args:tt)*) => {
        use core::fmt::Write;
        let _ = write!(DebugSerial{}, $($args)*);
    };
}

#[repr(C, packed)]
pub struct PageHandle(u16);

#[repr(C, packed)]
pub struct DenseBlock {
    page_handle: PageHandle,
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    //if let Some(loc) = info.location() {
        //kprint!("{}:{}: {}\r\n", loc.file(), loc.line(), info.message());
    //}
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

static RODATA_DUMMY: u8 = 255;
static mut DATA_DUMMY: u8 = 156;
static mut BSS_DUMMY: u8 = 0;

#[link_section = ".text.init"]
#[naked]
#[unsafe(no_mangle)]
unsafe extern "C" fn naked_start() {
    core::arch::naked_asm!(
        "cli",
        "lea rsp, STACK_TOP",
        "call rust_start",
    "2:",
        "cli",
        "hlt",
        "jmp 2b"
    );
}

#[unsafe(no_mangle)]
fn rust_start() {
    print!("hello rust world!\r\n");
    loop {
        unsafe {
            core::arch::asm!("pause");
        }
    }
}

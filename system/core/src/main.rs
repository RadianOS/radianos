#![no_std]
#![no_main]

use core::str;

pub mod caps;
pub mod policy;
pub mod pmm;
pub mod containers;
pub mod db;

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
macro_rules! dense_soa_generic {
    (struct $name:ident; $($f_name:ident: $f_repr:ty,)*) => {
        #[repr(C)]
        pub struct $name {
            $(pub $f_name: $crate::containers::StaticVec<$f_repr, 64>,)*
        }
    }
}

struct DebugSerial;
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

#[macro_export]
macro_rules! kprint {
    ($($args:tt)*) => ({
        use core::fmt::Write;
        let _ = write!($crate::DebugSerial{}, $($args)*);
    });
}

#[repr(C)]
pub struct PageHandle(u16);

#[repr(C)]
pub struct DenseBlock {
    page_handle: PageHandle,
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
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

/// Do not remove these or bootloader fails due to 0-sized section, thanks
#[allow(dead_code)]
static RODATA_DUMMY: u8 = 255;
#[allow(dead_code)]
static mut DATA_DUMMY: u8 = 156;
#[allow(dead_code)]
static mut BSS_DUMMY: u8 = 0;

#[link_section = ".text.init"]
#[unsafe(naked)]
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
    let db = db::Database::get_mut();
    let _ = caps::Capability::new().with(caps::Capability::WRITE_LOG);
    kprint!("creating worker #0\r\n");
    let start_task = policy::Action::default().with(policy::Action::START_TASK);
    db.workers.push(db::Worker::new());
    policy::PolicyEngine::add_rule(db, policy::PolicyRule{
        subject: db.find_from_str("worker_0").unwrap(),
        allowed: start_task
    });
    let res = policy::PolicyEngine::check(db, db.find_from_str("worker_0").unwrap(), start_task);
    kprint!("check policy? {}\r\n", res);


    let logo = include_str!("logo.txt");
    let mut last_char = ' ';
    for c in logo.chars() {
        if c != last_char {
            last_char = c;
            kprint!("{}", match c {
                'B' => "\x1b[0;91m",
                '&' => "\x1b[1;91m",
                '#' => "\x1b[0;91m",
                'P' => "\x1b[0;91m",
                'G' => "\x1b[1;31m",
                _ => "\x1b[0;0m",
            });
        }
        kprint!("{}", c);
    }
    kprint!("\x1b[0;0m\r\n");

    kprint!("kernel console, type <help>?\r\n");
    let mut mean_counter = 0;
    loop {
        let mut line = [0u8; 128];
        let mut index = 0;
        kprint!("\r\n");
        kprint!("RadianOS>");
        loop {
            let b = DebugSerial::get_byte();
            if b == b'\r' || index >= line.len() {
                let s = unsafe { str::from_raw_parts(line.as_ptr(), index) };
                kprint!("\r\n{}", s);
                if s.starts_with("help") {
                    kprint!("* mean - say something mean\r\n");
                } else if s.starts_with("mean") {
                    kprint!("{}", [
                        "go away\r\n",
                        "иди нахуй\r\n",
                        "vmovntdqa without the ntdqa\r\n",
                        "something mean\r\n"
                    ][mean_counter % 4]);
                    mean_counter += 1;
                }
                break;
            } else if b == 0x08 {
                kprint!("\x08 \x08");
                if index > 0 {
                    index -= 1;
                }
            } else if b != 0 {
                line[index] = b;
                index += 1;
                DebugSerial::put_byte(b);
            }
        }
    }
}

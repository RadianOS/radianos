#![no_std]
#![no_main]
#![feature(str_from_raw_parts)]

use core::str;

pub mod containers;
pub mod db;
pub mod pmm;
pub mod policy;
pub mod vfs;

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
        let byte;
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
    pmm::Manager::init();

    let db = db::Database::get_mut();
    kprint!("creating worker #0\r\n");
    let start_task = policy::Action::default().with(policy::Action::START_TASK);
    db.workers.push(db::Worker::new()); //kernel worker
    policy::PolicyEngine::add_rule(db, policy::PolicyRule::default()); //default rule
    policy::PolicyEngine::add_rule(
        db,
        policy::PolicyRule {
            subject: db.find_from_str("worker_0").unwrap(),
            allowed: start_task,
            capabilities: policy::Capability::new().with(policy::Capability::WRITE_LOG),
        },
    );

    let res =
        policy::PolicyEngine::check_action(db, db.find_from_str("worker_0").unwrap(), start_task);
    kprint!("check policy? {}\r\n", res);
    vfs::Manager::init(db);

    let logo = include_str!("logo.txt");
    let mut last_char = ' ';
    for c in logo.chars() {
        if c != last_char {
            last_char = c;
            kprint!(
                "{}",
                match c {
                    'B' => "\x1b[0;91m",
                    '&' => "\x1b[1;91m",
                    '#' => "\x1b[0;91m",
                    'P' => "\x1b[0;91m",
                    'G' => "\x1b[1;31m",
                    _ => "\x1b[0;0m",
                }
            );
        }
        kprint!("{}", c);
    }
    kprint!("\x1b[0;0m\r\n");

    kprint!("kernel console, type <help>?\r\n");
    let mut mean_counter = 0;
    let mut current_node = vfs::NodeHandle::default();
    let current_actor = db.find_from_str("worker_0").unwrap();
    loop {
        let mut line = [0u8; 128];
        let mut index = 0;
        kprint!("\r\n");
        kprint!("RadianOS>");
        loop {
            let b = DebugSerial::get_byte();
            if b == b'\r' || index >= line.len() {
                let s = unsafe { str::from_raw_parts(line.as_ptr(), index) };
                kprint!("\r\n");
                if s.starts_with("help") {
                    kprint!("* list - list all nodes\r\n");
                    kprint!("* tree - list ALL, and i mean ALL nodes\r\n");
                    kprint!("* mean - say something mean\r\n");
                    kprint!("* at - print current node\r\n");
                    kprint!("* cd <name> - change node\r\n");
                    kprint!("* write <data> - write line at current node\r\n");
                    kprint!("* rule_remove <index> - remove policy rule\r\n");
                    kprint!("* rule_list - lists all policy rules\r\n");
                } else if s.starts_with("rule_list") {
                    policy::PolicyEngine::for_each_policy_rule(db, |rule| {
                        kprint!("- {:?}\r\n", rule);
                    });
                } else if s.starts_with("rule_remove") {
                    let mut split = s.split_whitespace();
                    split.next();
                    if let Some(index) = split.next() {
                        if let Ok(index) = index.parse::<u16>() {
                            policy::PolicyEngine::remove_rule(db, policy::PolicyRuleHandle(index));
                        } else {
                            kprint!("invalid number\r\n");
                        }
                    } else {
                        kprint!("missing arg\r\n");
                    }
                } else if s.starts_with("write") {
                    let mut split = s.split_whitespace();
                    split.next();
                    if let Some(name) = split.next() {
                        let handle = vfs::Manager::get_node(db, current_node).get_provider();
                        let res = vfs::Manager::invoke_provider_write(
                            db,
                            *handle,
                            current_actor,
                            name.as_bytes(),
                        );
                        kprint!("\r\n{:?}\r\n", res);
                    } else {
                        kprint!("missing arg\r\n");
                    }
                } else if s.starts_with("tree") {
                    let print_node = |level, handle| {
                        let node = vfs::Manager::get_node(db, handle);
                        let name = node.get_name();
                        let level_prefix = [
                            "-", "--", "---", "----", "-----", "------", "-------", "--------",
                        ][level];
                        kprint!("{} {}\r\n", level_prefix, name);
                    };
                    // TODO: recurse, but without the stack overhead
                    vfs::Manager::for_each_children(db, current_node, |handle| {
                        print_node(0, handle);
                        vfs::Manager::for_each_children(db, handle, |handle| {
                            print_node(1, handle);
                            vfs::Manager::for_each_children(db, handle, |handle| {
                                print_node(2, handle);
                                vfs::Manager::for_each_children(db, handle, |handle| {
                                    print_node(3, handle);
                                });
                            });
                        });
                    });
                } else if s.starts_with("list") {
                    vfs::Manager::for_each_children(db, current_node, |handle| {
                        let node = vfs::Manager::get_node(db, handle);
                        let name = node.get_name();
                        kprint!("- {}\r\n", name);
                    });
                } else if s.starts_with("at") {
                    let name = vfs::Manager::get_node(db, current_node).get_name();
                    kprint!("{}\r\n", name);
                } else if s.starts_with("cd") {
                    let mut split = s.split_whitespace();
                    split.next();
                    if let Some(name) = split.next() {
                        if name == ".." {
                            let parent = vfs::Manager::get_node(db, current_node).get_parent();
                            current_node = *parent;
                        } else if let Some(handle) =
                            vfs::Manager::find_children(db, current_node, name)
                        {
                            current_node = handle;
                        } else {
                            kprint!("{} not found\r\n", name);
                        }
                    } else {
                        kprint!("missing arg\r\n");
                    }
                } else if s.starts_with("mean") {
                    kprint!(
                        "{}\r\n",
                        [
                            "go away\r\n",
                            "иди нахуй\r\n",
                            "vmovntdqa without the ntdqa\r\n",
                            "something mean\r\n"
                        ][mean_counter % 4]
                    );
                    mean_counter += 1;
                } else {
                    kprint!("{}???\r\n", s);
                }
                break;
            } else if b == 0x08 || b == 0x7F {
                kprint!("\x08 \x08");
                index = index.saturating_sub(1);
            } else if b != 0 {
                line[index] = b;
                index += 1;
                DebugSerial::put_byte(b);
            }
        }
    }
}

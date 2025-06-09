#![no_std]
#![no_main]
#![feature(str_from_raw_parts)]

use core::str;
use radian_core::{cpu, prelude::*, smp, vmm, task};

/// Do not remove these or bootloader fails due to 0-sized section, thanks
#[allow(dead_code)]
static RODATA_DUMMY: u8 = 255;
#[allow(dead_code)]
static mut DATA_DUMMY: u8 = 156;
#[allow(dead_code)]
static mut BSS_DUMMY: u8 = 0;

#[unsafe(link_section = ".text.init")]
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

/// Fine have your stack overhead
fn tree_traverse_node(db: &db::Database, handle: vfs::NodeHandle, level: usize) -> bool {
    let tree_print_node = |handle, level| {
        let node = vfs::Manager::get_node(db, handle);
        let name = node.get_name();
        let level_prefix = [
            "-", "--", "---", "----", "-----", "------", "-------", "--------",
        ][level];
        kprint!("{} {}\r\n", level_prefix, name);
    };
    let mut walk_had_children = false;
    vfs::Manager::for_each_children(db, handle, |handle| {
        tree_print_node(handle, level);
        tree_traverse_node(db, handle, level + 1);
        walk_had_children = true;
    });
    walk_had_children
}

unsafe extern "C" {
    unsafe static KERNEL_START: u8;
    unsafe static KERNEL_END: u8;
}

fn parse_literal(a: &str) -> Option<usize> {
    if a.starts_with("0x") {
        usize::from_str_radix(a.strip_prefix("0x").unwrap(), 16).ok()
    } else if a.starts_with("0h") {
        usize::from_str_radix(a.strip_prefix("0h").unwrap(), 16).ok()
    } else if a.starts_with("0b") {
        usize::from_str_radix(a.strip_prefix("0b").unwrap(), 2).ok()
    } else {
        a.parse::<usize>().ok()
    }
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn test_usermode_thunk() {
    core::arch::naked_asm!(
    "2:",
        "pause",
        "jmp 2b"
    );
}

struct ConsoleState<'a> {
    db: &'a mut db::Database,
    current_node: vfs::NodeHandle,
    current_aspace: vmm::AddressSpaceHandle,
    current_actor: db::ObjectHandle,
    current_task: task::TaskHandle,
    current_user: db::ObjectHandle,
}

struct Command {
    name: &'static str,
    desc: &'static str,
    handler: fn(&mut ConsoleState, &str),
}
const COMMANDS: [Command; 21] = [
    Command{
        name: "help",
        desc: "get help",
        handler: |state, s| {
            for c in COMMANDS.iter() {
                kprint!("* {}: {}\r\n", c.name, c.desc);
            }
        }
    },
    Command{
        name: "list",
        desc: "list all nodes",
        handler: |state, s| {
            vfs::Manager::for_each_children(state.db, state.current_node, |handle| {
                let node = vfs::Manager::get_node(state.db, handle);
                let name = node.get_name();
                kprint!("- {}\r\n", name);
            });
        }
    },
    Command{
        name: "mapl",
        desc: "<vaddr/paddr> <count> <flags>",
        handler: |state, s| {
            let mut split = s.split_whitespace();
            if let Some(Some(addr)) = split.next().map(parse_literal) {
                if let Some(Some(count)) = split.next().map(parse_literal) {
                    if let Some(Some(flags)) = split.next().map(parse_literal) {
                        vmm::Manager::map(
                            state.db,
                            state.current_aspace,
                            addr as u64,
                            addr as u64,
                            count,
                            flags as u64,
                        );
                        vmm::Manager::reload_cr3(state.db, state.current_aspace);
                    } else {
                        kprint!("invalid flags\r\n");
                    }
                } else {
                    kprint!("invalid count\r\n");
                }
            } else {
                kprint!("invalid addr\r\n");
            }
        }
    },
    Command{
        name: "map",
        desc: "<vaddr> <paddr> <count> <flags>",
        handler: |state, s| {
            let mut split = s.split_whitespace();
            if let Some(Some(vaddr)) = split.next().map(parse_literal) {
                if let Some(Some(paddr)) = split.next().map(parse_literal) {
                    if let Some(Some(count)) = split.next().map(parse_literal) {
                        if let Some(Some(flags)) = split.next().map(parse_literal) {
                            vmm::Manager::map(
                                state.db,
                                state.current_aspace,
                                paddr as u64,
                                vaddr as u64,
                                count,
                                flags as u64,
                            );
                            vmm::Manager::reload_cr3(state.db, state.current_aspace);
                        } else {
                            kprint!("invalid flags\r\n");
                        }
                    } else {
                        kprint!("invalid count\r\n");
                    }
                } else {
                    kprint!("invalid paddr\r\n");
                }
            } else {
                kprint!("invalid vaddr\r\n");
            }
        }
    },
    Command{
        name: "rule_list",
        desc: "list policy rules",
        handler: |state, s| {
            policy::Manager::for_each_policy_rule(state.db, |rule| {
                kprint!("- {:?}\r\n", rule);
            });
        }
    },
    Command{
        name: "tlb_reload",
        desc: "reload tlb",
        handler: |state, s| {
            vmm::Manager::reload_cr3(state.db, state.current_aspace);
        }
    },
    Command{
        name: "cd",
        desc: "change node",
        handler: |state, s| {
            let mut split = s.split_whitespace();
            if let Some(name) = split.next() {
                if name == ".." {
                    let parent = vfs::Manager::get_node(state.db, state.current_node).get_parent();
                    state.current_node = *parent;
                } else if let Some(handle) =
                    vfs::Manager::find_children(state.db, state.current_node, name)
                {
                    state.current_node = handle;
                } else {
                    kprint!("{} not found\r\n", name);
                }
            } else {
                kprint!("missing arg\r\n");
            }
        }
    },
    Command{
        name: "at",
        desc: "get current node",
        handler: |state, s| {
            let name = vfs::Manager::get_node(state.db, state.current_node).get_name();
            kprint!("{}\r\n", name);
        }
    },
    Command{
        name: "rule_remove",
        desc: "<index>",
        handler: |state, s| {
            let mut split = s.split_whitespace();
            if let Some(index) = split.next() {
                if let Ok(index) = index.parse::<u16>() {
                    policy::Manager::remove_rule(state.db, policy::PolicyRuleHandle(index));
                } else {
                    kprint!("invalid number\r\n");
                }
            } else {
                kprint!("missing arg\r\n");
            }
        }
    },
    Command{
        name: "t_user",
        desc: "test usermode",
        handler: |state, s| {
            task::Manager::switch_to_usermode(test_usermode_thunk as u64);
        }
    },
    Command{
        name: "write",
        desc: "<data> to current node",
        handler: |state, s| {
            let mut split = s.split_whitespace();
            if let Some(name) = split.next() {
                let handle = vfs::Manager::get_node(state.db, state.current_node).get_provider();
                let res = vfs::Manager::invoke_provider_write(
                    state.db,
                    *handle,
                    state.current_actor,
                    name.as_bytes(),
                );
                kprint!("\r\n{:?}\r\n", res);
            } else {
                kprint!("missing arg\r\n");
            }
        }
    },
    Command{
        name: "tree",
        desc: "list node tree",
        handler: |state, s| {
            tree_traverse_node(state.db,state. current_node, 0);
        }
    },
    Command{
        name: "aspace",
        desc: "<id> make new address space",
        handler: |state, s| {
            state.current_aspace = vmm::Manager::new_address_space(state.db, pmm::Manager::alloc_page_zeroed());
            kprint!("new aspace {:?}\r\n", state.current_aspace);
        }
    },
    Command{
        name: "worker",
        desc: "[id] make new worker or set",
        handler: |state, s| {
            let mut split = s.split_whitespace();
            if let Some(Some(index)) = split.next().map(parse_literal) {
                state.current_actor = db::ObjectHandle::new::<{db::ObjectHandle::WORKER}>(index as u16);
                kprint!("set {:?}\r\n", state.current_actor);
            } else {
                state.current_actor = task::Manager::new_worker(state.db, state.current_aspace);
                kprint!("new {:?}\r\n", state.current_actor);
            }
        }
    },
    Command{
        name: "new_task",
        desc: "make new task in worker",
        handler: |state, s| {
            let handle = task::Manager::new_task(state.db, state.current_actor).unwrap();
            state.current_task = handle;
            kprint!("new {:?}\r\n", state.current_task);
        }
    },
    Command{
        name: "rip3_to",
        desc: "jump to <addr>",
        handler: |state, s| {
            let mut split = s.split_whitespace();
            if let Some(Some(rip)) = split.next().map(parse_literal) {
                kprint!("jumping to {:016x}\r\n", rip);
                task::Manager::switch_to_usermode(rip as u64);
            }
        }
    },
    Command{
        name: "test_elf",
        desc: "test load elf",
        handler: |state, s| {
            let elf_bytes = include_bytes!("test.elf");
            task::Manager::load_elf_into_worker(state.db, state.current_actor, elf_bytes, true);
            vmm::Manager::reload_cr3(state.db, state.current_aspace);
            task::Manager::switch_to_usermode(0x200000);
        }
    },
    Command{
        name: "cli",
        desc: "disable interrupts",
        handler: |state, s| {
            cpu::Manager::set_interrupts::<false>();
        }
    },
    Command{
        name: "sti",
        desc: "enable interrupts",
        handler: |state, s| {
            cpu::Manager::set_interrupts::<false>();
        }
    },
    Command{
        name: "users",
        desc: "list users",
        handler: |state, s| {
            policy::Manager::for_each_user(state.db, |user| {
                kprint!("- {}", user.get_name());
            });
        }
    },
    Command{
        name: "groups",
        desc: "list groups",
        handler: |state, s| {
            policy::Manager::for_each_group(state.db, |group| {
                kprint!("- {}", group.get_name());
            });
        }
    },
];

#[unsafe(no_mangle)]
fn rust_start() {
    pmm::Manager::init();

    let db = db::Database::get_mut();
    smp::Manager::init();
    vmm::Manager::init(db);
    cpu::Manager::init();
    policy::Manager::init(db);
    task::Manager::init(db);
    vfs::Manager::init(db);

    // All of this is mostly a formality to "startup" the kernel worker and task
    db.aspaces.push(pmm::Handle::default()); //kernel space assumed :)
    let kernel_aspace = vmm::Manager::new_address_space(db, pmm::Manager::alloc_page_zeroed());
    vmm::Manager::reload_cr3(db, kernel_aspace);
    let start_task = policy::Action::default().with(policy::Action::START_TASK);
    let kernel_worker = task::Manager::new_worker(db, kernel_aspace);
    let kernel_task = task::Manager::new_task(db, kernel_worker).unwrap();
    policy::Manager::add_rule(db, policy::PolicyRule::default()); //default rule
    policy::Manager::add_rule(
        db,
        policy::PolicyRule {
            subject: kernel_worker,
            allowed: start_task,
            capabilities: policy::Capability::new().with(policy::Capability::WRITE_LOG),
        },
    );
    let res = policy::Manager::check_action(db, kernel_worker, start_task);
    assert_eq!(kernel_worker, db.find_from_str("worker_0").unwrap());
    kprint!("[policy] check policy? {res}\r\n");

    // Enable interrupts :)
    //cpu::Manager::set_interrupts::<true>();

    let logo = include_str!("logo.txt");
    let mut last_char = ' ';
    for c in logo.chars() {
        if c == '\n' || c == '\r' {
            if c != last_char {
                last_char = c;
                kprint!("\x1b[0;0m\r\n");
            } else {
                kprint!("\r\n");
            }
        } else if c != last_char {
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
            kprint!("{}", c);
        } else {
            kprint!("{}", c);
        }
    }
    kprint!("\x1b[0;0m\r\n");

    kprint!("kernel console, type <help>?\r\n");
    let mut state = ConsoleState{
        current_actor: db.find_from_str("worker_0").unwrap(),
        current_aspace: kernel_aspace,
        current_node: vfs::NodeHandle::default(),
        current_task: task::TaskHandle::default(),
        current_user: db::ObjectHandle::default(),
        db,
    };
    loop {
        let mut line = [0u8; 128];
        let mut index = 0;

        let user_name = policy::Manager::get_user(state.db, state.current_user).get_name();
        let hostname = "hostname";
        kprint!("RadianOS:{user_name}@{hostname}>");
        loop {
            let b = DebugSerial::get_byte();
            if b == b'\r' || index >= line.len() {
                kprint!("\r\n");

                let s = unsafe { str::from_raw_parts(line.as_ptr(), index) };
                let mut split = s.split_whitespace();
                if let Some(cmd) = split.next() {
                    if let Some(c) = COMMANDS.iter().find(|&c| c.name.eq_ignore_ascii_case(cmd)) {
                        let args = split.next().unwrap_or("");
                        (c.handler)(&mut state, args);
                    } else {
                        kprint!("unknown command <{}>\r\n", s);
                    }
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

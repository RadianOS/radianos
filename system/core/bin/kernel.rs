#![no_std]
#![no_main]
#![feature(str_from_raw_parts)]

extern crate alloc;
use crate::styles::{BBRRED, BRED, RADOS, RBRRED, RESET, USER};
use core::{arch::global_asm, str};
use iced_x86::Formatter;
use radian_core::{
    TbsAlloc,
    containers::{StaticString, StaticVec},
    cpu,
    prelude::*,
    smp,
    styles::{BRED, RBRRED, RESET},
    task, vmm, weak_typed_enum,
};

/// Do not remove these or bootloader fails due to 0-sized section, thanks
#[allow(dead_code)]
static RODATA_DUMMY: u8 = 255;
#[allow(dead_code)]
static mut DATA_DUMMY: u8 = 156;
#[allow(dead_code)]
static mut BSS_DUMMY: u8 = 0;

/// Fine have your stack overhead
fn tree_traverse_node(db: &db::Database, handle: vfs::NodeHandle, level: usize) -> bool {
    let tree_print_node = |handle, level| {
        let node = vfs::Manager::get_node(db, handle);
        let name = node.get_name();
        if level > 0 {
            for i in 0..level - 1 {
                kprint!("│   ");
            }
            kprint!("└── ");
        }
        kprint!("{}\r\n", name);
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

fn parse_boolean(a: &str) -> Option<bool> {
    if a.eq_ignore_ascii_case("yes")
        || a.eq_ignore_ascii_case("on")
        || a.eq_ignore_ascii_case("true")
        || a.eq_ignore_ascii_case("y")
        || a.eq_ignore_ascii_case("t")
        || a.eq_ignore_ascii_case("1")
    {
        Some(true)
    } else {
        Some(false)
    }
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn test_usermode_thunk() {
    core::arch::naked_asm!("2:", "pause", "jmp 2b");
}

struct ConsoleState<'a> {
    db: &'a mut db::Database,
    current_node: vfs::NodeHandle,
    current_aspace: vmm::AddressSpaceHandle,
    current_actor: db::ObjectHandle,
    current_task: task::TaskHandle,
    current_user: db::ObjectHandle,
    history_stack: StaticVec<StaticString<64>, 4>,
}

struct Command {
    name: &'static str,
    desc: &'static str,
    handler: fn(&mut ConsoleState, &str),
}

const COMMANDS: [Command; 28] = [
    Command {
        name: "help",
        desc: "get help",
        handler: |state, s| {
            for c in COMMANDS.iter() {
                kprint!("* {}: {}\r\n", c.name, c.desc);
            }
        },
    },
    Command {
        name: "clear",
        desc: "clear the screen",
        handler: |_state, _s| {
            // Clear screen ANSI escape sequence
            kprint!("\x1b[2J");
            // Move cursor to home position (0,0)
            kprint!("\x1b[H");
        },
    },
    Command {
        name: "list",
        desc: "list all nodes",
        handler: |state, s| {
            vfs::Manager::for_each_children(state.db, state.current_node, |handle| {
                let node = vfs::Manager::get_node(state.db, handle);
                let name = node.get_name();
                kprint!("- {}\r\n", name);
            });
        },
    },
    Command {
        name: "leak",
        desc: "<length> [align] leak this amt of memory",
        handler: |state, s| {
            let mut split = s.split_whitespace();
            if let Some(Some(size)) = split.next().map(parse_literal) {
                unsafe {
                    let p = alloc::alloc::alloc(alloc::alloc::Layout::array::<u8>(size).unwrap());
                    kprint!("{:?}\r\n", p);
                }
            }
        },
    },
    Command {
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
        },
    },
    Command {
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
        },
    },
    Command {
        name: "rule_list",
        desc: "list policy rules",
        handler: |state, s| {
            policy::Manager::for_each_policy_rule(state.db, |rule| {
                kprint!("- {:?}\r\n", rule);
            });
        },
    },
    Command {
        name: "tlb_reload",
        desc: "reload tlb",
        handler: |state, s| {
            vmm::Manager::reload_cr3(state.db, state.current_aspace);
        },
    },
    Command {
        name: "cd",
        desc: "change node or print current",
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
                let name = vfs::Manager::get_node(state.db, state.current_node).get_name();
                kprint!("{}\r\n", name);
            }
        },
    },
    Command {
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
        },
    },
    Command {
        name: "t_user",
        desc: "test usermode",
        handler: |state, s| {
            task::Manager::switch_to_usermode(test_usermode_thunk as u64);
        },
    },
    Command {
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
        },
    },
    Command {
        name: "tree",
        desc: "list node tree",
        handler: |state, s| {
            tree_traverse_node(state.db, state.current_node, 0);
        },
    },
    Command {
        name: "aspace",
        desc: "<id> make new address space",
        handler: |state, s| {
            state.current_aspace =
                vmm::Manager::new_address_space(state.db, pmm::Manager::alloc_page_zeroed());
            kprint!("new aspace {:?}\r\n", state.current_aspace);
        },
    },
    Command {
        name: "worker",
        desc: "[id] make new worker or set",
        handler: |state, s| {
            let mut split = s.split_whitespace();
            if let Some(Some(index)) = split.next().map(parse_literal) {
                state.current_actor =
                    db::ObjectHandle::new::<{ db::ObjectHandle::WORKER }>(index as u16);
                kprint!("set {:?}\r\n", state.current_actor);
            } else {
                state.current_actor = task::Manager::new_worker(state.db, state.current_aspace);
                kprint!("new {:?}\r\n", state.current_actor);
            }
        },
    },
    Command {
        name: "new_task",
        desc: "make new task in worker",
        handler: |state, s| {
            let handle = task::Manager::new_task(state.db, state.current_actor).unwrap();
            state.current_task = handle;
            kprint!("new {:?}\r\n", state.current_task);
        },
    },
    Command {
        name: "rip3_to",
        desc: "jump to <addr>",
        handler: |state, s| {
            let mut split = s.split_whitespace();
            if let Some(Some(rip)) = split.next().map(parse_literal) {
                kprint!("jumping to {:016x}\r\n", rip);
                task::Manager::switch_to_usermode(rip as u64);
            }
        },
    },
    Command {
        name: "test_elf",
        desc: "test load elf",
        handler: |state, s| {
            let elf_bytes = include_bytes!("test.elf");
            task::Manager::load_elf_into_worker(state.db, state.current_actor, elf_bytes, true);
            vmm::Manager::reload_cr3(state.db, state.current_aspace);
            task::Manager::switch_to_usermode(0x200000);
        },
    },
    Command {
        name: "int",
        desc: "<num> do an interrupts",
        handler: |state, s| {
            let mut split = s.split_whitespace();
            if let Some(Some(value)) = split.next().map(parse_literal) {
                // Originally was gonna do this with a recursive macro but
                // A) it crashed my compiler
                // B) it made the binary bigger
                unsafe extern "C" {
                    static mut quick_monitor_area: u8;
                }
                unsafe {
                    let p = (&raw mut quick_monitor_area);
                    p.add(0).write(0xcd); /* int <imm8> */
                    p.add(1).write(value as u8);
                    p.add(2).write(0xc3); /* retq */
                    let f: unsafe extern "C" fn() = core::mem::transmute(p);
                    f();
                }
            }
        },
    },
    Command {
        name: "sti",
        desc: "<on/off> enable/disable interrupts",
        handler: |state, s| {
            let mut split = s.split_whitespace();
            if let Some(Some(value)) = split.next().map(parse_boolean) {
                if value {
                    cpu::Manager::set_interrupts::<true>();
                } else {
                    cpu::Manager::set_interrupts::<false>();
                }
            }
        },
    },
    Command {
        name: "users",
        desc: "list users",
        handler: |state, s| {
            policy::Manager::for_each_user(state.db, |user| {
                kprint!("- {}\r\n", user.get_name());
            });
        },
    },
    Command {
        name: "groups",
        desc: "list groups",
        handler: |state, s| {
            policy::Manager::for_each_group(state.db, |group| {
                kprint!("- {}\r\n", group.get_name());
            });
        },
    },
    Command {
        name: "history",
        desc: "print local command history",
        handler: |state, s| {
            for c in 0..state.history_stack.len() {
                kprint!("{}\r\n", state.history_stack[c].as_str());
            }
        },
    },
    Command {
        name: "pal",
        desc: "print allocator info",
        handler: |state, s| {
            TbsAlloc::print_debug();
        },
    },
    Command {
        name: "poke",
        desc: "<addr> <value> <count> poke address (byte)",
        handler: |state, s| {
            let mut split = s.split_whitespace();
            if let Some(Some(addr)) = split.next().map(parse_literal) {
                if let Some(Some(value)) = split.next().map(parse_literal) {
                    if vmm::Manager::has_mapping_present(
                        &state.db,
                        state.current_aspace,
                        addr as u64,
                    ) {
                        let len = split
                            .next()
                            .map(parse_literal)
                            .unwrap_or(Some(1))
                            .unwrap_or(1);
                        let ptr = addr as *mut u8;
                        unsafe {
                            ptr.write_bytes(value as u8, len);
                        }
                    } else {
                        kprint!("area not mapped\r\n");
                    }
                }
            } else {
                kprint!("invalid addr\r\n");
            }
        },
    },
    Command {
        name: "peek",
        desc: "<addr> <count> peek address",
        handler: |state, s| {
            let mut split = s.split_whitespace();
            if let Some(Some(addr)) = split.next().map(parse_literal) {
                let len = split
                    .next()
                    .map(parse_literal)
                    .unwrap_or(Some(1))
                    .unwrap_or(1);
                if vmm::Manager::has_mapping_present(&state.db, state.current_aspace, addr as u64) {
                    let ptr = addr as *const u8;
                    unsafe {
                        for i in 0..len {
                            if i == 0 || i % 8 == 0 {
                                if i != 0 {
                                    kprint!("\r\n");
                                }
                                kprint!("{:016x} ", ptr as usize + i);
                            }
                            kprint!("{:02x} ", ptr.add(i).read());
                        }
                        kprint!("\r\n");
                    }
                } else {
                    kprint!("area not mapped\r\n");
                }
            } else {
                kprint!("invalid addr\r\n");
            }
        },
    },
    Command {
        name: "swap",
        desc: "initiate hotswap procedure",
        handler: |state, s| {
            cpu::Manager::set_interrupts::<false>(); //do not interrupt me
            //
            // After this point all things like lifetimes, statics, etc are worthless
            // we will replace them with our new rustacean friend :)
            //
            // "Please let this be the last thunk" - NO
            kprint!("awaiting new kernel...\r\n");
            let mut numbuf = StaticString::<16>::new();
            let mut index = 0;
            'get_size: loop {
                if let Some(b) = DebugSerial::get_byte() {
                    if b == 0x0d || b == 0x0a {
                        break 'get_size;
                    }
                    numbuf.bytes_mut()[index] = b;
                    index += 1;
                }
            }
            let file_size = numbuf.as_str().parse::<u64>().unwrap() - 8192;
            unsafe {
                core::arch::asm!(
                    "jmp hotswap_uart",
                    in("rcx") file_size,
                    options(noreturn),
                    options(nostack)
                );
            }
        },
    },
    Command {
        name: "dis",
        desc: "<addr> <count>",
        handler: |state, s| {
            let mut split = s.split_whitespace();
            if let Some(Some(addr)) = split.next().map(parse_literal) {
                let length = split.next().map(parse_literal).unwrap_or(Some(4)).unwrap();
                if vmm::Manager::has_mapping_present(&state.db, state.current_aspace, addr as u64) {
                    let slice = unsafe { core::slice::from_raw_parts(addr as *const u8, length) };
                    let mut decoder = iced_x86::Decoder::with_ip(
                        64,
                        slice,
                        addr as u64,
                        iced_x86::DecoderOptions::NONE,
                    );
                    let mut formatter = iced_x86::GasFormatter::new();
                    let mut instruction = iced_x86::Instruction::default();
                    let mut output = alloc::string::String::new();
                    while decoder.can_decode() {
                        decoder.decode_out(&mut instruction);
                        output.clear();
                        formatter.format(&instruction, &mut output);
                        kprint!("{output}\r\n");
                    }
                } else {
                    kprint!("area not mapped\r\n");
                }
            } else {
                kprint!("invalid addr\r\n");
            }
        },
    },
];

global_asm!(include_str!("head.S"), options(att_syntax));

pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    if s1.len() >= 16 || s2.len() >= 16 {
        return 0;
    }
    let insertion_cost = 1;
    let deletion_cost = 1;
    let subst_cost = 1;
    if s1.is_empty() || s2.is_empty() {
        [s1.len(), s2.len()][(s1.is_empty()) as usize]
    } else {
        let mut dist = [[0u32; 16]; 16];
        for i in 1..s1.len() {
            dist[i][0] = i as u32;
        }
        for i in 1..s2.len() {
            dist[0][i] = i as u32;
        }
        for j in 1..s2.len() {
            for i in 1..s1.len() {
                let cost = if s1.as_bytes()[i] == s2.as_bytes()[j] {
                    0
                } else {
                    subst_cost
                };
                let x = (dist[i - 1][j] + deletion_cost).min(dist[i][j - 1] + insertion_cost);
                dist[i][j] = (dist[i - 1][j - 1] + cost).min(x);
            }
        }
        dist[s1.len() - 1][s2.len() - 1] as usize
    }
}

#[unsafe(no_mangle)]
extern "sysv64" fn rust_start(entries: *mut pmm::MemoryEntry, num_entries: usize) {
    pmm::Manager::init(entries, num_entries);

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

    TbsAlloc::TbsAllocator::init(db, kernel_aspace);
    //    TbsAlloc::test_self();
    let ref_box = alloc::boxed::Box::new(065);
    kprint!("{ref_box:?}\r\n");

    // Enable interrupts :)
    //cpu::Manager::set_interrupts::<true>();

    let logo = include_str!("logo.txt");
    let mut last_char = ' ';
    for c in logo.chars() {
        if c == '\n' || c == '\r' {
            if c != last_char {
                last_char = c;
                kprint!("{RESET}\r\n");
            } else {
                kprint!("\r\n");
            }
        } else if c != last_char {
            last_char = c;
            kprint!(
                "{}",
                match c {
                    'B' => RBRRED,
                    '&' => BBRRED,
                    '#' => RBRRED,
                    'P' => RBRRED,
                    'G' => BRED,
                    _ => RESET,
                }
            );
            kprint!("{}", c);
        } else {
            kprint!("{}", c);
        }
    }
    kprint!("{RESET}\r\n");

    kprint!("kernel test console, type <help>?\r\n");
    let mut state = ConsoleState {
        current_actor: db.find_from_str("worker_0").unwrap(),
        current_aspace: kernel_aspace,
        current_node: vfs::NodeHandle::default(),
        current_task: task::TaskHandle::default(),
        current_user: db::ObjectHandle::default(),
        history_stack: StaticVec::new(),
        db,
    };
    loop {
        let mut line = StaticString::<64>::new();
        let mut index = 0;

        let user_name = policy::Manager::get_user(state.db, state.current_user).get_name();
        let hostname = "radiant-pc";
        kprint!("{RADOS}mRadianOS:{USER}{user_name}@{hostname}{RESET}>");
        loop {
            if let Some(b) = DebugSerial::get_byte() {
                if b == b'\r' || index >= line.max_len() {
                    kprint!("\r\n");
                    let s = line.as_str();
                    let mut split = s.split_whitespace();
                    if let Some(cmd) = split.next() {
                        if let Some(c) = COMMANDS.iter().find(|&c| c.name.eq_ignore_ascii_case(cmd))
                        {
                            let args = split.next().unwrap_or("");
                            (c.handler)(&mut state, args);
                        } else {
                            let mut min_dist = usize::MAX;
                            let mut min_cmd = None;
                            for c in COMMANDS.iter() {
                                let dist = levenshtein_distance(c.name, cmd);
                                if dist < min_dist {
                                    min_dist = dist;
                                    min_cmd = Some(c);
                                }
                            }
                            if let Some(c) = min_cmd {
                                kprint!("maybe you meant <{}>?\r\n", c.name);
                            } else {
                                kprint!("unknown command <{}>\r\n", s);
                            }
                        }
                    }
                    // Free queue :)
                    state.history_stack.push_fifo(line.clone());
                    break;
                } else if b == 0x08 || b == 0x7F {
                    if index > 0 {
                        kprint!("\x08 \x08");
                        index -= 1;
                    }
                } else {
                    line.bytes_mut()[index] = b;
                    index += 1;
                    DebugSerial::put_byte(b);
                }
            } else {
                unsafe {
                    core::arch::asm!("pause");
                }
            }
        }
    }
}

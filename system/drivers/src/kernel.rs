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

#[unsafe(no_mangle)]
fn rust_start() {
    pmm::Manager::init();

    let db = db::Database::get_mut();
    smp::Manager::init();
    vmm::Manager::init(db);
    cpu::Manager::init();
    task::Manager::init(db);

    // All of this is mostly a formality to "startup" the kernel worker and task
    db.aspaces.push(pmm::Handle::default()); //kernel space assumed :)
    let kernel_aspace = vmm::Manager::new_address_space(db, pmm::Manager::alloc_page_zeroed());
    vmm::Manager::reload_cr3(db, kernel_aspace);
    let start_task = policy::Action::default().with(policy::Action::START_TASK);
    let kernel_worker = task::Manager::new_worker(db, kernel_aspace);
    let kernel_task = task::Manager::new_task(db, kernel_worker).unwrap();
    policy::PolicyEngine::add_rule(db, policy::PolicyRule::default()); //default rule
    policy::PolicyEngine::add_rule(
        db,
        policy::PolicyRule {
            subject: kernel_worker,
            allowed: start_task,
            capabilities: policy::Capability::new().with(policy::Capability::WRITE_LOG),
        },
    );
    let res = policy::PolicyEngine::check_action(db, kernel_worker, start_task);
    assert_eq!(kernel_worker, db.find_from_str("worker_0").unwrap());

    kprint!("check policy? {}\r\n", res);
    vfs::Manager::init(db);

    let elf_bytes = include_bytes!("test.elf");
    let user_aspace = vmm::Manager::new_address_space(db, pmm::Manager::alloc_page_zeroed());
    let user_worker = task::Manager::new_worker(db, user_aspace);
    let user_task = task::Manager::new_task(db, user_worker).unwrap();
    task::Manager::load_elf_into_worker(db, user_worker, elf_bytes, true);
    vmm::Manager::reload_cr3(db, user_aspace);
    task::Manager::switch_to_usermode(0x200000);

    // Enable interrupts :)
    cpu::Manager::set_interrupts::<true>();

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
                    kprint!(
                        "* map <vaddr> <paddr> <count> <flags> - map page into current ASPACE\r\n"
                    );
                    kprint!(
                        "* map_r <vaddr> <paddr> <count> <flags> - map page into current ASPACE and reload tlb\r\n"
                    );
                    kprint!("* tlb_reload - reload cr3\r\n");
                    kprint!("* user - test usermode\r\n");
                } else if s.starts_with("user") {
                    task::Manager::switch_to_usermode(test_usermode_thunk as u64);
                } else if s.starts_with("map") {
                    let mut split = s.split_whitespace();
                    split.next();
                    if let Some(Some(vaddr)) = split.next().map(parse_literal) {
                        if let Some(Some(paddr)) = split.next().map(parse_literal) {
                            if let Some(Some(count)) = split.next().map(parse_literal) {
                                if let Some(Some(flags)) = split.next().map(parse_literal) {
                                    vmm::Manager::map(
                                        db,
                                        kernel_aspace,
                                        paddr as u64,
                                        vaddr as u64,
                                        count,
                                        flags as u64,
                                    );
                                    if s.starts_with("map_r") {
                                        vmm::Manager::reload_cr3(db, kernel_aspace);
                                    }
                                } else {
                                    kprint!("invalid flags");
                                }
                            } else {
                                kprint!("invalid count");
                            }
                        } else {
                            kprint!("invalid paddr");
                        }
                    } else {
                        kprint!("invalid vaddr");
                    }
                } else if s.starts_with("tlb_reload") {
                    vmm::Manager::reload_cr3(db, kernel_aspace);
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
                    tree_traverse_node(db, current_node, 0);
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

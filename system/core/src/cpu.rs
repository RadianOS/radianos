use crate::{const_assert, kprint};

const KERNEL_CODE_SEGMENT: usize = 0x08;
#[allow(dead_code)]
const KERNEL_DATA_SEGMENT: usize = 0x10;
#[allow(dead_code)]
const USER_CODE_SEGMENT: usize = 0x18;
#[allow(dead_code)]
const USER_DATA_SEGMENT: usize = 0x20;

#[derive(Debug)]
#[repr(C, packed)]
pub struct InterruptStackFrame {
    gpr: [u64; 16],
    irq: u64,
    //
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
}
impl InterruptStackFrame {
    pub const R15: usize = 8 * 0;
    pub const R14: usize = 8 * 1;
    pub const R13: usize = 8 * 2;
    pub const R12: usize = 8 * 3;
    pub const R11: usize = 8 * 4;
    pub const R10: usize = 8 * 5;
    pub const R9: usize = 8 * 6;
    pub const R8: usize = 8 * 7;
    pub const RDI: usize = 8 * 8;
    pub const RSI: usize = 8 * 9;
    pub const RDX: usize = 8 * 10;
    pub const RCX: usize = 8 * 11;
    pub const RBX: usize = 8 * 12;
    pub const RAX: usize = 8 * 13;
    pub const RBP: usize = 8 * 14;
    pub const RSP: usize = 8 * 15;
}


#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct InterruptDescriptor {
    offset_1: u16,
    selector: u16,
    ist: u8,
    type_attributes: u8,
    offset_2: u16,
    offset_3: u32,
    zero: u32,
}
impl InterruptDescriptor {
    pub const fn new_zero() -> Self {
        Self {
            offset_1: 0,
            offset_2: 0,
            offset_3: 0,
            selector: 0,
            type_attributes: 0,
            ist: 0,
            zero: 0,
        }
    }
    pub fn new(addr: u64, type_attributes: u8, ist: u8) -> Self {
        assert_eq!(addr % 16, 0);
        Self {
            offset_1: (addr & 0xffff) as u16,
            offset_2: ((addr >> 16) & 0xffff) as u16,
            offset_3: ((addr >> 32) & 0xffff_ffff) as u32,
            selector: KERNEL_CODE_SEGMENT as u16,
            ist,
            type_attributes,
            zero: 0,
        }
    }
    /// Interrupt gate
    pub fn new_interrupt_gate(addr: u64) -> Self {
        Self::new(addr, 0x8e, 0)
    }
    /// Trap gate
    #[allow(dead_code)]
    pub fn new_trap_gate(addr: u64) -> Self {
        Self::new(addr, 0x8f, 0)
    }
}

#[allow(dead_code)]
const EXCEPT_MEMMONIC: [&str; 22] = [
    "#DE", "#DB", "NMI", "#BP", "#OF", "#BR", "#UD", "#NM", "#DF", "CSEG", "#TS", "#NP", "#SS",
    "#GP", "#PF", "INTEL", "#MF", "#AC", "#MC", "#XM", "#VE", "#CP",
];

#[repr(C, packed)]
struct TaskStateSegment {
    resv1: u32,
    rsp0: u64,
    rsp1: u64,
    rsp2: u64,
    resv2: u64,
    ist1: u64,
    ist2: u64,
    ist3: u64,
    ist4: u64,
    ist5: u64,
    ist6: u64,
    ist7: u64,
    resv3: u64,
    resv4: u16,
    iopb: u16,
}
impl TaskStateSegment {
    pub const fn new_zero() -> Self {
        Self {
            resv1: 0,
            resv2: 0,
            resv3: 0,
            resv4: 0,
            rsp0: 0,
            rsp1: 0,
            rsp2: 0,
            iopb: 0,
            ist1: 0,
            ist2: 0,
            ist3: 0,
            ist4: 0,
            ist5: 0,
            ist6: 0,
            ist7: 0,
        }
    }
}

#[derive(Debug, Default)]
#[repr(C, packed)]
struct GlobalDescriptor {
    limit_1: u16,
    base_1: u16,
    base_2: u8,
    access: u8,
    flags: u8,
    base_3: u8,
}
impl GlobalDescriptor {
    pub const fn new_zero() -> Self {
        Self {
            limit_1: 0,
            base_1: 0,
            base_2: 0,
            base_3: 0,
            access: 0,
            flags: 0,
        }
    }
    pub const fn new(base: u64, limit: u32, access: u8, flags: u8) -> Self {
        Self {
            limit_1: (limit & 0xffff) as u16,
            base_1: (base & 0xffff) as u16,
            base_2: ((base >> 16) & 0xff) as u8,
            access,
            flags: (flags << 4) | ((limit >> 16) & 0x0f) as u8,
            base_3: ((base >> 24) & 0xff) as u8,
        }
    }
    pub const fn new_tss(base: u64, access: u8, flags: u8) -> (Self, Self) {
        let limit = (core::mem::size_of::<TaskStateSegment>() - 1) as u32;
        let lower = Self::new(base, limit, access, flags);
        let mut upper = GlobalDescriptor::new_zero();
        unsafe {
            ((&raw mut upper) as *mut u32)
                .add(0)
                .write((base >> 32) as u32);
        }
        (lower, upper)
    }
}

#[repr(C, packed)]
struct TableDescriptor {
    limit: u16,
    base: u64,
}

macro_rules! standard_interrupt_body {
    ($call:literal) => {
        core::arch::naked_asm!(
            "push rsp", //8
            "push rbp", //16
            "push rax", //24
            "push rbx", //32
            "push rcx", //40
            "push rdx", //48
            "push rsi", //56
            "push rdi", //64
            "push r8", //72
            "push r9", //80
            "push r10", //88
            "push r11", //96
            "push r12", //104
            "push r13", //112
            "push r14", //120
            "push r15", //128
            "mov rdi, rsp",
            $call,
            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop r11",
            "pop r10",
            "pop r9",
            "pop r8",
            "pop rdi",
            "pop rsi",
            "pop rdx",
            "pop rcx",
            "pop rbx",
            "pop rax",
            "pop rbp",
            "pop rsp",
            "iretq",
        );
    }
}

pub struct Manager;
impl Manager {
    // never inlined for obvious reasons
    #[unsafe(naked)]
    unsafe extern "C" fn load_gdt_thunk() {
        core::arch::naked_asm!(
            "lgdt [GLOBAL_GDT_R]",
            "push 0x08",
            "lea rax, 2f",
            "push rax",
            "retfq",
            "2:",
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            "ret",
        )
    }

    pub fn set_interrupts<const B: bool>() {
        unsafe {
            if B {
                core::arch::asm!("sti");
            } else {
                core::arch::asm!("cli");
            }
        }
    }

    fn load_idt_thunk() {
        unsafe {
            core::arch::asm!("lidt [GLOBAL_IDT_R]",);
        }
    }

    fn load_gdt(gdt: *mut [GlobalDescriptor]) {
        unsafe {
            let addr = (*gdt).as_ptr() as u64;
            GLOBAL_GDT_R.limit = (core::mem::size_of::<GlobalDescriptor>() * gdt.len()) as u16 - 1;
            GLOBAL_GDT_R.base = addr;
            Self::load_gdt_thunk();
        }
    }

    fn load_idt(idt: *mut [InterruptDescriptor]) {
        unsafe {
            let addr = (*idt).as_ptr() as u64;
            GLOBAL_IDT_R.limit =
                (core::mem::size_of::<InterruptDescriptor>() * idt.len()) as u16 - 1;
            GLOBAL_IDT_R.base = addr;
            Self::load_idt_thunk();
        }
    }

    #[unsafe(naked)]
    unsafe extern "C" fn dummy_int_handler() {
        #[unsafe(no_mangle)]
        fn dummy_int_handler_inner(rsp: u64) {
            let s = unsafe{(rsp as *mut InterruptStackFrame).as_mut()}.unwrap();
            kprint!("chto? {:?}\r\n", s);
            crate::abort();
        }
        standard_interrupt_body!("call dummy_int_handler_inner");
    }

    /// Call `reload_idt` to see reflected changes
    /// SAFETY: Address must not be below or in `.text.int_vector`
    pub fn register_interrupt(addr: u64, irq: usize) {
        unsafe {
            let base_rip = GLOBAL_IDT_ASM.0[irq].as_ptr() as u64 + 7;
            let b = u32::to_le_bytes((addr - base_rip).try_into().unwrap());
            GLOBAL_IDT_ASM.0[irq][3] = b[0];
            GLOBAL_IDT_ASM.0[irq][4] = b[1];
            GLOBAL_IDT_ASM.0[irq][5] = b[2];
            GLOBAL_IDT_ASM.0[irq][6] = b[3];
            crate::vmm::Manager::invalidate_single((&raw mut GLOBAL_IDT_ASM) as u64);
        }
    }

    pub fn init() {
        const_assert!(core::mem::size_of::<GlobalDescriptor>() == 64 / 8);
        kprint!("[cpu] loading new gdt\r\n");
        unsafe {
            let (low, high) =
                GlobalDescriptor::new_tss((&raw const GLOBAL_TSS) as u64, 0x89, 0x0);
            GLOBAL_GDT[5] = low;
            GLOBAL_GDT[6] = high;
            Self::load_gdt(&raw mut GLOBAL_GDT);
        }
        kprint!("[cpu] loading new idt\r\n");
        for i in 0..256 {
            unsafe {
                GLOBAL_IDT_ASM.0[i][1] = i as u8; //update pushed value (WHY IS THIS AT RUNTIME?) fuck rust x2
                GLOBAL_IDT[i] = InterruptDescriptor::new_interrupt_gate(GLOBAL_IDT_ASM.0[i].as_ptr() as u64);
            }
            Self::register_interrupt(Self::dummy_int_handler as u64, i);
        }
        Self::load_idt(&raw mut GLOBAL_IDT);
        kprint!("[cpu] set tss\r\n");
        unsafe {
            GLOBAL_TSS.rsp0 = (&STACK_TOP) as *const _ as u64;
            GLOBAL_TSS.iopb = 104;
            core::arch::asm!(
                "ltr ax",
                in("ax") (5 * 8) | 0,
            );
        }
    }
}

unsafe extern "C" {
    unsafe static STACK_TOP: u8;
}

// Globals
#[repr(align(16))]
struct IsrAsmCode([[u8; 16]; 256]);
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.int_vector")]
static mut GLOBAL_IDT_ASM: IsrAsmCode = IsrAsmCode([[
    0x6a, 0x00, /* push <irq> */
    0xe9, 0x00, 0x00, 0x00, /* jmp rip + <offs32> */
    0x90, 0x90, 0x90, 0x90, /* nop4 */
    0x90, 0x90, /* nop2 */
    0x90, 0x90, 0x90, 0x90, /* nop4 */
]; 256]);
// Evil TSS and GDT
static mut GLOBAL_TSS: TaskStateSegment = TaskStateSegment::new_zero();
static mut GLOBAL_GDT: [GlobalDescriptor; 7] = [
    GlobalDescriptor::new_zero(),                 //null
    GlobalDescriptor::new(0, 0xfffff, 0x9a, 0xa), //kernel code
    GlobalDescriptor::new(0, 0xfffff, 0x92, 0xc), //kernel data
    GlobalDescriptor::new(0, 0xfffff, 0xfa, 0xa), //user code
    GlobalDescriptor::new(0, 0xfffff, 0xf2, 0xc), //user data
    GlobalDescriptor::new_zero(),                 //tss (low)
    GlobalDescriptor::new_zero(),                 //tss (high)
];
static mut GLOBAL_IDT: [InterruptDescriptor; 256] = [InterruptDescriptor::new_zero(); 256];
/// Thanks fucking rust for being useless and not allowing to
/// place statics with pre-initialized linker values ffs, C can, why you can't?
#[unsafe(no_mangle)]
static mut GLOBAL_GDT_R: TableDescriptor = TableDescriptor { limit: 0, base: 0 };
#[unsafe(no_mangle)]
static mut GLOBAL_IDT_R: TableDescriptor = TableDescriptor { limit: 0, base: 0 };

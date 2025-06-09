use crate::{const_assert, kprint};

const KERNEL_CODE_SEGMENT: usize = 0x08;
const KERNEL_DATA_SEGMENT: usize = 0x10;
const USER_CODE_SEGMENT: usize = 0x18;
const USER_DATA_SEGMENT: usize = 0x20;

#[derive(Debug)]
#[repr(C, packed)]
pub struct InterruptStackFrame {
    ip: usize,
    cs: usize,
    flags: usize,
    sp: usize,
    ss: usize,
}

type InterruptFn = unsafe extern "x86-interrupt" fn(stack_frame: InterruptStackFrame);

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
        Self{
            offset_1: 0,
            offset_2: 0,
            offset_3: 0,
            selector: 0,
            type_attributes: 0,
            ist: 0,
            zero: 0,
        }
    }
    pub fn new(f: InterruptFn, type_attributes: u8, ist: u8) -> Self {
        let addr = f as u64;
        Self{
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
    pub fn new_interrupt_gate(f: InterruptFn) -> Self {
        Self::new(f, 0x8e, 0)
    }
    /// Trap gate
    pub fn new_trap_gate(f: InterruptFn) -> Self {
        Self::new(f, 0x8f, 0)
    }
}

const EXCEPT_MEMMONIC: [&'static str; 22] = [
    "#DE", "#DB", "NMI", "#BP", "#OF", "#BR",
    "#UD", "#NM", "#DF", "CSEG", "#TS", "#NP",
    "#SS", "#GP", "#PF", "INTEL", "#MF", "#AC",
    "#MC", "#XM", "#VE", "#CP",
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
        Self{
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
        Self{
            limit_1: 0,
            base_1: 0,
            base_2: 0,
            base_3: 0,
            access: 0,
            flags: 0,
        }
    }
    pub const fn new(base: u64, limit: u32, access: u8, flags: u8) -> Self {
        Self{
            limit_1: (limit & 0xffff) as u16,
            base_1: (base & 0xffff) as u16,
            base_2: ((base >> 16) & 0xff) as u8,
            access,
            flags: (flags << 4) | ((limit >> 16) & 0x0f) as u8,
            base_3: ((base >> 24) & 0xff) as u8,
        }
    }
    pub const fn new_tss(base: u64, limit: u32, access: u8, flags: u8) -> (Self, Self) {
        let lower = Self::new(base, limit, access, flags);
        let mut upper = GlobalDescriptor::new_zero();
        unsafe {
            ((&raw mut upper) as *mut u32).add(0).write((base >> 32) as u32);
        }
        (lower, upper)
    }
}

#[repr(C, packed)]
struct TableDescriptor {
    limit: u16,
    base: u64,
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
            GLOBAL_IDT_R.limit = (core::mem::size_of::<InterruptDescriptor>() * idt.len()) as u16 - 1;
            GLOBAL_IDT_R.base = addr;
            Self::load_idt_thunk();
        }
    }

    unsafe extern "x86-interrupt" fn dummy_int_handler(stack_frame: InterruptStackFrame) {
        kprint!("wora wora {:?}!\r\n", stack_frame);
    }

    /// Call `reload_idt` to see reflected changes
    pub fn register_interrupt(f: InterruptFn, irq: usize) {
        unsafe {
            GLOBAL_IDT[irq] = InterruptDescriptor::new_interrupt_gate(f);
        }
    }

    pub fn init() {
        const_assert!(core::mem::size_of::<GlobalDescriptor>() == 64 / 8);
        kprint!("[gdt] loading new gdt\r\n");
        unsafe {
            let (low, high) = GlobalDescriptor::new_tss((&raw const GLOBAL_TSS) as u64, 0xfffff, 0x89, 0x0);
            GLOBAL_GDT[5] = low;
            GLOBAL_GDT[6] = high;
            Self::load_gdt(&raw mut GLOBAL_GDT);
        }
        kprint!("[gdt] loading new idt\r\n");
        for i in 0..256 {
            Self::register_interrupt(Self::dummy_int_handler, i);
        }
        Self::load_idt(&raw mut GLOBAL_IDT);
    }
}

// Globals
static mut GLOBAL_TSS: TaskStateSegment = TaskStateSegment::new_zero();
static mut GLOBAL_GDT: [GlobalDescriptor; 7] = [
    GlobalDescriptor::new_zero(), //null
    GlobalDescriptor::new(0, 0xfffff, 0x9a, 0xa), //kernel code
    GlobalDescriptor::new(0, 0xfffff, 0x92, 0xc), //kernel data
    GlobalDescriptor::new(0, 0xfffff, 0xfa, 0xa), //user code
    GlobalDescriptor::new(0, 0xfffff, 0xf2, 0xc), //user data
    GlobalDescriptor::new_zero(), //tss (low)
    GlobalDescriptor::new_zero(), //tss (high)
];
static mut GLOBAL_IDT: [InterruptDescriptor; 256] = [InterruptDescriptor::new_zero(); 256];

/// Thanks fucking rust for being useless and not allowing to
/// place statics with pre-initialized linker values ffs, C can, why you can't?
#[unsafe(no_mangle)]
static mut GLOBAL_GDT_R: TableDescriptor = TableDescriptor{
    limit: 0,
    base: 0,
};
#[unsafe(no_mangle)]
static mut GLOBAL_IDT_R: TableDescriptor = TableDescriptor{
    limit: 0,
    base: 0,
};

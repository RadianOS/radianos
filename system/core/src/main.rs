#![no_std]
#![no_main]
#![feature(naked_functions)]

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

#[naked]
#[unsafe(no_mangle)]
unsafe extern "C" fn naked_start() {
    core::arch::naked_asm!(
    "2:",
        "cli",
        "hlt",
        "jmp 2b"
    );
}

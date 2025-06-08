#![no_std]
#![no_main]

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

#[unsafe(no_mangle)]
fn efi_main() {
    loop {
        unsafe {
        core::arch::asm!("pause"); }
    }
}

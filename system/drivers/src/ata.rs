#![no_std]
#![no_main]
#![feature(lang_items)]

use core::arch::asm;

#[cfg(not(test))]
#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    //if let Some(loc) = info.location() {
    //    kprint!("{}:{}: {}\r\n", loc.file(), loc.line(), info.message());
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

#[lang = "eh_personality"]
#[cfg(not(test))]
extern "C" fn eh_personality() {}

pub fn main() {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

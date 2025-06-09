#![no_std]
#![feature(lang_items)]

pub mod prelude;

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

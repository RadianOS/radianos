// Serial port I/O for QEMU COM1
use core::fmt::{self, Write};

const SERIAL_PORT: u16 = 0x3F8;

/// Write a byte to the serial port.
pub fn serial_write_byte(byte: u8) {
    unsafe {
        let mut line_status = 0u8;
        while line_status & 0x20 == 0 {
            core::arch::asm!(
                "in al, dx",
                out("al") line_status,
                in("dx") (SERIAL_PORT + 5)
            );
        }
        core::arch::asm!(
            "out dx, al",
            in("al") byte,
            in("dx") SERIAL_PORT
        );
    }
}

/// Write a string to the serial port.
pub fn serial_write_str(s: &str) {
    for byte in s.bytes() {
        serial_write_byte(byte);
    }
}

pub struct SerialWriter;
impl Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        serial_write_str(s);
        Ok(())
    }
}

pub fn log_write_fmt(args: fmt::Arguments) {
    let _ = SerialWriter.write_fmt(args);
}

#[macro_export]
macro_rules! boot_print {
    ($($arg:tt)*) => {{
        $crate::serial::log_write_fmt(core::format_args!($($arg)*));
    }};
}

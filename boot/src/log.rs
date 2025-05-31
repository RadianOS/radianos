extern crate alloc;

use core::fmt::{self, Write};
use crate::serial::serial_write_str;

struct SerialWriter;

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
macro_rules! log {
    ($($arg:tt)*) => {{
        $crate::log::log_write_fmt(core::format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        $crate::serial::serial_write_str("[INFO] ");
        $crate::log::log_write_fmt(core::format_args!($($arg)*));
        $crate::serial::serial_write_str("\n");
    }};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        $crate::serial::serial_write_str("[ERROR] ");
        $crate::log::log_write_fmt(core::format_args!($($arg)*));
        $crate::serial::serial_write_str("\n");
    }};
}
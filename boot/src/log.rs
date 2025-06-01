extern crate alloc;

use crate::serial::serial_write_str;
use core::fmt::{self, Write};


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
        use ansi_rgb::{Background, green};
        let tag = ::alloc::string::String::from("[INFO] ").bg(green()).to_string();
        $crate::serial::serial_write_str(&tag);
        $crate::log::log_write_fmt(core::format_args!($($arg)*));
        $crate::serial::serial_write_str("\n");
    }};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        use ansi_rgb::{Background, red};
        let tag = ::alloc::string::String::from("[ERROR] ").bg(red()).to_string();
        $crate::serial::serial_write_str(&tag);
        $crate::log::log_write_fmt(core::format_args!($($arg)*));
        $crate::serial::serial_write_str("\n");
    }};
}




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
    ($output:expr, $($arg:tt)*) => {{
        use uefi::proto::console::text::Color;
        $output.set_color(Color::LightGreen, Color::Black).unwrap();
        $output.output_string("[INFO] ").unwrap();
        $output.set_color(Color::White, Color::Black).unwrap();
        $output.output_string(&format!($($arg)*)).unwrap();
        $output.output_string("\n").unwrap();
    }};
}

#[macro_export]
macro_rules! error {
    ($output:expr, $($arg:tt)*) => {{
        use uefi::proto::console::text::Color;
        $output.set_color(Color::LightRed, Color::Black).unwrap();
        $output.output_string("[ERROR] ").unwrap();
        $output.set_color(Color::White, Color::Black).unwrap();
        $output.output_string(&format!($($arg)*)).unwrap();
        $output.output_string("\n").unwrap();
    }};
}





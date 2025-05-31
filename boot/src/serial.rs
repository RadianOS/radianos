use x86_64::instructions::port::Port;

/// QEMU uses the standard COM1 serial port at 0x3F8
const SERIAL_PORT: u16 = 0x3F8;

/// Write a byte to the serial port.
pub fn serial_write_byte(byte: u8) {
    unsafe {
        let mut line_status = Port::<u8>::new(SERIAL_PORT + 5);
        while (line_status.read() & 0x20) == 0 {} // Wait until empty

        let mut data = Port::new(SERIAL_PORT);
        data.write(byte);
    }
}

/// Write a string to the serial port.
pub fn serial_write_str(s: &str) {
    for byte in s.bytes() {
        serial_write_byte(byte);
    }
}

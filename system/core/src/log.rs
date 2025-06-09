mod asm;
use asm::{inportb, outportb};


mod log {
  fn serial_ready() -> bool {
    return inportb(0x3F8 + 5) & 0x20 != 0; // check 0x3F8 is able to write.
  }

  fn serial_write(c: char) {
    while !serial_ready() {}
    outportb(0x3F8, c as u8);
  }

  pub fn init() {
    outportb(0x3F8 + 1, 0x00); // disable interrupts
    
    outportb(0x3F8 + 3, 0x80);
    outportb(0x3F8 + 0, 0x03);
    outportb(0x3F8 + 1, 0x00);
    outportb(0x3F8 + 3, 0x03);
    outportb(0x3F8 + 2, 0xC7);
    
    outportb(0x3F8 + 4, 0x0B);
  }

  pub fn print(msg: Option<&str>) {
    if msg.is_none() { return; }
  
    for c in msg.unwrap().chars() { // write all characters one by one.. TODO: Add formatting.
          serial_write(c);
    }
  }

  pub fn warning(msg: Option<&str>) {
    print("")
  }
}

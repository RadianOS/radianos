mod asm;
use asm::{inportb, outportb};

/*
How to use:
log::error(" filepath/file.rs: I don't like this variable "); [ERROR]  filepath/file.rs: I don't like this variable.
log:warning(" filepath/file.rs: I'm so bad at rust."); -> [WARNING]  filepath/file.rs: I'm so bad at rust
log::info(" filepath/file.rs: I'm so tired, stop with this.") -> [INFO]  filepath/file.rs: I'm so tired of this.
*/

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

    serial_write('\n');
  }

  pub fn error(msg: Option<&str>) {
    if let Some(txt) = msg {
      print(Some(&format!("\033[31m [ERROR] \033[0m {}", txt)));
    }
  }
  pub fn warning(msg: Option<&str>) {
    if let Some(txt) = msg {
      print(Some(&format!("\033[95m [WARNING] \033[0m {}", txt)));
    }
  }

  pub fn info(msg: Option<&str>) {
    if let Some(txt) = msg {
      print(Some(&format!("\033[96m [INFO] \033[0m {}", txt)));
    }
  }
}

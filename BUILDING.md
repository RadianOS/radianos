## RadianOS Building

### Prerequisites:
- ```rustup target add x86_64-unknown-uefi``` :
Install the x86_64-unknown-uefi rustup target.
- No need to install rust nightly, already set in rust-toolchain.toml.

### Building:
- Bootloader - Go into the /boot/ cargo binary, and write: ```cargo build --target x86_64-unknown-uefi```, and you will get a .efi output.
  You will also need ```xorriso```, ```qemu```, ```make```, ```llvm``` and ```mtools```. With homebrew: ```brew install qemu mtools xorriso make cmake llvm```.

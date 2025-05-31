## RadianOS Building

### Prerequisites:
- ```rustup target add x86_64-unknown-uefi``` :
Install the x86_64-unknown-uefi rustup target.
- No need to install rust nightly, already set in rust-toolchain.toml.

### Building:
- Bootloader - Go into the /boot/ cargo binary, and write: ```cargo build --target x86_64-unknown-uefi```, and you will get a .efi output.
  You will also need ```xorriso```, ```qemu```, ```make```, ```llvm``` and ```mtools```. With homebrew: ```brew install qemu mtools xorriso make cmake llvm```.


### Running

- Bootloader: So after having all of those tools make sure to have mtools and run the following command inside `radianos/target/debug/x86_64-unknown-uefi` you will see the
`boot.efi` now how to boot into it? well first we have obtained our `boot.efi` file now we need to make sure to have the following OVMF files
```aiignore
/usr/share/OVMF/x64/OVMF_CODE.4m.fd
/usr/share/OVMF/x64/OVMF_VARS.4m.fd
```

we have to copy the `OVMF_VARS.4m.fd` as `OVMF_VARS.fd` inside the current path where the `boot.efi` is at , if you want to then you can place it somewhere else too but when im writing this, i have it where it was built , once you copied the OVMF_VARS file into the current dir run the following commands
```aiignore
mkfs.vfat -n EFIBOOT -C efidisk.img 10240
mcopy -i efidisk.img -s efidisk/* ::/
qemu-system-x86_64 \
        -drive if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/x64/OVMF_CODE.4m.fd \
        -drive if=pflash,format=raw,file=OVMF_VARS.fd \
        -drive format=raw,file=efidisk.img \
        -m 512
```

why did i tell you to place OVMF_VARS.4m.fd into the current dir? Its because i was getting permission denied error when i was using it from where it is at technically 
if i were to just run this
```aiignore
qemu-system-x86_64 \
        -drive if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/x64/OVMF_CODE.4m.fd \
        -drive if=pflash,format=raw,file=/usr/share/OVMF/x64/OVMF_VARS.4m.fd \
        -drive format=raw,file=efidisk.img \
        -m 512
```

where OVMF_VARS is at where it is then we will get the following error
`qemu-system-x86_64: Could not open '/usr/share/OVMF/x64/OVMF_VARS.4m.fd': Permission denied`
and so that's why!
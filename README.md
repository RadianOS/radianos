RadianOS is not just an OS it’s a foundation for creating modern, interoperable operating systems with shared architecture, tools, and performance at its core.

## Building

```sh
rustup target add x86_64-unknown-uefi
make run
```

## Hotswap kernel

On your Linux shell:
```sh
# Run QEMU and note the PTY where the serial will run
make run
```

Get a dumb terminal emulator, like `picocom` and run `picocom /dev/pts/1` (where `1` is replaced by whatever your PTY is).

Then to hotswap the kernel, in the ROS terminal type: `hotswap`. The kernel will stop all execution and cease everything. You can then send the binary file of the kernel like so:

```sh
HOTSWAP_TARGET=/dev/pts/1 make run hotswap-kernel
```

# DOCS (under development)
1. [What makes a system Radian?](https://github.com/RadianOS/radianos/blob/master/COMPILANCE.md)
2. [DRIVER RULES](https://github.com/RadianOS/radianos/blob/master/DRIVER_COMPILANCE.md)
3. [RADIAN_CORE (RadianOS-Compilant Runtime Framework)](https://github.com/RadianOS/radianos/blob/master/RADIAN_CORE.md)
4. [FILE STRUCTURE](https://github.com/RadianOS/radianos/blob/master/STRUCT.md)

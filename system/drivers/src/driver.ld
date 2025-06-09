ENTRY(driver_main);
MEMORY {
    ram (wxa) : ORIGIN = 0x200000, LENGTH = 32M
}
PHDRS {
    text PT_LOAD;
    data PT_LOAD;
    rodata PT_LOAD;
    bss PT_LOAD;
}
SECTIONS {
    .text : ALIGN(4K) {
        *(.text.init .text* .text.*)
    } >ram AT>ram :text
    .data : ALIGN(4K) {
        *(.data* .data.*)
        *(.sdata* .sdata.*)
    } >ram AT>ram :data
    .rodata : ALIGN(4K) {
        *(.rodata* .rodata.*)
        *(.srodata* .srodata.*)
    } >ram AT>ram :rodata
    .bss : ALIGN(4K) {
        *(COMMON)
        *(.bss* .bss.*)
        *(.sbss* .sbss.*)
    } >ram AT>ram :bss
}

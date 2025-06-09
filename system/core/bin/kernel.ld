ENTRY(naked_start);
MEMORY {
    ram (wxa) : ORIGIN = 0x100000, LENGTH = 32M
}
PHDRS {
    text PT_LOAD;
    data PT_LOAD;
    rodata PT_LOAD;
    bss PT_LOAD;
}
SECTIONS {
    .text : ALIGN(4K) {
        *(.text.init)
        *(.text.int_vector)
        *(.text* .text.*)
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
        // cant trust uefi
        . = ALIGN(4096);
        PROVIDE(STACK_BOTTOM = .);
        . += 8192;
        PROVIDE(STACK_TOP = .);
    } >ram AT>ram :bss

    /* The . pcrel is not reliable, just do it by hand! */
    PROVIDE(KERNEL_START = ORIGIN(ram));
    PROVIDE(KERNEL_END = ORIGIN(ram) + LENGTH(ram));
}

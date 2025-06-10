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
    PROVIDE(KERNEL_START = 0x1000);

    .text : ALIGN(4K) {
        *(.text.init)
        *(.text.hotswap)
        *(.text.int_vector)
        . = ALIGN(4096);
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
        PROVIDE(BSS_START = .);
        *(COMMON)
        *(.bss* .bss.*)
        *(.sbss* .sbss.*)
        . = ALIGN(4096);
        PROVIDE(STACK_BOTTOM = .);
        . += (4096 * 3);
        PROVIDE(STACK_TOP = .);
        PROVIDE(BSS_END = .);
    } >ram AT>ram :bss

    PROVIDE(KERNEL_END = .);
}

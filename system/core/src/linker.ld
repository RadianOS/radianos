ENTRY(naked_start);
MEMORY {
    ram  (wxa) : ORIGIN = 0x100000000, LENGTH = 128M
}
PHDRS {
    text PT_LOAD;
    data PT_LOAD;
    rodata PT_LOAD;
    bss PT_LOAD;
}
SECTIONS {
    .text : ALIGN(4K) {
        *(.text.init .text*)
    } >ram AT>ram :text
    .data : ALIGN(4K) {
        *(.data*)
    } >ram AT>ram :data
    .rodata : ALIGN(4K) {
        *(.rodata*)
    } >ram AT>ram :rodata
    .bss : ALIGN(4K) {
        *(COMMON)
        *(.bss*)
        // cant trust uefi
        . = ALIGN(4096);
        PROVIDE(stack_bottom = .);
        . += 8192;
        PROVIDE(stack_top = .);
    } >ram AT>ram :bss
}

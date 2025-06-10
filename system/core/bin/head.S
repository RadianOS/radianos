/* fuck you */
.section .text.init
.global naked_start
naked_start:
    cli
    lea STACK_TOP, %rsp
    /* Store values for re-entrant start (can't use stack obviously) */
    movq %rsi, uefi_param_rsi
    movq %rdi, uefi_param_rdi
    movq %cr3, %rax
    movq %rax, uefi_param_cr3
    callq rust_start
    ud2

.global reentrant_start
reentrant_start:
    cli
    lea STACK_TOP, %rsp
    /* Reload values */
    movq uefi_param_rsi, %rsi
    movq uefi_param_rdi, %rdi
    movq uefi_param_cr3, %rax
    movq %rax, %cr3
    /* May not clear last byte of BSS but it's fine
       since that's just stack which we overwrite with
       RIP on the next call. While UEFI bootloader does
       clear our BSS, we have to do this for hotswap */
    movq $BSS_END, %rcx
    subq $BSS_START, %rcx
    movq $BSS_START, %rdi
    rep stosb
    movq uefi_param_rdi, %rdi
    callq rust_start
    ud2

/* These must be within the first page */
uefi_param_rsi:
    .zero 8
uefi_param_rdi:
    .zero 8
uefi_param_cr3:
    .zero 8

.section .text.hotswap
.global hotswap_uart
hotswap_uart:
    movq $(0x100000 + 8192), %rsi
1:
    movw $(0x3f8 + 5), %dx
    inb %dx, %al
    andb $0x01, %al
    jz 1b
    movw $0x3f8, %dx
    inb %dx, %al
    movb %al, (%rsi)
    incq %rsi
    loop 1b
    jmp reentrant_start

.section .text
.global quick_monitor_area
quick_monitor_area:
    .zero 32

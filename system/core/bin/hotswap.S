/* fuck you */

.global _Zfnhotswap_header
_Zfnhotswap_header:
    movq $0x100000, %rsi
2:  movw $(0x3f8 + 5), %dx
1:  inb %dx, %al
    andb $0x01, %al
    jz 1b
    movw $0x3f8, %dx
    inb %dx, %al
    movb %al, (%rsi)
    incq %rsi
    decq %rcx
    jnz 2b
    /* start kernel again */
    jmp naked_start

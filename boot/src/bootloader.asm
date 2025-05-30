[org 0x7C00]       ; BIOS loads boot sector to 0x7C00

start:
    mov si, message

print_loop:
    lodsb           ; load byte at [SI] into AL, increment SI
    or al, al       ; check if null terminator
    jz hang
    mov ah, 0x0E    ; BIOS teletype output
    int 0x10
    jmp print_loop

hang:
    cli             ;clear interrupts
    hlt             ;halt CPU
    jmp hang

message:
    db "Hello!", 0

times 510 - ($ - $$) db 0
dw 0xAA55


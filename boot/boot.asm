; boot.asm - 512-byte boot sector loader (Stage 1)

BITS 16
ORG 0x7C00

start:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    sti

    ; Print "RadianOS" (debug)
    mov si, msg
.print:
    lodsb
    or al, al
    jz .done
    mov ah, 0x0E
    int 0x10
    jmp .print

.done:
    ; Load stage 2 loader (loader.bin) from disk (LBA 1)
    mov ah, 0x02          ; BIOS read sectors
    mov al, 1             ; Number of sectors
    mov ch, 0             ; Cylinder
    mov cl, 2             ; Sector (start at 2)
    mov dh, 0             ; Head
    mov dl, 0x80          ; First hard disk
    mov bx, 0x7E00        ; Buffer
    int 0x13              ; BIOS disk service
    jc disk_error

    jmp 0x0000:0x7E00     ; Jump to loaded stage 2

disk_error:
    hlt
    jmp $

msg db "RadianOS", 0

times 510 - ($ - $$) db 0
dw 0xAA55


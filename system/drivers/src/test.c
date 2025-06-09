/*
clang -ffreestanding -nostdlib -O2 -Wall -T ./system/drivers/src/driver.ld ./system/drivers/src/test.c -o ./system/drivers/src/test.elf
*/
char bss_thunk = 0;
char data_thunk = 1;
const char rodata_thunk = 4;
__attribute__((naked)) void driver_main() {
    asm volatile(
        "movq %rax, %rbx\r\n"
        "movq %rcx, %rax\r\n"
        "movq %rdx, %rax\r\n"
        "cli\r\n"
        "hlt\r\n"
    );
}

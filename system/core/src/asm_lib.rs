mod asm {
        pub fn outportb(port: u16, data: u8) {
                core::arch::naked_asm!(
                    asm("outb %1, %0" : : "dN" (port), "a" (data));
                )
            }
        
        pub fn inportb(port: u16) {
                core::arch::naked_asm!(
                    u8 r;
                    asm("inb %1, %0" : "=a" (r) : "dN" (port));
                    return r;
                )
        
                
            }
}

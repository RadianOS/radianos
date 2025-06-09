unsafe extern "C" fn outportb(const port: u16, const data: u8) {
        core::arch::naked_asm!(
            asm("outb %1, %0" : : "dN" (port), "a" (data));
        )
    }

unsafe extern "C" fn inportb(const port: u16) {
        core::arch::naked_asm!(
            u8 r;
            asm("inb %1, %0" : "=a" (r) : "dN" (port));
            return r;
        )

        
    }

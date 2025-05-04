#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "limine.h"
#include "drivers/vga/flanterm.h"
#include "drivers/vga/backends/fb.h"
#include "pmm.h"

size_t strlen(const char *str) {
    size_t len = 0;
    while (str[len]) {
        len++;
    }
    return len;
}

__attribute__((used, section(".limine_requests")))
volatile struct limine_memmap_request memmap_request = {
    .id = LIMINE_MEMMAP_REQUEST,
    .revision = 0
};

__attribute__((used, section(".limine_requests")))
static volatile LIMINE_BASE_REVISION(3);

__attribute__((used, section(".limine_requests")))
static volatile struct limine_framebuffer_request framebuffer_request = {
    .id = LIMINE_FRAMEBUFFER_REQUEST,
    .revision = 0
};

__attribute__((used, section(".limine_requests_start")))
static volatile LIMINE_REQUESTS_START_MARKER;

__attribute__((used, section(".limine_requests_end")))
static volatile LIMINE_REQUESTS_END_MARKER;

void *memmove(void *dest, const void *src, size_t n) {
    uint8_t *pdest = (uint8_t *)dest;
    const uint8_t *psrc = (const uint8_t *)src;
    if (src > dest) {
        for (size_t i = 0; i < n; i++) {
            pdest[i] = psrc[i];
        }
    } else if (src < dest) {
        for (size_t i = n; i > 0; i--) {
            pdest[i-1] = psrc[i-1];
        }
    }
    return dest;
}

int memcmp(const void *s1, const void *s2, size_t n) {
    const uint8_t *p1 = (const uint8_t *)s1;
    const uint8_t *p2 = (const uint8_t *)s2;
    for (size_t i = 0; i < n; i++) {
        if (p1[i] != p2[i]) {
            return p1[i] < p2[i] ? -1 : 1;
        }
    }
    return 0;
}

static void hcf(void) {
    for (;;) {
#if defined (__x86_64__)
        asm ("hlt");
#elif defined (__aarch64__) || defined (__riscv)
        asm ("wfi");
#elif defined (__loongarch64)
        asm ("idle 0");
#endif
    }
}

void write_string(struct flanterm_context *ft_ctx, const char *str) {
    flanterm_write(ft_ctx, str, strlen(str));
}

void write_hex(struct flanterm_context *ft_ctx, uintptr_t value) {
    char hex_buffer[20];
    const char hex_digits[] = "0123456789ABCDEF";
    int index = 0;
    if (value == 0) {
        hex_buffer[index++] = '0';
    } else {
        while (value > 0) {
            hex_buffer[index++] = hex_digits[value & 0xF];
            value >>= 4;
        }
    }
    for (int i = 0; i < index / 2; ++i) {
        char temp = hex_buffer[i];
        hex_buffer[i] = hex_buffer[index - 1 - i];
        hex_buffer[index - 1 - i] = temp;
    }
    hex_buffer[index] = '\0';
    write_string(ft_ctx, hex_buffer);
}

void write_number(struct flanterm_context *ft_ctx, size_t value) {
    char num_buffer[20];
    int index = 0;
    if (value == 0) {
        num_buffer[index++] = '0';
    } else {
        while (value > 0) {
            num_buffer[index++] = (value % 10) + '0';
            value /= 10;
        }
    }
    for (int i = 0; i < index / 2; ++i) {
        char temp = num_buffer[i];
        num_buffer[i] = num_buffer[index - 1 - i];
        num_buffer[index - 1 - i] = temp;
    }
    num_buffer[index] = '\0';
    write_string(ft_ctx, num_buffer);
}

void kmain(void) {
    struct limine_framebuffer *framebuffer = framebuffer_request.response->framebuffers[0];
    struct flanterm_context *ft_ctx = flanterm_fb_init(
        NULL,
        NULL,
        framebuffer->address,
        framebuffer->width,
        framebuffer->height,
        framebuffer->pitch,
        framebuffer->red_mask_size,
        framebuffer->red_mask_shift,
        framebuffer->green_mask_size,
        framebuffer->green_mask_shift,
        framebuffer->blue_mask_size,
        framebuffer->blue_mask_shift,
        NULL,
        NULL, NULL,
        NULL, NULL,
        NULL, NULL,
        NULL, 0, 0, 1,
        0, 0,
        0
    );
    if (LIMINE_BASE_REVISION_SUPPORTED == false) {
        hcf();
    }
    if (memmap_request.response == NULL) {
        write_string(ft_ctx, "No memory map found!\n");
        hcf();
    }
    if (framebuffer_request.response == NULL || framebuffer_request.response->framebuffer_count < 1) {
        write_string(ft_ctx, "No framebuffer found!\n");
        hcf();
    }
    pmm_init();
    if (total_pages == 0) {
        write_string(ft_ctx, "PMM: Total pages is 0\n");
        hcf();
    }
    if (bitmap == NULL) {
        write_string(ft_ctx, "PMM: Bitmap is NULL\n");
        hcf();
    } else {
        write_string(ft_ctx, "PMM: Bitmap size: ");
        write_number(ft_ctx, (total_pages + 7) / 8);
        write_string(ft_ctx, " bytes\n");
    }
    write_string(ft_ctx, "Memory Map:\n");
    for (size_t i = 0; i < memmap_request.response->entry_count; i++) {
        struct limine_memmap_entry *entry = memmap_request.response->entries[i];
        write_string(ft_ctx, "Entry ");
        write_number(ft_ctx, i);
        write_string(ft_ctx, ": Base: 0x");
        write_hex(ft_ctx, entry->base);
        write_string(ft_ctx, ", Length: 0x");
        write_hex(ft_ctx, entry->length);
        write_string(ft_ctx, ", Type: ");
        write_number(ft_ctx, entry->type);
        write_string(ft_ctx, "\n");
    }
    void *page = pmm_alloc();
    if (page) {
        write_string(ft_ctx, "PMM alloc successful! Page address: ");
        write_hex(ft_ctx, (uintptr_t)page);
        write_string(ft_ctx, "\n");
    } else {
        write_string(ft_ctx, "\033[1;31m(KERNEL PANIC!!!!) PMM alloc failed!\033[1;31m\n");
    }
    hcf();
}

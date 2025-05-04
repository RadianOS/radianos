#include "pmm.h"
#include "limine.h"
#include <stdbool.h>

#define PAGE_SIZE 4096

static uint8_t *bitmap = NULL;
static size_t total_pages = 0;
static uintptr_t highest_addr = 0;

static inline void set_bit(size_t bit) {
    bitmap[bit / 8] |= (1 << (bit % 8));
}

static inline void clear_bit(size_t bit) {
    bitmap[bit / 8] &= ~(1 << (bit % 8));
}

static inline bool test_bit(size_t bit) {
    return bitmap[bit / 8] & (1 << (bit % 8));
}

void *memset(void *s, int c, size_t n) {
    uint8_t *ptr = (uint8_t *)s;
    while (n--) {
        *ptr++ = (uint8_t)c;
    }
    return s;
}

void *memcpy(void *restrict dest, const void *restrict src, size_t n) {
    uint8_t *restrict pdest = (uint8_t *restrict)dest;
    const uint8_t *restrict psrc = (const uint8_t *restrict)src;
    while (n--) {
        *pdest++ = *psrc++;
    }
    return dest;
}

uintptr_t find_bitmap_location(size_t size) {
    size = (size + PAGE_SIZE - 1) & ~(PAGE_SIZE - 1);

    for (size_t i = 0; i < memmap_request.response->entry_count; i++) {
        struct limine_memmap_entry *entry = memmap_request.response->entries[i];
        if (entry->type == LIMINE_MEMMAP_RESERVED || entry->type == LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE) {
            uintptr_t start = entry->base;
            uintptr_t end = entry->base + entry->length;
            if (start % PAGE_SIZE != 0)
                start = (start + PAGE_SIZE - 1) & ~(PAGE_SIZE - 1);
            if (end - start >= size)
                return start;
        }
    }

    while (1) { __asm__("hlt"); }
}

void pmm_init() {
    for (size_t i = 0; i < memmap_request.response->entry_count; i++) {
        struct limine_memmap_entry *entry = memmap_request.response->entries[i];
        if (entry->type == LIMINE_MEMMAP_USABLE) {
            uintptr_t top = entry->base + entry->length;
            if (top > highest_addr) highest_addr = top;
        }
    }

    total_pages = highest_addr / PAGE_SIZE;
    size_t bitmap_size = total_pages / 8;

    bitmap = (uint8_t *)find_bitmap_location(bitmap_size);
    memset(bitmap, 0xFF, bitmap_size);

    for (size_t i = 0; i < memmap_request.response->entry_count; i++) {
        struct limine_memmap_entry *entry = memmap_request.response->entries[i];
        if (entry->type == LIMINE_MEMMAP_USABLE) {
            uintptr_t base = entry->base;
            uintptr_t end = entry->base + entry->length;
            for (uintptr_t addr = base; addr < end; addr += PAGE_SIZE) {
                size_t index = addr / PAGE_SIZE;
                clear_bit(index);
            }
        }
    }

    for (uintptr_t addr = (uintptr_t)bitmap;
         addr < (uintptr_t)(bitmap + bitmap_size);
         addr += PAGE_SIZE) {
        set_bit(addr / PAGE_SIZE);
    }
}

void *pmm_alloc() {
    for (size_t i = 0; i < total_pages; i++) {
        if (!test_bit(i)) {
            set_bit(i);
            return (void *)(i * PAGE_SIZE);
        }
    }
    return NULL;
}

void pmm_free(void *ptr) {
    size_t index = (uintptr_t)ptr / PAGE_SIZE;
    clear_bit(index);
}

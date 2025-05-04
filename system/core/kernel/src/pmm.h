#ifndef PMM_H
#define PMM_H

#include <stddef.h>
#include <stdint.h>
extern volatile struct limine_memmap_request memmap_request;
extern uint8_t *bitmap;
extern size_t total_pages;
void pmm_init(void);
void *pmm_alloc(void);
void pmm_free(void *ptr);

#endif

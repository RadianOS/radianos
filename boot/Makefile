CC=gcc
CFLAGS=-m32 -ffreestanding -nostdlib -fno-builtin -fno-stack-protector -Wall -Wextra
LD=ld
LDFLAGS=-m elf_i386 -T loader.ld

BUILD_DIR=build

all: $(BUILD_DIR)/boot.img

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

# Assemble stage 1 boot sector
$(BUILD_DIR)/boot1.bin: boot.asm | $(BUILD_DIR)
	$(ASM) $(ASFLAGS) $< -o $@

# Compile and link stage 2 loader
$(BUILD_DIR)/loader.bin: loader.c loader.ld | $(BUILD_DIR)
	$(CC) $(CFLAGS) -c loader.c -o $(BUILD_DIR)/loader.o
	$(LD) $(LDFLAGS) -o $@ $(BUILD_DIR)/loader.o

# Combine stage1 + stage2 to create bootable image
$(BUILD_DIR)/boot.img: $(BUILD_DIR)/boot1.bin $(BUILD_DIR)/loader.bin
	cat $^ > $@

clean:
	rm -rf $(BUILD_DIR)

.PHONY: all clean


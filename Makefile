# Variables
ifeq ($(OVMF_CODE),)
OVMF_CODE_1 := /usr/share/OVMF/OVMF_CODE_4M.fd
OVMF_CODE_2 := $(OVMF_PATH)/OVMF_CODE.fd #Arch
OVMF_CODE_3 := /usr/share/OVMF/x64/OVMF_CODE.4m.fd
OVMF_CODE := $(or $(and $(wildcard $(OVMF_CODE_1)),$(OVMF_CODE_1)),$(OVMF_CODE_2))
endif
ifeq ($(OVMF_VARS),)
OVMF_VARS_1 := /usr/share/OVMF/OVMF_VARS_4M.fd
OVMF_VARS_2 := $(OVM_PATH)/OVMF_VARS.fd #Arch
OVMF_VARS_3 := /usr/share/OVMF/x64/OVMF_VARS_4M.fd
OVMF_VARS := $(or $(and $(wildcard $(OVMF_VARS_1)),$(OVMF_VARS_1)),$(OVMF_VARS_2))
endif
# These should be set to the full path in your .zshrc/bashrc/shrc profile, not in the makefile

FAT_IMG := fat.img
ISO_FILE := radianos.iso
# Default to debug unless RELEASE=1 is set
BOOTLOADER_BUILD_DIR := $(CURDIR)/target/x86_64-unknown-uefi/$(if $(RELEASE),release,debug)
BOOTLOADER_PATH := $(BOOTLOADER_BUILD_DIR)/boot.efi

KERNEL_BUILD_DIR := $(CURDIR)/target/x86_64-unknown-none/$(if $(RELEASE),release,debug)
KERNEL_PATH := $(KERNEL_BUILD_DIR)/kernel

RUSTC_FLAGS := \
	-C opt-level=3 \
	-L $(KERNEL_BUILD_DIR)/deps \
	--crate-type staticlib \
	--target x86_64-unknown-none

ESP_DIR := esp/efi/boot

.PHONY: run clean build-kernel build-bootloader check-artifacts esp fat iso qemu rust-clean

run: iso
	# Run with QEMU
	$(MAKE) qemu

build-bootloader:
	cargo build $(if $(RELEASE),--release,) --target x86_64-unknown-uefi --bin boot

build-kernel:
	clang -ffreestanding -nostdlib -O2 -Wall -T ./system/drivers/src/driver.ld ./system/core/bin/test.c -o ./system/core/bin/test.elf
	RUSTFLAGS='-C link-arg=-Tsystem/core/bin/kernel.ld -C relocation-model=static' cargo build $(if $(RELEASE),--release,) --target x86_64-unknown-none --bin kernel

build-drivers:
	clear && objdump -t target/x86_64-unknown-none/debug/deps/libradian_core-eee6dde371a58c3d.a | awk '/a/ {print "PROVIDE( \"" $4 "\" = " $1 ");"}' | awk '!seen[$2]++' | sort
	RUSTFLAGS='-C link-arg=-Tsystem/drivers/src/driver.ld -C relocation-model=pic' cargo build $(if $(RELEASE),--release,) --target x86_64-unknown-none --bin ata

check-artifacts: build-drivers build-kernel build-bootloader
	@if [ ! -f $(BOOTLOADER_PATH) ]; then echo "Error: boot.efi not found!"; exit 1; fi

esp: check-artifacts
	mkdir -p $(ESP_DIR)
	cp $(BOOTLOADER_PATH) $(ESP_DIR)/bootx64.efi
	cp $(KERNEL_PATH) $(ESP_DIR)/kernel.elf

fat: esp
	dd if=/dev/zero of=$(FAT_IMG) bs=1M count=33
	mformat -i $(FAT_IMG) -F ::
	mmd -i $(FAT_IMG) ::/EFI
	mmd -i $(FAT_IMG) ::/EFI/BOOT
	mcopy -i $(FAT_IMG) $(ESP_DIR)/bootx64.efi ::/EFI/BOOT
	mcopy -i $(FAT_IMG) $(ESP_DIR)/kernel.elf ::/EFI/BOOT/KERNEL

iso: fat
	mkdir -p iso
	cp $(FAT_IMG) iso/
	xorriso -as mkisofs -R -f -e $(FAT_IMG) -no-emul-boot -o $(ISO_FILE) iso

qemu: iso
	qemu-system-x86_64 \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
		-drive format=raw,file=$(ISO_FILE) \
		-smp 4 -m 4G -cpu max -s -d unimp,guest_errors,int \
		-monitor stdio \
		-device qemu-xhci -device usb-kbd -audiodev pa,id=snd0 -machine pcspk-audiodev=snd0 -M q35 --no-reboot

qemu-nographic: iso # yo stop allocating so much my pc only has 8G atleast allocate 2G
	qemu-system-x86_64 \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
		-drive format=raw,file=$(ISO_FILE) \
		-smp 4 -m 4G -cpu max -s -d unimp,guest_errors,int \
		-device qemu-xhci -device usb-kbd -audiodev pa,id=snd0 -machine pcspk-audiodev=snd0 -M q35 --no-reboot -nographic

clean:
# Delete: the ISO, FAT image, ESP directory, and the build artifacts
	rm -rf iso $(FAT_IMG) $(ESP_DIR) $(ISO_FILE)

rust-clean:
	cargo clean

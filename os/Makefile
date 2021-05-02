TARGET      := riscv64gc-unknown-none-elf
MODE        := debug
KERNEL_FILE := target/$(TARGET)/$(MODE)/os

OBJDUMP     := rust-objdump --arch-name=riscv64
OBJCOPY     := rust-objcopy --binary-architecture=riscv64

.PHONY: build asm clean qemu

build:
	cargo build

asm:
	@$(OBJDUMP) -d $(KERNEL_FILE)

clean:
	@cargo clean

qemu: build
	@qemu-system-riscv64 \
		-machine virt \
                -cpu rv64 \
		-smp 4     \
		-m 128M   \
		-nographic \
		-bios none \
		-serial mon:stdio \
		-kernel $(KERNEL_FILE)

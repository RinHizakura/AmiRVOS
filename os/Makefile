ifeq ("$(RELEASE)", "1")
	MODE := release
	OPT  := "--$(MODE)"
else
	MODE := debug
	OPT  :=
endif

CURDIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
TARGET      := riscv64gc-unknown-none-elf
KERNEL_FILE := target/$(TARGET)/$(MODE)/os
RFS_FILE    := ../mkfs/fs.img
GIT_HOOKS   := $(CURDIR)/../.git/hooks/applied

.PHONY: build asm clean qemu

all: build $(GIT_HOOKS)

$(GIT_HOOKS):
	@cd $(CURDIR)/..; scripts/install-git-hooks

build:
	cargo build $(OPT)

clean:
	@cargo clean

qemu: build
	@qemu-system-riscv64          \
		-machine virt         \
		-cpu rv64             \
		-smp 4                \
		-m 128M               \
		-nographic            \
		-bios none            \
		-serial mon:stdio     \
		-global virtio-mmio.force-legacy=false                   \
		-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
		-drive file=$(RFS_FILE),if=none,format=raw,id=x0         \
		-kernel $(KERNEL_FILE)

GDBSTUB_COMM := 127.0.0.1:1234
debug:
	RUST_GDB=riscv64-unknown-linux-gnu-gdb rust-gdb \
		-ex "file $(KERNEL_FILE)"               \
		-ex "target remote $(GDBSTUB_COMM)"     \

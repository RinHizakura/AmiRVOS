ifeq ("$(RELEASE)", "1")
	MODE := release
	OPT  := "--$(MODE)"
else
	MODE := debug
	OPT  :=
endif

KERNEL        := os
MKFS          := mkfs
RFS_FILE_NAME := fs.img

CURDIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
TARGET      := riscv64gc-unknown-none-elf
KERNEL_FILE := $(KERNEL)/target/$(TARGET)/$(MODE)/os
RFS_FILE    := $(MKFS)/$(RFS_FILE_NAME)
GIT_HOOKS   := $(CURDIR)/.git/hooks/applied

.PHONY: build asm clean qemu

all: $(KERNEL_FILE) $(RFS_FILE) $(GIT_HOOKS)

$(GIT_HOOKS):
	scripts/install-git-hooks

# Force the generation of FS image
rebuild:
	cargo -Z unstable-options -C $(MKFS) run $(OPT) $(RFS_FILE_NAME)

$(KERNEL_FILE):
	cargo -Z unstable-options -C $(KERNEL) build $(OPT)

$(RFS_FILE):
	cargo -Z unstable-options -C $(MKFS) run $(OPT) $(RFS_FILE_NAME)

clean:
	@cargo -Z unstable-options -C $(KERNEL) clean
	@cargo -Z unstable-options -C $(MKFS) clean
	$(RM) $(RFS_FILE)

qemu: $(KERNEL_FILE) $(RFS_FILE)
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

CARGO := cargo
DOCKER := docker
TAR := tar

NEEDLE_CORE := needle-core
TARGET_LINUX := x86_64-unknown-linux-gnu
TARGET_WINDOWS := x86_64-pc-windows-gnu

DOCKER_IMAGE_NAME := buildenv
LINUX_IMAGE_TAG := linux
WINDOWS_IMAGE_TAG := windows

SPIRV_DIR := shaders/spv

all: build run

# Rust code
clean:
	@$(CARGO) clean
	@(cd $(NEEDLE_CORE) && $(CARGO) clean)
	@rm -rvf ${SPIRV_DIR}

pkg: pkg-linux_docker pkg-windows_docker
	@cp -v target/x86_64-unknown-linux-gnu/release/needle .
	@cp -v target/x86_64-pc-windows-gnu/release/needle.exe .
	@sha256sum needle \
		| tee needle.sha256
	@sha256sum needle.exe \
		| tee needle.exe.sha256

fmt:
	@$(CARGO) fmt

build: fmt
	@$(CARGO) build

release: fmt
	@$(CARGO) build --release

run:
	@$(CARGO) run --release

pkg-linux:
	@$(CARGO) build --release --target=${TARGET_LINUX}

pkg-linux_docker:
	@$(DOCKER) run --rm -it -v $(shell pwd):/app ${DOCKER_IMAGE_NAME}:${LINUX_IMAGE_TAG} \
		bash -c "make pkg-linux"

include windows.mk
include shader.mk

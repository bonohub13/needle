CARGO := cargo
DOCKER := docker

NEEDLE_CORE := needle-core
TARGET_LINUX := x86_64-unknown-linux-gnu
TARGET_WINDOWS := x86_64-pc-windows-gnu

DOCKER_IMAGE_NAME := buildenv
LINUX_IMAGE_TAG := linux
WINDOWS_IMAGE_TAG := windows

all: build run

# Rust code
clean:
	$(CARGO) clean
	(cd $(NEEDLE_CORE) && $(CARGO) clean)

pkg: clean pkg-linux_docker pkg-windows_docker

fmt:
	$(CARGO) fmt

build: fmt
	$(CARGO) build

release: fmt
	$(CARGO) build --release

run: clean
	$(CARGO) run --release

pkg-linux:
	$(CARGO) build --release --target=${TARGET_LINUX}

pkg-windows:
	$(CARGO) build --release --target=${TARGET_WINDOWS}

pkg-linux_docker:
	$(DOCKER) run --rm -it -v $(shell pwd):/app ${DOCKER_IMAGE_NAME}:${LINUX_IMAGE_TAG} \
		bash -c "make pkg-linux"

pkg-windows_docker:
	$(DOCKER) run --rm -it -v $(shell pwd):/app ${DOCKER_IMAGE_NAME}:${WINDOWS_IMAGE_TAG} \
		bash -c "make pkg-windows"

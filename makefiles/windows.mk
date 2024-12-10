CARGO := cargo
DOCKER := docker

TARGET_WINDOWS := x86_64-pc-windows-gnu

RUST_DOCKER_IMAGE_NAME := buildenv
WINDOWS_IMAGE_TAG := windows

windows-update:
	@$(CARGO) update --target=${TARGET_WINDOWS}

pkg-windows:
	@$(CARGO) build --release --target=${TARGET_WINDOWS}

pkg-windows_docker:
	@$(DOCKER) run --rm -it -v $(shell pwd):/app \
		${RUST_DOCKER_IMAGE_NAME}:${WINDOWS_IMAGE_TAG} \
		bash -c "make pkg-windows"

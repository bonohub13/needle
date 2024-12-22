CARGO := cargo
DOCKER := docker

TARGET_LINUX := x86_64-unknown-linux-gnu

RUST_DOCKER_IMAGE_NAME := buildenv
LINUX_IMAGE_TAG := linux

linux-update:
	@$(CARGO) update --target=${TARGET_LINUX}

pkg-linux:
	$(CARGO) build --release --target=${TARGET_LINUX}

pkg-linux_docker:
	$(DOCKER) run --rm -it -v $(shell pwd):/app \
		${RUST_DOCKER_IMAGE_NAME}:${LINUX_IMAGE_TAG} \
		bash -c "make pkg-linux"

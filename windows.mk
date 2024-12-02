TARGET_WINDOWS := x86_64-pc-windows-gnu
DOCKER_IMAGE_NAME := buildenv
WINDOWS_IMAGE_TAG := windows

pkg-windows:
	@$(CARGO) build --release --target=${TARGET_WINDOWS}

pkg-windows_docker:
	@$(DOCKER) run --rm -it -v $(shell pwd):/app ${DOCKER_IMAGE_NAME}:${WINDOWS_IMAGE_TAG} \
		bash -c "make pkg-windows"

windows-run:
	@$(DOCKER) run --rm -it -v $(shell pwd):/app ${DOCKER_IMAGE_NAME}:${WINDOWS_IMAGE_TAG} \
		bash

init_package_env:
	@python -m venv tools

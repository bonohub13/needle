# Commands
DOCKER := docker

# directories
DOCKER_DIR := docker
CARGO_REGISTRY := ${HOME}/.cargo/registry

# docker image/tag names
GLSL_DOCKER_IMAGE_NAME := glsl_buildenv
RUST_DOCKER_IMAGE_NAME := buildenv
BASE_IMAGE_TAG := base
LINUX_IMAGE_TAG := linux
WINDOWS_IMAGE_TAG := windows

build-images:
	@cd ${DOCKER_DIR} && make all

docker-exec:
	$(DOCKER) run --rm -it \
		-v ${PWD}:/app \
		-v ${CARGO_REGISTRY}:/usr/local/cargo/registry \
		${IMAGE_NAME}:${TAG} \
		bash -c "$(CMD)"

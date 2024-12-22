DOCKER := docker

SHADER_BUILD_IMAGE_TAG := glsl_buildenv
BUILD_IMAGE_NAME := buildenv
BASE_IMAGE_TAG := base
LINUX_IMAGE_TAG := linux
WINDOWS_IMAGE_TAG := windows

SHADER_IMAGE_FILE := shader.dockerfile
BASE_IMAGE_FILE := base.dockerfile
LINUX_IMAGE_FILE := linux.dockerfile
WINDOWS_IMAGE_FILE := windows.dockerfile

all: build-base build-linux build-windows build-shader

build-base:
	docker build . -t ${BUILD_IMAGE_NAME}:${BASE_IMAGE_TAG} \
		-f ${BASE_IMAGE_FILE} ${DOCKERFLAG}

build-linux:
	docker build . -t ${BUILD_IMAGE_NAME}:${LINUX_IMAGE_TAG} \
		-f ${LINUX_IMAGE_FILE}

build-windows:
	docker build . -t ${BUILD_IMAGE_NAME}:${WINDOWS_IMAGE_TAG} \
		-f ${WINDOWS_IMAGE_FILE}

build-shader:
	docker build . -t ${SHADER_BUILD_IMAGE_TAG}:${LINUX_IMAGE_TAG} \
		-f ${SHADER_IMAGE_FILE}

rebuild:
	DOCKERFLAG=--no-cache make build-base build-linux build-windows build-shader

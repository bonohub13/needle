DOCKER := docker

IMAGE_NAME := buildenv
BASE_IMAGE_TAG := base
LINUX_IMAGE_TAG := linux
WINDOWS_IMAGE_TAG := windows

BASE_IMAGE_FILE := base.dockerfile
LINUX_IMAGE_FILE := linux.dockerfile
WINDOWS_IMAGE_FILE := windows.dockerfile

all: build-base build-linux build-windows

build-base:
	docker build . -t ${IMAGE_NAME}:${BASE_IMAGE_TAG} -f ${BASE_IMAGE_FILE}

build-linux:
	docker build . -t ${IMAGE_NAME}:${LINUX_IMAGE_TAG} -f ${LINUX_IMAGE_FILE}

build-windows:
	docker build . -t ${IMAGE_NAME}:${WINDOWS_IMAGE_TAG} -f ${WINDOWS_IMAGE_FILE}

DOCKER_DIR := docker

build-images:
	@cd ${DOCKER_DIR} && make all

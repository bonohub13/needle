GLSLC := glslc
DOCKER := docker

SHADER_DOCKER_IMAGE_NAME := glsl_buildenv
LINUX_IMAGE_TAG := linux

SHADER_DIR := shaders
SPIRV_DIR := ${SHADER_DIR}/spv

VERTEX_SHADER_PATH ?= shader.vert
FRAGMENT_SHADER_PATH ?= shader.frag

shader-docker:
	@$(DOCKER) run --rm -it -v $(shell pwd):/app ${DOCKER_IMAGE_NAME}:${LINUX_IMAGE_TAG} \
		bash -c "make shader"

shader: prepare
	@$(GLSLC) -o ${SPIRV_DIR}/${VERTEX_SHADER_PATH}.spv ${SHADER_DIR}/${VERTEX_SHADER_PATH}
	@$(GLSLC) -o ${SPIRV_DIR}/${FRAGMENT_SHADER_PATH}.spv ${SHADER_DIR}/${FRAGMENT_SHADER_PATH}

prepare:
	@if [ ! -d ${SPIRV_DIR} ]; then mkdir -pv ${SPIRV_DIR}; fi

GLSLC := glslc
DOCKER := docker

SHADER_DIR := shaders
SPIRV_DIR := ${SHADER_DIR}/spv

VERTEX_SHADER_PATH ?= shader.vert
FRAGMENT_SHADER_PATH ?= shader.frag

shader-docker:
	@IMAGE_NAME=glsl_buildenv TAG=${LINUX_IMAGE_TAG} CMD="make shader" \
		make docker-exec

shader: prepare
	$(GLSLC) -o ${SPIRV_DIR}/${VERTEX_SHADER_PATH}.spv ${SHADER_DIR}/${VERTEX_SHADER_PATH}
	$(GLSLC) -o ${SPIRV_DIR}/${FRAGMENT_SHADER_PATH}.spv ${SHADER_DIR}/${FRAGMENT_SHADER_PATH}

prepare:
	if [ ! -d ${SPIRV_DIR} ]; then mkdir -pv ${SPIRV_DIR}; fi

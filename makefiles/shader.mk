GLSLC := glslc
NAGA := naga
DOCKER := docker

SHADER_DIR := shaders
SPIRV_DIR := ${SHADER_DIR}/spv

VERTEX_SHADER_PATH ?= shader.vert
FRAGMENT_SHADER_PATH ?= shader.frag

shader-docker:
	@IMAGE_NAME=glsl_buildenv TAG=${LINUX_IMAGE_TAG} CMD="make shader" \
		make docker-exec

shader: prepare
	$(NAGA) --shader-stage vert ${SHADER_DIR}/vs_main.wgsl ${SPIRV_DIR}/shader.vert.spv
	$(NAGA) --shader-stage frag ${SHADER_DIR}/fs_main.wgsl ${SPIRV_DIR}/shader.frag.spv

prepare:
	if [ ! -d ${SPIRV_DIR} ]; then mkdir -pv ${SPIRV_DIR}; fi

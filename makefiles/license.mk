# Commands
CARGO := cargo
DOCKER := docker
OSV_SCANNER := osv-scanner

# Path
SBOM_FILE ?= sbom.spdx.json

include makefiles/docker.mk

generate-sbom_docker:
	@IMAGE_NAME=${RUST_DOCKER_IMAGE_NAME} TAG=${BASE_IMAGE_TAG} CMD="make generate-sbom" \
		make docker-exec

generate-sbom:
	$(CARGO) sbom > ${SBOM_FILE}
	$(CARGO) cyclonedx -f json -a
	$(OSV_SCANNER) scan -r . || true

addlicense:
	$(DOCKER) run --rm -it -v ${PWD}:/src ghcr.io/google/addlicense:latest \
		-c "Kensuke Saito" \
		-l "MIT" \
		-s=only \
		$(shell find src -type f -name "*.rs")

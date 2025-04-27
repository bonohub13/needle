# Commands
CARGO := cargo
DOCKER := docker

# Variables
TARGET_LINUX := x86_64-unknown-linux-gnu

linux-update:
	$(CARGO) update --target=${TARGET_LINUX}

pkg-linux:
	$(CARGO) build --release --target=${TARGET_LINUX}

pkg-linux_docker:
	@IMAGE_NAME=${RUST_DOCKER_IMAGE_NAME} TAG=${LINUX_IMAGE_TAG} CMD="make pkg-linux" \
		make docker-exec

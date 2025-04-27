CARGO := cargo
DOCKER := docker

TARGET_WINDOWS := x86_64-pc-windows-gnu

windows-update:
	$(CARGO) update --target=${TARGET_WINDOWS}

pkg-windows:
	$(CARGO) build --release --target=${TARGET_WINDOWS}

pkg-windows_docker:
	@IMAGE_NAME=${RUST_DOCKER_IMAGE_NAME} TAG=${WINDOWS_IMAGE_TAG} CMD="make pkg-windows" \
		make docker-exec

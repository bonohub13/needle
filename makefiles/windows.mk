CARGO := cargo
DOCKER := docker

TARGET_WINDOWS := x86_64-pc-windows-msvc

windows-update:
	$(CARGO) update --target=${TARGET_WINDOWS}

windows-clippy:
	$(CARGO) clippy --target=${TARGET_WINDOWS}

pkg-windows:
	$(CARGO) xwin build --release --target=${TARGET_WINDOWS}

debug-windows:
	$(CARGO) xwin build --target=${TARGET_WINDOWS}

windows-clippy_docker:
	@IMAGE_NAME=${RUST_DOCKER_IMAGE_NAME} TAG=${WINDOWS_IMAGE_TAG} CMD="make windows-clippy" \
		make docker-exec

pkg-windows_docker:
	@IMAGE_NAME=${RUST_DOCKER_IMAGE_NAME} TAG=${WINDOWS_IMAGE_TAG} CMD="make pkg-windows" \
		make docker-exec

debug-windows_docker:
	@IMAGE_NAME=${RUST_DOCKER_IMAGE_NAME} TAG=${WINDOWS_IMAGE_TAG} CMD="make debug-windows" \
		make docker-exec

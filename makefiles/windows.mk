CARGO := cargo
DOCKER := docker

TARGET_WINDOWS := x86_64-pc-windows-gnu

windows-update:
	$(CARGO) update --target=${TARGET_WINDOWS}

windows-clippy:
	$(CARGO) clippy --target=${TARGET_WINDOWS}

pkg-windows:
	$(CARGO) build --release --target=${TARGET_WINDOWS}
	[ -f "./target/${TARGET_WINDOWS}/release/*.dll" ] \
		|| cp -rvf /usr/lib/gcc/x86_64-w64-mingw32/*-win32/*.dll ./target/${TARGET_WINDOWS}/release

windows-clippy_docker:
	@IMAGE_NAME=${RUST_DOCKER_IMAGE_NAME} TAG=${WINDOWS_IMAGE_TAG} CMD="make windows-clippy" \
		make docker-exec

pkg-windows_docker:
	@IMAGE_NAME=${RUST_DOCKER_IMAGE_NAME} TAG=${WINDOWS_IMAGE_TAG} CMD="make pkg-windows" \
		make docker-exec

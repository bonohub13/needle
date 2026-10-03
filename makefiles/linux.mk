# Commands
CARGO := cargo
DOCKER := docker

# Variables
TARGET_LINUX := x86_64-unknown-linux-gnu

linux-update:
	$(CARGO) update --target=${TARGET_LINUX}

linux-clippy:
	$(CARGO) clippy --target=${TARGET_LINUX} --release

pkg-linux:
	if command -v lld; then \
		RUSTFLAGS="-C link-arg=-fuse-ld=lld" $(CARGO) build --release --target=${TARGET_LINUX}; \
	else \
		@$(CARGO) build --release --target=${TARGET_LINUX}; \
	fi

debug-linux:
	if command -v mold; then \
		RUSTFLAGS="-C link-arg=-fuse-ld=mold" $(CARGO) build --target=${TARGET_LINUX}; \
	else \
		@$(CARGO) build --target=${TARGET_LINUX}; \
	fi

linux-clippy_docker:
	@IMAGE_NAME=${RUST_DOCKER_IMAGE_NAME} TAG=${LINUX_IMAGE_TAG} CMD="make linux-clippy" \
		make docker-exec

debug-linux_docker:
	@IMAGE_NAME=${RUST_DOCKER_IMAGE_NAME} TAG=${LINUX_IMAGE_TAG} CMD="make debug-linux" \
		make docker-exec

pkg-linux_docker:
	@IMAGE_NAME=${RUST_DOCKER_IMAGE_NAME} TAG=${LINUX_IMAGE_TAG} CMD="make pkg-linux" \
		make docker-exec

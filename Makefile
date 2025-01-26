CARGO := cargo
TAR := tar

NEEDLE_CORE := needle-core

SPIRV_DIR := shaders/spv
MAKEFILES_DIR := makefiles

include ${MAKEFILES_DIR}/linux.mk
include ${MAKEFILES_DIR}/windows.mk
include ${MAKEFILES_DIR}/shader.mk
include ${MAKEFILES_DIR}/docker.mk

all: build run

# Rust code
clean:
	@$(CARGO) clean
	@(cd $(NEEDLE_CORE) && $(CARGO) clean)
	@rm -rvf ${SPIRV_DIR}

pkg: clean pkg-linux_docker pkg-windows_docker
	@cp -v target/x86_64-unknown-linux-gnu/release/needle .
	@cp -v target/x86_64-pc-windows-gnu/release/needle.exe .
	@sha256sum needle \
		| tee needle.sha256
	@sha256sum needle.exe \
		| tee needle.exe.sha256

fmt:
	@$(CARGO) fmt

fetch:
	@$(CARGO) fetch --manifest-path=Cargo.toml
	@$(CARGO) fetch --manifest-path=${NEEDLE_CORE}/Cargo.toml

update:
	@$(CARGO) update --verbose
	@cd ${NEEDLE_CORE} && $(CARGO) update --verbose

build: fmt
	@$(CARGO) build --offline

release: fmt
	@$(CARGO) build --release --offline

run:
	@$(CARGO) run --release --offline

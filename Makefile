CARGO := cargo
TAR := tar

NEEDLE_CORE := needle-core

SPIRV_DIR := shaders/spv
MAKEFILES_DIR := makefiles
PKG := pkg

include ${MAKEFILES_DIR}/linux.mk
include ${MAKEFILES_DIR}/windows.mk
include ${MAKEFILES_DIR}/shader.mk
include ${MAKEFILES_DIR}/docker.mk
include ${MAKEFILES_DIR}/docs.mk

all: build run

# Rust code
clean:
	@$(CARGO) clean
	@(cd $(NEEDLE_CORE) && $(CARGO) clean)
	@rm -rvf ${SPIRV_DIR}
	@rm -rvf ${HOME}/.config/needle/shaders
	@rm -rvf ${PKG}

pkg: clean shader-docker pkg-linux_docker pkg-windows_docker
	@[ -d ${PKG} ] || mkdir -v ${PKG}
	@cp -v target/x86_64-unknown-linux-gnu/release/needle ${PKG}
	@cp -v target/x86_64-pc-windows-gnu/release/needle.exe ${PKG}
	@SRC_FILE=${PKG}/needle make generate_hash
	@SRC_FILE=${PKG}/needle.exe make generate_hash
	@SRC_FILE=shaders/spv/shader.vert.spv make generate_hash
	@SRC_FILE=shaders/spv/shader.frag.spv make generate_hash

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

release: fmt shader-docker
	@$(CARGO) build --release --offline

run:
	@$(CARGO) run --release --offline

generate_hash:
	cat ${SRC_FILE} \
		| sha512sum \
		| tee $(shell echo "${SRC_FILE}" | awk -F/ '{printf "${PKG}/%s\n", $$NF}').sha512

.PHONY: clean pkg fmt fetch update build release run

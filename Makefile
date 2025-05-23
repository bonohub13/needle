CARGO := cargo
TAR := tar

SPIRV_DIR := shaders/spv
MAKEFILES_DIR := makefiles
PKG := pkg

include ${MAKEFILES_DIR}/linux.mk
include ${MAKEFILES_DIR}/windows.mk
include ${MAKEFILES_DIR}/shader.mk
include ${MAKEFILES_DIR}/license.mk
include ${MAKEFILES_DIR}/docs.mk

all: build run

# Rust code
clean:
	@$(CARGO) clean
	@rm -rvf ${SPIRV_DIR}
	@rm -rvf ${HOME}/.config/needle/shaders
	@rm -rvf ${PKG}

pkg: clean addlicense shader-docker pkg-linux_docker pkg-windows_docker generate-sbom_docker
	@[ -d ${PKG} ] || mkdir -v ${PKG}
	@cp -v target/x86_64-unknown-linux-gnu/release/needle ${PKG}
	@cp -v target/x86_64-pc-windows-gnu/release/needle.exe ${PKG}
	@SRC_FILE=${PKG}/needle make generate_hash
	@SRC_FILE=${PKG}/needle.exe make generate_hash
	@SRC_FILE=shaders/spv/shader.vert.spv make generate_hash
	@SRC_FILE=shaders/spv/shader.frag.spv make generate_hash

install:
	@[ -d ${HOME}/.cargo/bin ] || ( \
		mkdir -pv "${HOME}/.cargo/bin" && \
		echo "Add \"$${HOME}/.cargo/bin\" to path" \
	)
	@$(CARGO) install --path .

fmt:
	@$(CARGO) fmt

fetch:
	@$(CARGO) fetch --manifest-path=Cargo.toml

update:
	@$(CARGO) update --verbose

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

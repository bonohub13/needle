CARGO := cargo
TAR := tar
ZIP := zip

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

pkg: clean update fetch
	@make addlicense
	@make shader-docker
	@make pkg-linux_docker
	@make pkg-windows_docker
	@make generate-sbom_docker
	@[ -d ${PKG} ] || mkdir -v ${PKG}
	@cp -v target/x86_64-unknown-linux-gnu/release/needle ${PKG}
	@cp -v target/x86_64-pc-windows-msvc/release/needle.exe ${PKG}
	@cp -v shaders/spv/shader.*.spv ${PKG}
	@SRC_FILE=${PKG}/needle make generate_hash
	@SRC_FILE=${PKG}/needle.exe make generate_hash
	@SRC_FILE=${PKG}/shader.vert.spv make generate_hash
	@SRC_FILE=${PKG}/shader.frag.spv make generate_hash

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
	@make fetch

clippy:
	@$(CARGO) clippy

clippy-docker:
	@make linux-clippy_docker
	@make windows-clippy_docker

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

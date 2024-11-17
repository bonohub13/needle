CARGO := cargo
NEEDLE_CORE := needle-core

all: build run

# Rust code
clean:
	$(CARGO) clean
	(cd $(NEEDLE_CORE) && $(CARGO) clean)

fmt:
	$(CARGO) fmt

build: fmt
	$(CARGO) build

release: fmt
	$(CARGO) build --release

run: clean
	$(CARGO) run --release

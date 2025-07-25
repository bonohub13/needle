FROM buildenv:base

RUN apt install -y \
    gcc-mingw-w64 \
    g++-mingw-w64
RUN rustup target add x86_64-pc-windows-gnu
RUN rustup component add clippy

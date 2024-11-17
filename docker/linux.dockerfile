FROM buildenv:base

RUN rustup target add x86_64-unknown-linux-gnu

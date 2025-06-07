FROM debian:12-slim

RUN apt update
RUN apt upgrade -y
RUN apt install -y \
    glslc \
    build-essential \
    curl
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH=/root/.cargo/bin:${PATH}
RUN rustup update
RUN cargo install naga-cli

WORKDIR /app

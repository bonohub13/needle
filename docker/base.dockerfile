FROM rust:latest

RUN apt update
RUN apt upgrade -y

RUN rustup update
RUN rustup component add rustfmt

WORKDIR /app

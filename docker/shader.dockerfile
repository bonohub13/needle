FROM debian:12-slim

RUN apt update
RUN apt upgrade -y
RUN apt install -y \
    glslc \
    build-essential

WORKDIR /app

FROM rust:latest

RUN apt update
RUN apt upgrade -y
RUN apt install -y golang-go

RUN rustup update
RUN rustup component add rustfmt

RUN cargo install cargo-sbom
RUN cargo install cargo-cyclonedx

RUN wget https://go.dev/dl/go1.24.2.linux-$(dpkg --print-architecture).tar.gz
RUN rm -rvf /usr/local/go
RUN tar -C /usr/local -xzvf go1.24.2.linux-$(dpkg --print-architecture).tar.gz
ENV PATH=/usr/local/go/bin:${PATH}
RUN go install github.com/google/osv-scanner/v2/cmd/osv-scanner@latest
ENV PATH=/root/go/bin:${PATH}

WORKDIR /app

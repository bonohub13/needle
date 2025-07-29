FROM buildenv:base

RUN apt install -y --install-recommends \
    llvm \
    clang-tools-19 \
    build-essential
RUN rustup component add \
    clippy \
    llvm-tools
RUN rustup target add x86_64-pc-windows-msvc
RUN cargo install --locked cargo-xwin
RUN ln -s clang-19 /usr/bin/clang
RUN ln -s clang /usr/bin/clang++
RUN ln -s lld-19 /usr/bin/ld.lld
RUN ln -s clang-19 /usr/bin/clang-cl
RUN ln -sf llvm-ar-19 /usr/bin/llvm-lib
RUN ln -s lld-link-19 /usr/bin/lld-link
RUN ln -sf llvm-rc-19 /usr/bin/llvm-rc
# Check if properly linked
RUN ln -s clang++ -v
RUN ln -s ld.lld -v
RUN ln -s llvm-lib -v
RUN ln -s clang-cl -v
RUN ln -s lld-link --version

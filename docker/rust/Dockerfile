ARG RUST_TAG=1.64.0
FROM rust:${RUST_TAG}

RUN rustup component add rustfmt

ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update
RUN apt-get install -y cmake protobuf-compiler

#!/bin/bash

set -e

BIN="$(cd "$(dirname "$0")" ; pwd)"
PROJECT="$(dirname "${BIN}")"

declare -a FLAGS_INHERIT
source "${BIN}/verbose.sh"

"${BIN}/create-local-settings.sh"

source "${PROJECT}/etc/settings-local.sh"

DOCKER_FLAGS=()
## DOCKER_FLAGS=(-e 'RUSTFLAGS=-Z macro-backtrace')
time docker run --rm -v "cargo-home:/var/cargo-home" -e "CARGO_HOME=/var/cargo-home" \
    "${DOCKER_FLAGS[@]}" \
    -v "${PROJECT}:${PROJECT}" -w "${PROJECT}" "${DOCKER_REPOSITORY}/rust" \
    cargo check

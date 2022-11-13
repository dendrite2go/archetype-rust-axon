#!/bin/bash

BIN="$(cd "$(dirname "$0")" ; pwd)"
PROJECT="$(dirname "${BIN}")"

"${BIN}/create-local-settings.sh"

RUST_TAG='1.64.0'

source "${PROJECT}/etc/settings-local/sh"

RUSTUP_VOLUME="rustup-${RUST_TAG}"

if [[ -z "$(docker volume inspect "${RUSTUP_VOLUME}" | yq.sh -y '{"name":.[0].Name}')" ]]
then
  docker run --rm -v "${RUSTUP_VOLUME}:/opt/rustup" "rust:${RUST_TAG}" cp -r /usr/local/rustup /opt/rustup
fi

docker run --rm -ti -v "${RUSTUP_VOLUME}:/usr/local/rustup" -v "${HOME}:${HOME}" -w "$(pwd)" -e "USER=${USER}" "rust:${RUST_TAG}" "$@"

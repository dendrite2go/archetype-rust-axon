#!/bin/bash

BIN="$(cd "$(dirname "$0")" ; pwd)"

## /usr/local/rustup
RUST_VERSION='1.44'

RUSTUP_VOLUME="rustup-${RUST_VERSION}"

if [[ -z "$(docker volume inspect "${RUSTUP_VOLUME}" | yq.sh -y '{"name":.[0].Name}')" ]]
then
  docker run --rm -v "${RUSTUP_VOLUME}:/opt/rustup" rust:1.44 cp -r /usr/local/rustup /opt/rustup
fi

docker run --rm -ti -v "${RUSTUP_VOLUME}:/usr/local/rustup" -v "${HOME}:${HOME}" -w "$(pwd)" -e "USER=${USER}" rust:1.44 "$@"

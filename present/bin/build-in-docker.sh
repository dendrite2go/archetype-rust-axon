#!/usr/bin/env bash

BIN="$(cd "$(dirname "$0")" ; pwd)"
MODULE="$(dirname "${BIN}")"
PROJECT="$(dirname "${MODULE}")"

source "${PROJECT}/bin/verbose.sh"
source "${PROJECT}/etc/settings-local.sh"

function run-with-protoc() {
  if type protoc >/dev/null 2>&1
  then
    (
      cd "${BIN}"
      "$@"
    )
  else
    docker run --rm -v "${PROJECT}:${PROJECT}" -w "${BIN}" "${DOCKER_REPOSITORY}/build-protoc" "$@"
  fi
}

run-with-protoc ./generate-proto-js-package.sh -v

docker run --rm -i -v "${MODULE}:${MODULE}" -w "${MODULE}" node:latest npm run build
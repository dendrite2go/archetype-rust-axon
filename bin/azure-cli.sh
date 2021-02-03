#!/bin/bash

set -e

BIN="$(cd "$(dirname "$0")" ; pwd)"
PROJECT="$(dirname "${BIN}")"

if [[ "$#" -gt 0 ]]
then
  COMMAND=(-c "$*")
else
  COMMAND=()
fi

docker run -ti \
    -v "${HOME}/.azure/.ssh:/root/.ssh" \
    -v "${PROJECT}:${PROJECT}" \
    -w "$(pwd)" \
    dendrite2go/azure-cli-k8s:2.18.0 \
    /bin/bash "${COMMAND[@]}"

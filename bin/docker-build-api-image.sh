#!/bin/bash

BIN="$(cd "$(dirname "$0")" ; pwd)"
PROJECT="$(dirname "${BIN}")"

source "${BIN}/verbose.sh"

DOCKER_CONTEXT="${PROJECT}/target/linux/docker"
mkdir -p "${DOCKER_CONTEXT}"
cp "${PROJECT}/target/linux/debug/dendrite_example" "${DOCKER_CONTEXT}/."

docker build -f "${PROJECT}/docker/api/Dockerfile" --tag dendrite2go/rustic-api:0.0.1-SNAPSHOT "${DOCKER_CONTEXT}"

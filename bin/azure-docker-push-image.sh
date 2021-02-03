#!/bin/bash

set -e

BIN="$(cd "$(dirname "$0")" ; pwd)"
PROJECT="$(dirname "${BIN}")"

source "${BIN}/verbose.sh"

"${BIN}/create-local-settings.sh"
source "${PROJECT}/etc/settings-local.sh"

TOKEN_JSON="${PROJECT}/data/local/docker-token.json"

LOGIN_SERVER="$(cat "${TOKEN_JSON}" | "${BIN}/yq.sh" -r '.loginServer')"
ACCESS_TOKEN="${PROJECT}/data/local/access-token"

: > "${ACCESS_TOKEN}"
chmod go= "${ACCESS_TOKEN}"

cat "${TOKEN_JSON}" | "${BIN}/yq.sh" -r '.accessToken' >> "${ACCESS_TOKEN}"

log "[${LOGIN_SERVER}] [$(echo "${ACCESS_TOKEN}" | cut -c-12)...]"

docker tag "dendrite2go/$1:${ENSEMBLE_IMAGE_VERSION}" "${LOGIN_SERVER}/$1:${ENSEMBLE_IMAGE_VERSION}"

cat "${ACCESS_TOKEN}" | docker login "${LOGIN_SERVER}" -u '00000000-0000-0000-0000-000000000000' --password-stdin
docker push "${LOGIN_SERVER}/$1:${ENSEMBLE_IMAGE_VERSION}"

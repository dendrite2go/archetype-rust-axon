#!/bin/bash

set -e

BIN="$(cd "$(dirname "$0")" ; pwd)"
PROJECT="$(dirname "${BIN}")"

TOKEN_JSON="${PROJECT}/data/local/docker-token.json"

: > "${TOKEN_JSON}"
chmod go= "${TOKEN_JSON}"

az acr login --name dendrite2go --expose-token | tee -a "${TOKEN_JSON}"

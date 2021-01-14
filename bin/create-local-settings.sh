#!/bin/bash

set -e

BIN="$(cd "$(dirname "$0")" ; pwd)"
PROJECT="$(dirname "${BIN}")"

declare -a FLAGS_INHERIT
. "${BIN}/verbose.sh"

info "PROJECT=[${PROJECT}]"

find "${PROJECT}" -name '*-sample.*' -print0 | xargs -0 -n 1 "${BIN}/create-local-setting.sh" "${FLAGS_INHERIT[@]}" "$@"

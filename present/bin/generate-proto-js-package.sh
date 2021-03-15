#!/bin/bash

BIN="$(cd "$(dirname "$0")" ; pwd)"
MODULE="$(dirname "${BIN}")"
PROJECT="$(dirname "${MODULE}")"

source "${PROJECT}/bin/verbose.sh"

cd "${PROJECT}/proto"

if [ \! -f 'proto_example.proto' ]
then
  error "Protocol buffer specification files for back-end API not found in: $(pwd)"
fi

log "Generating JS stubs from $(pwd)"

OUT_DIR="${MODULE}/src/grpc/backend"
mkdir -p "${OUT_DIR}"

protoc --js_out="import_style=commonjs:${OUT_DIR}" --grpc-web_out="import_style=commonjs+dts,mode=grpcwebtext:${OUT_DIR}" -I. *.proto

# Add /* eslint-disable */
cd "${MODULE}/src/grpc/backend"
sed -E -i \
  -e '1s:^/\* eslint-disable \*/$:/*@@@ eslint-disable @@@*/:' \
  -e "1i\\
/* eslint-disable */" \
  -e '/^\/\*@@@ eslint-disable @@@\*\//d' \
  *.js

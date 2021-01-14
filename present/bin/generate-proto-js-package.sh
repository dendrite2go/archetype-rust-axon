#!/bin/bash

BIN="$(cd "$(dirname "$0")" ; pwd)"
MODULE="$(dirname "${BIN}")"
PROJECT="$(dirname "${MODULE}")"

source "${PROJECT}/bin/verbose.sh"

cd "${PROJECT}/proto"

if [ \! -f 'grpc_example.proto' ]
then
  error "Protocol buffer specification files for Example not found in current directory"
fi

log "Generating JS stubs from $(pwd)"

OUT_DIR="${MODULE}/src/grpc/example"
mkdir -p "${OUT_DIR}"

protoc --js_out="import_style=commonjs:${OUT_DIR}" --grpc-web_out="import_style=commonjs+dts,mode=grpcwebtext:${OUT_DIR}" -I. *.proto

# Add /* eslint-disable */
cd "${MODULE}/src/grpc/example"
sed -E -i \
  -e '1s:^/\* eslint-disable \*/$:/*@@@ eslint-disable @@@*/:' \
  -e "1i\\
/* eslint-disable */" \
  -e '/^\/\*@@@ eslint-disable @@@\*\//d' \
  *.js

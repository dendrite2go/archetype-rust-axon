#!/bin/bash

BIN="$(cd "$(dirname "$0")" ; pwd)"
PROJECT="$(dirname "${BIN}")"
PROTO_DIR="${PROJECT}/proto"

source "${BIN}/verbose.sh"

HOST='host.docker.internal'
if [[ ".$1" = '.--host' ]]
then
  HOST="$2"
  shift 2
fi

VARIABLE='message'
VALUE='{}'
case "$1" in
--greet)
  PROTO="proto_example.proto"
  PORT='3000'
  URL='proto_example.GreeterService/Greet'
  shift
  ;;
--record)
  PROTO="proto_example.proto"
  PORT='3000'
  URL='proto_example.GreeterService/Record'
  VARIABLE=''
  shift
  ;;
--stop)
  PROTO="proto_example.proto"
  PORT='3000'
  URL='proto_example.GreeterService/Stop'
  VARIABLE=''
  shift
  ;;
--greetings)
  PROTO="proto_example.proto"
  PORT='3000'
  URL='proto_example.GreeterService/Greetings'
  VARIABLE=''
  shift
  ;;
--search)
  PROTO="proto_example.proto"
  PORT='3000'
  URL='proto_example.GreeterService/Search'
  VARIABLE='query'
  shift
  ;;
--direct)
  PROTO="proto_example.proto"
  PORT='8181'
  URL='proto_example.GreeterService/Greet'
  shift
  ;;
--authorize)
  PROTO="dendrite_config.proto"
  PORT='3000'
  URL='dendrite_config.ConfigurationService/Authorize'
  VARIABLE=''
  shift
  PROTO_DIR="${PROJECT}/target/proto"
  mkdir -p "${PROTO_DIR}"
  sed -e '/^package/s/ proto_/ /' "${PROJECT}/proto/proto_dendrite_config.proto" > "${PROTO_DIR}/dendrite_config.proto"
  ;;
*)
  exit 1
  ;;
esac

if [[ -n "$1" ]]
then
  VALUE="$1"
  shift
fi

if [[ -z "${VARIABLE}" ]]
then
  PAYLOAD="${VALUE}"
else
  PAYLOAD="$(echo "{'${VARIABLE}':'${VALUE}'}" | tr \'\" \"\')"
fi

docker run --rm -v "${PROJECT}:${PROJECT}" -w "${PROJECT}" -ti \
  fullstorydev/grpcurl -plaintext -import-path "${PROTO_DIR}" -proto "${PROTO}" \
    -d "${PAYLOAD}" "${HOST}:${PORT}" "${URL}"

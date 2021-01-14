#!/bin/bash

set -e

BIN="$(cd "$(dirname "$0")" ; pwd)"

. "${BIN}/verbose.sh"

SED_EXT=-r
case "$(uname)" in
Darwin*)
        SED_EXT=-E
esac
export SED_EXT

SAMPLE="$1"
LOCAL="$(echo "${SAMPLE}" | sed "${SED_EXT}" -e 's/-sample([+][^.]*)?/-local/')"

if [ -e "${LOCAL}" ]
then
    log "Skip: ${LOCAL}"
else
    info "Create: [${LOCAL}]"
    cp "${SAMPLE}" "${LOCAL}"
fi

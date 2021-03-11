#!/bin/bash

BIN="$(cd "$(dirname "$0")" ; pwd)"
PROJECT="$(dirname "${BIN}")"
SRC="${PROJECT}/src"
MODULE="${SRC}/example_event/trusted_generated.rs"

source "${BIN}/verbose.sh"
source "${PROJECT}/etc/settings-local.sh"

mkdir -p "$(dirname "${MODULE}")"

echo '//! Generated module. Do not edit.

use anyhow::Result;
use dendrite_auth::dendrite_config::PublicKey;
use dendrite_auth;

pub fn init() -> Result<()> {' > "${MODULE}"
(
  cd "${PROJECT}" || exit 1
  N=0
  for F in "${ROOT_PRIVATE_KEY}.pub" "${ADDITIONAL_TRUSTED_KEYS}"
  do
    if [[ -z "${F}" ]]
    then
      continue
    fi
    log ">>> Trusted key: [${F}]"
    KEY="$(cut -d ' ' -f2 "${F}")"
    NAME="$(cut -d ' ' -f3 "${F}")"
    if [[ -z "${KEY}" ]]
    then
      continue
    fi
    if [[ -z "${NAME}" ]]
    then
      N=$((${N} + 1))
      NAME="key-${N}"
    fi
    echo "    let public_key = PublicKey {"
    echo "        name: \"${NAME}\".to_string(),"
    echo "        public_key: \"${KEY}\".to_string(),"
    echo "    };"
    echo "    dendrite_auth::unchecked_set_public_key(public_key.clone())?;"
    echo "    dendrite_auth::unchecked_set_key_manager(public_key.clone())?;"
  done >> "${MODULE}"
)
echo '    Ok(())
}' >> "${MODULE}"

"${SILENT}" || sed -e 's/^/+/' "${MODULE}"

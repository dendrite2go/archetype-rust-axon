#!/bin/bash

BIN="$(cd "$(dirname "$0")" ; pwd)"
PROJECT="$(dirname "${BIN}")"

source "${BIN}/verbose.sh"
source "${BIN}/lib-sed.sh"

"${BIN}/create-local-settings.sh"

source "${PROJECT}/etc/settings-local.sh"

if [[ -z "${SIGN_PRIVATE_KEY}" ]]
then
  SIGN_PRIVATE_KEY="${ROOT_PRIVATE_KEY}"
fi

function is-prefix() {
  local FIRST="$1"
  local SECOND="$2"
  local DOTS="$(echo -n  "${FIRST}" | tr -c '' '.')"
  local CHUNK="$(echo "${SECOND}" | sed "${SED_EXT}" "s/^(${DOTS}).*$/\\1/")"
  test ".${CHUNK}" = ".${FIRST}"
}

if [ \! -x "/opt/configmanager" ]
then
  info "Run in docker container"
  VOLUMES=("${PROJECT}")
  for F in "${ROOT_PRIVATE_KEY}" "${SIGN_PRIVATE_KEY}"
  do
    D="$(dirname "$F")"
    PREFIX='false'
    for P in "${VOLUMES[@]}"
    do
      if is-prefix "${P}" "${D}"
      then
        PREFIX='true'
      fi
    done
    if "${PREFIX}"
    then
      :
    else
      VOLUMES[${#VOLUMES[@]}]="${D}"
    fi
  done
  VOLUME_ARGS=()
  for V in "${VOLUMES[@]}"
  do
    VOLUME_ARGS[${#VOLUME_ARGS[@]}]='-v'
    VOLUME_ARGS[${#VOLUME_ARGS[@]}]="${V}:${V}"
  done
  docker run --rm -i --init \
      "${VOLUME_ARGS[@]}" -w "$(pwd)" \
      --entrypoint /bin/bash \
      dendrite2go/configmanager \
      -c "${PROJECT}/bin/inject-keys.sh ${FLAGS_INHERIT[@]} '${AUTHORITY:-host.docker.internal:3000}'"
  exit $?
fi

AUTHORITY="$1" ; shift

function readyness-test() {
  echo ">>> Properties
actuator.test=test
>>> End" \
  | "/opt/configmanager" "${AUTHORITY:-host.docker.internal:3000}" 2>&1 \
  | sed \
      -e '/[Ee]rror/!d' \
      -e 's/Error.*desc = //'
}

while true
do
  RESPONSE="$(readyness-test | head -1)"
  log "RESPONSE: [${RESPONSE}]"
  if [[ -z "${RESPONSE}" ]]
  then
    break
  fi
  sleep 3
done

log "READY!"

(
  cd "${PROJECT}" || exit 1
  log "DIR=[$(pwd)]"

  ROOT_PUBLIC_KEY="$(cat "${ROOT_PRIVATE_KEY}.pub")"
  ROOT_KEY_NAME="$(cat "${ROOT_PRIVATE_KEY}.pub" | cut -d ' ' -f 3)"
  SIGN_KEY_NAME="$(cat "${SIGN_PRIVATE_KEY}.pub" | cut -d ' ' -f 3)"

  (
    echo ">>> Manager: ${ROOT_KEY_NAME}"
    cat "${ROOT_PRIVATE_KEY}"
    if [[ ".${SIGN_PRIVATE_KEY}" != ".${ROOT_PRIVATE_KEY}" ]]
    then
      echo '>>> Trusted:'
      cat "${SIGN_PRIVATE_KEY}.pub"
    fi

    echo ">>> Identity Provider: ${SIGN_KEY_NAME}"
    cat "${SIGN_PRIVATE_KEY}"

    CONFIG_DATA="${PROJECT}/target/config.data"
    if [ -f "${CONFIG_DATA}" ]
    then
      cat "${CONFIG_DATA}"
    fi

    echo '>>> End'
  ) | "/opt/configmanager" "${AUTHORITY:-host.docker.internal:3000}"
)
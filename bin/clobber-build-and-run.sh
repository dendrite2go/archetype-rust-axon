#!/usr/bin/env bash

set -e

BIN="$(cd "$(dirname "$0")" ; pwd)"
PROJECT="$(dirname "${BIN}")"

declare -a FLAGS_INHERIT
source "${BIN}/verbose.sh"

if [[ ".$1" = '.--help' ]]
then
    echo "Usage: $(basename "$0") [ -v [ -v ] ] [ --tee <file> ] [ --skip-build ] [ --build-uses-siblings ] [ --back-end-only ] [ -no-clobber ] [ --dev ]" >&2
    echo "       $(basename "$0") --help" >&2
    exit 0
fi

if [[ ".$1" = '.--tee' ]]
then
    exec > >(tee "$2") 2>&1
    shift 2
elif [[ ".$1" = '.--tee-time' ]]
then
    TIMESTAMP="$(date '+%Y%m%dT%H%M')"
    LOG_FILE="${PROJECT}/data/local/build-and-run-${TIMESTAMP}.log"
    info "LOG_FILE=[${LOG_FILE}]"
    exec > >(tee "${LOG_FILE}") 2>&1
    shift
fi

DO_BUILD='true'
if [[ ".$1" = '.--skip-build' ]]
then
  DO_BUILD='false'
  shift
fi

BUILD_VOLUME="${PROJECT}"
if [[ ".$1" = '.--build-uses-siblings' ]]
then
  BUILD_VOLUME="$(dirname "${PROJECT}")"
  shift
fi

DO_BUILD_BACK_END='true'
DO_BUILD_PRESENT='true'
DO_BUILD_SWAGGER_IMAGE='false'
if [[ ".$1" = '.--back-end-only' ]]
then
  DO_BUILD_PRESENT='false'
  DO_BUILD_SWAGGER_IMAGE='false'
  shift
fi

DO_CLOBBER='true'
if [[ ".$1" = '.--no-clobber' ]]
then
  DO_CLOBBER='false'
  shift
fi

: ${AXON_SERVER_PORT=8024}
: ${API_SERVER_PORT=8181}
: ${ENSEMBLE_NAME=rustic}
"${BIN}/create-local-settings.sh"

source "${PROJECT}/etc/settings-local.sh"

function waitForServerReady() {
    local URL="$1"
    local N="$2"
    if [[ -z "${N}" ]]
    then
        N=120
    fi
    while [[ "${N}" -gt 0 ]]
    do
        N=$[$N - 1]
        sleep 1
        if curl -sS "${URL}" >/dev/null 2>&1
        then
            break
        fi
    done
}

function countRunningContainers() {
    local HASH
    for HASH in $(docker-compose -p "${ENSEMBLE_NAME}" ps -q 2>/dev/null)
    do
        docker inspect -f '{{.State.Status}}' "${HASH}"
    done | grep -c running
}

function waitForDockerComposeReady() {
    (
        cd "${COMPOSE}"
        while [[ "$(countRunningContainers)" -gt 0 ]]
        do
            sleep 0.5
        done
    )
}

(
    cd "${PROJECT}"

#    src/bin/generate-root-key-pair.sh
#    src/bin/generate-module-for-trusted-keys.sh

    if "${DO_BUILD}"
    then
        info "Build rust docker image"
        docker build --build-arg RUST_TAG="${RUST_TAG}" -t "${DOCKER_REPOSITORY}/rust" docker/rust

        if "${DO_BUILD_BACK_END}"
        then
            # Generate module "trusted_generated".
            "${BIN}/generate-module-for-trusted-keys.sh" -v

            # Build server executables from Rust sources
            info "Build executables for the back-end"
            DOCKER_FLAGS=()
            ## DOCKER_FLAGS=(-e 'RUSTFLAGS=-Z macro-backtrace')
            time docker run --rm -v "cargo-home:/var/cargo-home" -e "CARGO_HOME=/var/cargo-home" \
                "${DOCKER_FLAGS[@]}" \
                -v "${BUILD_VOLUME}:${BUILD_VOLUME}" -w "${PROJECT}" "${DOCKER_REPOSITORY}/rust" \
                cargo build --target-dir 'target/linux'
        fi

        if "${DO_BUILD_PRESENT}"
        then
            info "Run npm install"
            "${PROJECT}/present/bin/npm-install-in-docker.sh"
            info "Run npm run build"
            "${PROJECT}/present/bin/build-in-docker.sh"
            info "Build docker images for presentation layer"
            docker build -t "${DOCKER_REPOSITORY}/${ENSEMBLE_NAME}-present:${ENSEMBLE_IMAGE_VERSION}" present
            EMPTY="${PROJECT}/target/empty-build-context"
            mkdir -p "${EMPTY}"
            docker build -t "${DOCKER_REPOSITORY}/${ENSEMBLE_NAME}-present:${ENSEMBLE_IMAGE_VERSION}-dev" -f present/Dockerfile-development "${EMPTY}"
        fi

        info "Build docker image for proxy"
        docker build -t "${DOCKER_REPOSITORY}/${ENSEMBLE_NAME}-proxy:${ENSEMBLE_IMAGE_VERSION}" docker/proxy

        if "${DO_BUILD_SWAGGER_IMAGE}"
        then
            info "Build docker image for Swagger UI"
            docker build -t "${DOCKER_REPOSITORY}/grpc-swagger" "docker/swagger"
        fi
    fi

    (
        info "Remove pre-existing docker containers"
        PRE_EXISTING="$(docker ps --filter "label=com.docker.compose.project=${ENSEMBLE_NAME}" -a --format '{{.ID}}')"
        if [[ -n "${PRE_EXISTING}" ]]
        then
            docker rm -f ${PRE_EXISTING}
        fi
    )

    if "${DO_CLOBBER}"
    then
      info "Remove pre-existing data volumes"
      docker volume rm -f "${ENSEMBLE_NAME}_axon-data"
      docker volume rm -f "${ENSEMBLE_NAME}_axon-eventdata"
      docker volume rm -f "${ENSEMBLE_NAME}_elastic-search-data"
    fi
)

info "Prepare configuration data"
"${BIN}/config-prepare.sh"

info "Start containers"
exec "${PROJECT}/docker/docker-compose-up.sh" "${FLAGS_INHERIT[@]}" "$@"
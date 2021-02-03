#!/bin/bash

set -e

BIN="$(cd "$(dirname "$0")" ; pwd)"
PROJECT="$(dirname "${BIN}")"

declare -a FLAGS_INHERIT
source "${BIN}/verbose.sh"
source "${BIN}/lib-sed.sh"
source "${BIN}/lib-template.sh"

RESOURCE_GROUP='rustic-example'
CLUSTER="${RESOURCE_GROUP}-cluster"

"${BIN}/create-local-settings.sh"
source "${PROJECT}/etc/settings-local.sh"

az group create --name "${RESOURCE_GROUP}" --location westeurope
az aks create --resource-group "${RESOURCE_GROUP}" --name "${CLUSTER}" \
    --node-count 1 --enable-addons monitoring \
    --ssh-key-value ~/.ssh/id_rsa.pub \
    --attach-acr "${DOCKER_REPOSITORY}"

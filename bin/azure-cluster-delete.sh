#!/bin/bash

set -e

BIN="$(cd "$(dirname "$0")" ; pwd)"
PROJECT="$(dirname "${BIN}")"

declare -a FLAGS_INHERIT
source "${BIN}/verbose.sh"

RESOURCE_GROUP='rustic-example'
CLUSTER="${RESOURCE_GROUP}-cluster"

"${BIN}/create-local-settings.sh"
source "${PROJECT}/etc/settings-local.sh"

az aks delete --name "${CLUSTER}" --resource-group "${RESOURCE_GROUP}" --yes

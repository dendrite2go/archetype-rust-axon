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

az aks get-credentials --resource-group "${RESOURCE_GROUP}" --name "${CLUSTER}" --overwrite-existing

KUBECTL_LOGS_FLAGS=()
if [[ ".$1" = '.-f' ]]
then
  KUBECTL_LOGS_FLAGS=(-f)
  shift
fi

kubectl logs "${KUBECTL_LOGS_FLAGS[@]}" $(kubectl get pods -l "app=$1" -o 'jsonpath={.items[*].metadata.name}')

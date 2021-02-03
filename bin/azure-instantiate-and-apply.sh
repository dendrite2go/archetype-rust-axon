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

az aks get-credentials --resource-group "${RESOURCE_GROUP}" --name "${CLUSTER}" --overwrite-existing
kubectl get nodes

instantiate "${PROJECT}/docker/azure/rustic-example" '.yaml'
kubectl apply -f "${PROJECT}/docker/azure/rustic-example.yaml"

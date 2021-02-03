#!/bin/bash

set -e

BIN="$(cd "$(dirname "$0")" ; pwd)"
PROJECT="$(dirname "${BIN}")"

declare -a FLAGS_INHERIT
source "${BIN}/verbose.sh"
source "${BIN}/lib-sed.sh"
source "${BIN}/lib-template.sh"

"${BIN}/create-local-settings.sh"
source "${PROJECT}/etc/settings-local.sh"

DO_CLOBBER='true'
if [[ ".$1" = '.--no-clobber' ]]
then
  DO_CLOBBER='false'
  shift
fi

DO_RUN='true'
if [[ ".$1" = '.--no-run' ]]
then
  DO_RUN='false'
  shift
fi

"${BIN}/clobber-build-and-run.sh" "${FLAGS_INHERIT[@]}" "$@" --no-clobber --no-run

# Build docker image for Azure
(
  cd "${PROJECT}/docker/azure"
  docker build --tag dendrite2go/azure-cli-k8s:2.18.0 .
)

RESOURCE_GROUP='rustic-example'
CLUSTER="${RESOURCE_GROUP}-cluster"

CLOBBER_COMMAND=''
if "${DO_CLOBBER}"
then
  CLOBBER_COMMAND="
az group delete --name '${RESOURCE_GROUP}' --yes --no-wait
"
fi

instantiate "${PROJECT}/docker/azure/rustic-example" '.yaml'

if "${DO_RUN}"
then
  "${BIN}/azure-cli.sh" "
az login
set -x
${CLOBBER_COMMAND}
az group create --name '${RESOURCE_GROUP}' --location westeurope
az aks create --resource-group '${RESOURCE_GROUP}' --name '${CLUSTER}' --node-count 1 --enable-addons monitoring --ssh-key-value ~/.ssh/id_rsa.pub
az aks get-credentials --resource-group '${RESOURCE_GROUP}' --name '${CLUSTER}'
kubectl get nodes
kubectl apply -f '${PROJECT}/docker/azure/rustic-example.yaml'
"
fi

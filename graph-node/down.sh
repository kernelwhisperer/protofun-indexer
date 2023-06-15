#!/usr/bin/env bash

set -e

ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
pushd "$ROOT" &> /dev/null

docker-compose down
docker volume rm graph-node_ipfsdata
docker volume rm graph-node_ipfsdata_export
docker volume rm graph-node_pgdata

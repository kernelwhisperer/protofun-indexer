#!/usr/bin/env bash

set -e

ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
pushd "$ROOT" &> /dev/null

exec docker compose up

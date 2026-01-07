#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

rustup default 1.88.0

rustup target add wasm32v1-none --toolchain 1.88.0

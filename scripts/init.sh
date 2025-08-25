#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

rustup default 1.84.1

rustup target add wasm32v1-none --toolchain 1.84.1

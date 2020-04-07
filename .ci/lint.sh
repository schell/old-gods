#!/bin/bash

ROOT="$(git rev-parse --show-toplevel)"
source $ROOT/.ci/common.sh

section "Test"

rustup run stable cargo test --verbose

section "Lint"
rustup run nightly cargo fmt -- --check

section "Build WASM"
wasm-pack build --debug --target web examples/loading-maps

section "done :tada:"

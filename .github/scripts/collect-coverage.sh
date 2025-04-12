#!/usr/bin/env bash

set -eux -o pipefail

export CARGO_INCREMENTAL='0'
export RUSTFLAGS='-Cinstrument-coverage'

cargo build --verbose --workspace --tests

export LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw'

cargo test --verbose --workspace

mkdir -p ./target/debug/coverage/
grcov . \
    --source-dir . \
    --binary-path ./target/debug/ \
    --output-types lcov,html \
    --llvm \
    --branch \
    --keep-only 'crates/**' \
    --ignore-not-existing \
    --excl-line 'grcov-excl-line|#\[derive\(|ensure!\(|assert!\(|/!|///' \
    --excl-start 'grcov-excl-start' \
    --excl-stop 'grcov-excl-stop' \
    --output-path ./target/debug/coverage/

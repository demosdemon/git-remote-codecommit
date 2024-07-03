#!/usr/bin/env bash

set -eux -o pipefail

export CARGO_INCREMENTAL='0'
export RUSTFLAGS='-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort'
export LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw'

cargo test --verbose --workspace

mkdir -p ./target/debug/coverage/
grcov . \
    -s . \
    --binary-path ./target/debug/ \
    -t lcov,html \
    --llvm \
    --branch \
    --ignore-not-existing \
    --excl-line 'grcov-excl-line|#\[derive\(|ensure!\(|assert!\(|/!|///' \
    --excl-start 'grcov-excl-start' \
    --excl-stop 'grcov-excl-stop' \
    -o ./target/debug/coverage/

#!/bin/sh
# Runs every verification step of docs/08-verificacao.md §3 in order and stops at the first failure.
set -eu
cd "$(dirname "$0")/.."
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
scripts/verba.sh
printf 'verificatio: tudo limpo\n'

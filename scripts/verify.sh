#!/bin/sh
# Runs the verification steps of docs/08-verificacao.md §3 that exist at this checkpoint, in order,
# and stops at the first failure.
set -eu
cd "$(dirname "$0")/.."
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
scripts/verba.sh
python_command=${SISYPHIUM_PYTHON:-python3}
"$python_command" scripts/calibrate-costs.py --self-test
"$python_command" scripts/consilium-reference.py --baseline
"$python_command" scripts/consilium-reference.py --check crates/sisyphium/tests/fixtures/consilium.tsv
cargo build -p sisyphium-tabula
"$python_command" scripts/verify-images.py --binary target/debug/sisyphium-tabula
"$python_command" scripts/verify-ffi.py
"$python_command" scripts/verify-hosts.py --platform current
"$python_command" scripts/verify-textus.py
printf 'verificatio: tudo limpo\n'

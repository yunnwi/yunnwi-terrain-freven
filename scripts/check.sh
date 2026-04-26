#!/usr/bin/env bash
set -euo pipefail

cargo fmt --check
cargo check
cargo build --release --target wasm32-unknown-unknown

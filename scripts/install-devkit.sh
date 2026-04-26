#!/usr/bin/env bash
set -euo pipefail

DEVKIT="${DEVKIT:-/path/to/freven-devkit-v0.1.0-rc7-aarch64-apple-darwin}"
INSTANCE="${INSTANCE:-$DEVKIT/instances/yunnwi_terrain_test}"

cargo build --release --target wasm32-unknown-unknown

mkdir -p "$INSTANCE/mods/yunnwi.terrain"
cp examples/mod.toml "$INSTANCE/mods/yunnwi.terrain/mod.toml"
cp target/wasm32-unknown-unknown/release/yunnwi_terrain_mod.wasm \
  "$INSTANCE/mods/yunnwi.terrain/yunnwi_terrain_mod.wasm"

mkdir -p "$INSTANCE/experiences/yunnwi.terrain.test"
cp examples/experience.stack.toml \
  "$INSTANCE/experiences/yunnwi.terrain.test/experience.stack.toml"

echo "Installed into: $INSTANCE"
echo "Run:"
echo "  cd $DEVKIT"
echo "  ./freven_boot play --instance instances/yunnwi_terrain_test --experience yunnwi.terrain.test -- --username dev --devtools"

# Yunnwi Terrain Mod for Freven rc7

Example runtime-loaded Wasm worldgen mod for Freven DevKit `v0.1.0-rc7`.

It demonstrates:

- runtime-loaded Wasm mod packaging with `mod.toml`
- `experience.stack.toml` layering over `freven.vanilla`
- custom registered blocks
- custom block colors via `BlockDescriptor`
- vanilla block lookup via `WorldGenInit::block_id_by_key`
- terrain generation through `WorldGenOutput.writes`
- initial spawn selection through `WorldGenOutput.bootstrap.initial_world_spawn_hint`

## Features

- mixed terrain: plains, forests, hills, mountains, high mountains
- cave generation
- trees
- simple houses
- custom blocks:
  - `yunnwi.terrain:cobblestone`
  - `yunnwi.terrain:log`
  - `yunnwi.terrain:leaves`

## Requirements

- Freven DevKit `v0.1.0-rc7`
- Rust with `wasm32-unknown-unknown` target

```bash
rustup target add wasm32-unknown-unknown
```

## Build

```bash
cargo build --release --target wasm32-unknown-unknown
```

The Wasm artifact will be:

```text
target/wasm32-unknown-unknown/release/yunnwi_terrain_mod.wasm
```

## Install into a Freven DevKit instance

Assuming DevKit is extracted at:

```text
/path/to/freven-devkit-v0.1.0-rc7-aarch64-apple-darwin
```

and a client instance exists at:

```text
instances/yunnwi_terrain_test
```

copy the mod:

```bash
DEVKIT=/path/to/freven-devkit-v0.1.0-rc7-aarch64-apple-darwin
INSTANCE=$DEVKIT/instances/yunnwi_terrain_test

mkdir -p "$INSTANCE/mods/yunnwi.terrain"
cp examples/mod.toml "$INSTANCE/mods/yunnwi.terrain/mod.toml"
cp target/wasm32-unknown-unknown/release/yunnwi_terrain_mod.wasm \
  "$INSTANCE/mods/yunnwi.terrain/yunnwi_terrain_mod.wasm"

mkdir -p "$INSTANCE/experiences/yunnwi.terrain.test"
cp examples/experience.stack.toml \
  "$INSTANCE/experiences/yunnwi.terrain.test/experience.stack.toml"
```

Run:

```bash
cd "$DEVKIT"
rm -rf instances/yunnwi_terrain_test/worlds/world_0
./freven_boot play --instance instances/yunnwi_terrain_test --experience yunnwi.terrain.test -- --username dev --devtools
```

## Notes

This mod intentionally uses registered runtime block IDs from `WorldGenInit` instead of hardcoded IDs.

For example, vanilla blocks are resolved with:

```rust
ctx.init().block_id_by_key("freven.vanilla:stone")
```

and custom blocks are resolved with:

```rust
ctx.init().block_id_by_key("yunnwi.terrain:leaves")
```

This is important because runtime IDs are registry-owned and should not be assumed by mods.

## Worldgen output

The generator stores a local 32×96×32 column buffer and then emits terrain writes
as vertical runs:

- single blocks use `WorldTerrainWrite::SetBlock`
- longer runs use `WorldTerrainWrite::FillBox`

`FillBox` uses half-open bounds: `[min, max)`.

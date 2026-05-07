# Yunnwi Terrain Mod for Freven rc8

MIT licensed example runtime-loaded Wasm terrain worldgen mod for Freven.

Example runtime-loaded Wasm worldgen mod for Freven DevKit `v0.1.0-rc8`.

It demonstrates:

- runtime-loaded Wasm mod packaging with `mod.toml`
- `experience.stack.toml` layering over `freven.vanilla`
- custom registered blocks
- custom block colors via `BlockDescriptor`
- vanilla block lookup via `WorldGenInit::block_id_by_key`
- custom terrain generation through `WorldGenOutput.writes`
- rc8 world vertical contract setup through `world_preset.toml`
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

- Freven DevKit `v0.1.0-rc8`
- Freven SDK `v0.1.2-rc5`
- Rust with `wasm32-unknown-unknown` target

Run:

    rustup target add wasm32-unknown-unknown

## Build

Run:

    cargo build --release --target wasm32-unknown-unknown

The Wasm artifact will be:

    target/wasm32-unknown-unknown/release/yunnwi_terrain_mod.wasm

## Install into a Freven DevKit instance

Assuming DevKit is extracted at:

    /path/to/freven-devkit-v0.1.0-rc8-aarch64-apple-darwin

and a client instance exists at:

    instances/yunnwi_terrain_test

copy the mod and experience files:

    DEVKIT=/path/to/freven-devkit-v0.1.0-rc8-aarch64-apple-darwin
    INSTANCE=$DEVKIT/instances/yunnwi_terrain_test

    mkdir -p "$INSTANCE/mods/yunnwi.terrain"
    cp examples/mod.toml "$INSTANCE/mods/yunnwi.terrain/mod.toml"
    cp target/wasm32-unknown-unknown/release/yunnwi_terrain_mod.wasm \
      "$INSTANCE/mods/yunnwi.terrain/yunnwi_terrain_mod.wasm"

    mkdir -p "$INSTANCE/experiences/yunnwi.terrain.test"
    cp examples/experience.stack.toml \
      "$INSTANCE/experiences/yunnwi.terrain.test/experience.stack.toml"
    cp examples/world_preset.toml \
      "$INSTANCE/experiences/yunnwi.terrain.test/world_preset.toml"

Run:

    cd "$DEVKIT"
    rm -rf instances/yunnwi_terrain_test/worlds/world_0
    rm -rf instances/yunnwi_terrain_test/worlds/yunnwi_terrain_world
    rm -f instances/yunnwi_terrain_test/world_bootstrap.toml

    ./freven_boot play --instance instances/yunnwi_terrain_test --experience yunnwi.terrain.test -- --username dev --devtools

## rc8 world preset

This mod generates a local `32×96×32` column buffer. In Freven rc8, worlds have an explicit vertical contract. The included `examples/world_preset.toml` sets:

    [world_preset.dimensions.vertical_contract]
    min_section_y = 0
    section_count = 3
    vertical_streaming_enabled = true

That allows worldgen writes in sections `0..2`, equivalent to world heights `y = 0..95`.

If you change the world preset for an existing test instance, delete `world_bootstrap.toml` and recreate the world so Freven rebuilds the persisted bootstrap from the updated preset.

## Notes

This mod intentionally uses registered runtime block IDs from `WorldGenInit` instead of hardcoded IDs.

For example, vanilla blocks are resolved with:

    ctx.init().block_id_by_key("freven.vanilla:stone")

and custom blocks are resolved with:

    ctx.init().block_id_by_key("yunnwi.terrain:leaves")

This is important because runtime IDs are registry-owned and should not be assumed by mods.

## Worldgen output

The generator stores a local `32×96×32` column buffer and then emits terrain writes as vertical runs:

- single blocks use `WorldTerrainWrite::SetBlock`
- longer runs use `WorldTerrainWrite::FillBox`

`FillBox` uses half-open bounds: `[min, max)`.

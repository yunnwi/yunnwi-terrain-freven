//! Block registration and stable string keys used by the mod.
//!
//! Runtime block IDs are assigned by Freven at load time. The worldgen code
//! resolves IDs from these keys through `WorldGenInit::block_id_by_key` instead
//! of assuming that numeric IDs are stable.

use freven_world_guest_sdk::{BlockDescriptor, GuestModule, RenderLayer};

pub const MOD_ID: &str = "yunnwi.terrain";

pub const COBBLESTONE_KEY: &str = "yunnwi.terrain:cobblestone";
pub const LOG_KEY: &str = "yunnwi.terrain:log";
pub const LEAVES_KEY: &str = "yunnwi.terrain:leaves";

pub const TERRAIN_WORLDGEN_KEY: &str = "yunnwi.terrain:terrain";

pub const VANILLA_STONE_KEY: &str = "freven.vanilla:stone";
pub const VANILLA_DIRT_KEY: &str = "freven.vanilla:dirt";
pub const VANILLA_GRASS_KEY: &str = "freven.vanilla:grass";

/// Registers the custom blocks exposed by this mod.
///
/// `material_id` currently acts like a renderer material slot. Keeping custom
/// blocks on unique material IDs lets them use independent debug tint colors.
pub fn register_blocks(module: GuestModule) -> GuestModule {
    module
        .register_block(
            COBBLESTONE_KEY,
            BlockDescriptor::new(true, true, RenderLayer::Opaque, 0x5F6368FF, 4),
        )
        .register_block(
            LOG_KEY,
            BlockDescriptor::new(true, true, RenderLayer::Opaque, 0x6B3F1DFF, 5),
        )
        .register_block(
            LEAVES_KEY,
            BlockDescriptor::new(true, true, RenderLayer::Opaque, 0x7BC96FFF, 6),
        )
}

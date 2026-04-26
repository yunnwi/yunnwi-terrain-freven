use freven_world_guest_sdk::{BlockDescriptor, GuestModule, RenderLayer};

pub const MOD_ID: &str = "yunnwi.terrain";

pub const COBBLESTONE_KEY: &str = "yunnwi.terrain:cobblestone";
pub const LOG_KEY: &str = "yunnwi.terrain:log";
pub const LEAVES_KEY: &str = "yunnwi.terrain:leaves";

pub const TERRAIN_WORLDGEN_KEY: &str = "yunnwi.terrain:terrain";

pub const VANILLA_STONE_KEY: &str = "freven.vanilla:stone";
pub const VANILLA_DIRT_KEY: &str = "freven.vanilla:dirt";
pub const VANILLA_GRASS_KEY: &str = "freven.vanilla:grass";

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

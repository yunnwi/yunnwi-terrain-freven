mod biomes;
mod blocks;
mod caves;
mod noise;
mod structures;
mod world;

use freven_world_guest_sdk::{GuestModule, LifecycleResponse, export_wasm_guest, log_info};
use world::generate;

fn make_module() -> GuestModule {
    blocks::register_blocks(GuestModule::new(blocks::MOD_ID))
        .register_worldgen_handler(blocks::TERRAIN_WORLDGEN_KEY, generate)
        .on_start_server(|_| {
            log_info!("yunnwi terrain mod loaded (server)!");
            LifecycleResponse::default().finish()
        })
        .on_tick_server(|_| LifecycleResponse::default().finish())
        .on_start_client(|_| {
            log_info!("yunnwi terrain mod loaded (client)!");
            LifecycleResponse::default().finish()
        })
        .on_tick_client(|_| LifecycleResponse::default().finish())
}

export_wasm_guest!(
    factory: make_module,
    lifecycle: [start_server, tick_server, start_client, tick_client],
    actions: false,
    worldgen: true,
);

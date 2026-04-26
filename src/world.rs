//! Freven worldgen provider implementation.
//!
//! The provider builds a local 32×96×32 block buffer, decorates it with
//! structures, compresses the result into terrain writes, and optionally emits
//! an initial spawn hint for world bootstrap.

use crate::biomes::{biome_weights, terrain_height};
use crate::blocks::*;
use crate::caves::{is_cave, is_cave_hall, is_cheese_cave};
use crate::noise::*;
use crate::structures::{place_house, place_tree};
use freven_volumetric_sdk_types::WorldCellPos;
use freven_world_guest_sdk::{
    BlockRuntimeId, InitialWorldSpawnHint, WorldGenBootstrapOutput, WorldGenCallResult,
    WorldGenContext, WorldGenOutput, WorldTerrainWrite,
};

/// Converts local `(x, y, z)` coordinates inside one 32³ section into a flat index.
pub fn sec_idx(x: usize, y: usize, z: usize) -> usize {
    x + DIM * (y + DIM * z)
}

/// Writes a runtime block ID into the local three-section column buffer.
pub fn set_world(s0: &mut [u32], s1: &mut [u32], s2: &mut [u32], x: i32, y: i32, z: i32, id: u32) {
    if x < 0 || z < 0 || x >= IDIM || z >= IDIM || y < 0 || y >= WORLD_H {
        return;
    }
    let (buf, ly) = if y < 32 {
        (&mut *s0, y as usize)
    } else if y < 64 {
        (&mut *s1, (y - 32) as usize)
    } else {
        (&mut *s2, (y - 64) as usize)
    };
    buf[sec_idx(x as usize, ly, z as usize)] = id;
}

/// Reads a runtime block ID from the local buffer. Out-of-bounds reads are air.
pub fn get_world(s0: &[u32], s1: &[u32], s2: &[u32], x: i32, y: i32, z: i32) -> u32 {
    if x < 0 || z < 0 || x >= IDIM || z >= IDIM || y < 0 || y >= WORLD_H {
        return AIR as u32;
    }
    let (buf, ly) = if y < 32 {
        (s0, y as usize)
    } else if y < 64 {
        (s1, (y - 32) as usize)
    } else {
        (s2, (y - 64) as usize)
    };
    buf[sec_idx(x as usize, ly, z as usize)]
}

/// Runtime block IDs resolved from Freven's registry for this worldgen call.
#[derive(Clone, Copy)]
pub struct GenBlockIds {
    pub stone: u32,
    pub dirt: u32,
    pub grass: u32,
    pub cobblestone: u32,
    pub log: u32,
    pub leaves: u32,
}

pub fn generate(ctx: WorldGenContext<'_>) -> WorldGenCallResult {
    let seed = ctx.init().seed;
    // Resolve numeric runtime IDs from stable string keys. Mods should not
    // assume that registered blocks always receive the same numeric IDs.
    let ids = GenBlockIds {
        stone: ctx
            .init()
            .block_id_by_key(VANILLA_STONE_KEY)
            .map(|id| id.0)
            .unwrap_or(1),
        dirt: ctx
            .init()
            .block_id_by_key(VANILLA_DIRT_KEY)
            .map(|id| id.0)
            .unwrap_or(2),
        grass: ctx
            .init()
            .block_id_by_key(VANILLA_GRASS_KEY)
            .map(|id| id.0)
            .unwrap_or(3),
        cobblestone: ctx
            .init()
            .block_id_by_key(COBBLESTONE_KEY)
            .map(|id| id.0)
            .unwrap_or(4),
        log: ctx
            .init()
            .block_id_by_key(LOG_KEY)
            .map(|id| id.0)
            .unwrap_or(5),
        leaves: ctx
            .init()
            .block_id_by_key(LEAVES_KEY)
            .map(|id| id.0)
            .unwrap_or(6),
    };
    let cx = ctx.request().cx();
    let cz = ctx.request().cz();
    let bx = cx * IDIM;
    let bz = cz * IDIM;

    let mut sec0 = vec![AIR as u32; DIM * DIM * DIM];
    let mut sec1 = vec![AIR as u32; DIM * DIM * DIM];
    let mut sec2 = vec![AIR as u32; DIM * DIM * DIM];
    let mut heights = [0i32; DIM * DIM];

    for z in 0..DIM {
        for x in 0..DIM {
            let wx = bx + x as i32;
            let wz = bz + z as i32;
            let h = terrain_height(wx as f32, wz as f32, seed) as i32;
            heights[x + DIM * z] = h;

            let bw = biome_weights(wx as f32, wz as f32, seed);
            let is_mountain = bw.mountains + bw.high_mountains > 0.5;
            let is_high = h > 55;
            let is_very_high = h > 72;

            for y in 0..=h {
                if is_cave(wx, y, wz, seed, h) {
                    continue;
                }
                if is_cheese_cave(wx, y, wz, seed, h) {
                    continue;
                }
                if is_cave_hall(wx, y, wz, seed, h) {
                    continue;
                }

                let id = if y + 7 < h {
                    ids.stone
                } else if y + 3 < h {
                    ids.dirt
                } else if is_very_high {
                    ids.stone // bare high-altitude peaks
                } else if is_mountain && is_high {
                    ids.stone // rocky mountain slopes
                } else {
                    ids.grass
                };
                set_world(&mut sec0, &mut sec1, &mut sec2, x as i32, y, z as i32, id);
            }
        }
    }

    let ch = hash2(cx, cz, seed);
    let center_bw = biome_weights((bx + IDIM / 2) as f32, (bz + IDIM / 2) as f32, seed);

    // Houses are rare and avoided in mountain-heavy chunks.
    let house_pos = if ch % 12 == 0 && center_bw.mountains + center_bw.high_mountains < 0.25 {
        let hh = hash2(cx, cz, seed.wrapping_add(42));
        let hx = (hh % 14) as i32 + 5;
        let hz = ((hh >> 8) % 14) as i32 + 5;
        let ground = heights[hx as usize + DIM * hz as usize];
        if ground >= 1 && ground + 7 < WORLD_H {
            place_house(&mut sec0, &mut sec1, &mut sec2, ids, hx, ground + 1, hz, hh);
            Some((hx, hz))
        } else {
            None
        }
    } else {
        None
    };

    // Tree count is biome-dependent. Keep trees away from houses.
    let tree_density = center_bw.forest * 9.0
        + center_bw.plains * 2.0
        + center_bw.rolling_hills * 3.0
        + center_bw.smooth_plains * 0.5;
    let count = (tree_density as usize).min(10);
    for i in 0..count {
        let th = hash2(
            cx * 31 + i as i32,
            cz * 37 + i as i32,
            seed.wrapping_add(i as u64 * 100 + 1),
        );
        let tx = (th % 22) as i32 + 5;
        let tz = ((th >> 8) % 22) as i32 + 5;

        if let Some((hx, hz)) = house_pos {
            let ddx = tx - hx;
            let ddz = tz - hz;
            if ddx * ddx + ddz * ddz < 81 {
                continue;
            } // 9-block clearance around houses
        }

        let ground = heights[tx as usize + DIM * tz as usize];
        if get_world(&sec0, &sec1, &sec2, tx, ground, tz) == ids.grass {
            place_tree(&mut sec0, &mut sec1, &mut sec2, ids, tx, ground + 1, tz, th);
        }
    }

    let mut writes = Vec::new();

    // Compress each vertical column into FillBox runs.
    // This keeps the example simple while avoiding one SetBlock write per block.
    for z in 0..DIM {
        for x in 0..DIM {
            let mut y = 0;
            while y < WORLD_H {
                let id = get_world(&sec0, &sec1, &sec2, x as i32, y, z as i32);
                if id == AIR as u32 {
                    y += 1;
                    continue;
                }

                let start_y = y;
                y += 1;
                while y < WORLD_H && get_world(&sec0, &sec1, &sec2, x as i32, y, z as i32) == id {
                    y += 1;
                }

                if y == start_y + 1 {
                    writes.push(WorldTerrainWrite::SetBlock {
                        pos: WorldCellPos::new(bx + x as i32, start_y, bz + z as i32),
                        block_id: BlockRuntimeId(id),
                    });
                } else {
                    writes.push(WorldTerrainWrite::FillBox {
                        min: WorldCellPos::new(bx + x as i32, start_y, bz + z as i32),
                        // FillBox uses half-open bounds: [min, max).
                        max: WorldCellPos::new(bx + x as i32 + 1, y, bz + z as i32 + 1),
                        block_id: BlockRuntimeId(id),
                    });
                }
            }
        }
    }

    // rc7 supports an advisory initial spawn hint. This replaces older terrain
    // shaping workarounds near (0, 0): the worldgen provider can suggest a
    // feet position and the host may validate or adjust it before persisting it.
    let bootstrap = if cx == 0 && cz == 0 {
        let spawn_x = 16usize;
        let spawn_z = 16usize;
        let spawn_y = heights[spawn_x + DIM * spawn_z] as f32 + 2.0;

        WorldGenBootstrapOutput {
            initial_world_spawn_hint: Some(InitialWorldSpawnHint {
                feet_position: [spawn_x as f32 + 0.5, spawn_y, spawn_z as f32 + 0.5],
            }),
        }
    } else {
        WorldGenBootstrapOutput::default()
    };

    WorldGenCallResult {
        output: WorldGenOutput { writes, bootstrap },
    }
}

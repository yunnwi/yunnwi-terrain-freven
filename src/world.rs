//! Freven worldgen provider implementation.
//!
//! The provider builds a local 32×96×32 block buffer, decorates it with
//! structures, compresses the result into terrain writes, and optionally emits
//! an initial spawn hint for world bootstrap.

use crate::biomes::{biome_weights, terrain_height};
use crate::blocks::*;
use crate::caves::{is_cave, is_cave_hall, is_cheese_cave};
use crate::climate::sample_climate;
use crate::geology::{SurfaceContext, slope_at, terrain_material};
use crate::noise::*;
use crate::spawn::find_world_spawn;
use crate::structures::{place_house, place_tree};
use freven_volumetric_api::{ColumnLocalCellPos, WorldGenColumnBuilder};
use freven_world_guest_sdk::{BlockRuntimeId, WorldGenCallResult, WorldGenContext};

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
    pub sand: u32,
    pub gravel: u32,
    pub snow: u32,
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
        sand: ctx
            .init()
            .block_id_by_key(SAND_KEY)
            .map(|id| id.0)
            .unwrap_or(7),
        gravel: ctx
            .init()
            .block_id_by_key(GRAVEL_KEY)
            .map(|id| id.0)
            .unwrap_or(8),
        snow: ctx
            .init()
            .block_id_by_key(SNOW_KEY)
            .map(|id| id.0)
            .unwrap_or(9),
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
            let is_mountain = bw.mountains + bw.high_mountains > 0.45;
            let slope = slope_at(wx, wz, |sx, sz| {
                terrain_height(sx as f32, sz as f32, seed) as i32
            });

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

                let id = terrain_material(
                    SurfaceContext {
                        wx,
                        wz,
                        y,
                        surface_h: h,
                        slope,
                        is_mountain,
                    },
                    seed,
                    ids,
                );

                set_world(&mut sec0, &mut sec1, &mut sec2, x as i32, y, z as i32, id);
            }
        }
    }

    let ch = hash2(cx, cz, seed);
    let center_bw = biome_weights((bx + IDIM / 2) as f32, (bz + IDIM / 2) as f32, seed);

    // Settlements should be uncommon landmarks, not frequent chunk clutter.
    //
    // Vintage Story / Minecraft style generation works better when structures
    // are sparse and memorable instead of evenly distributed.
    let house_pos = if ch % 85 == 0
        && center_bw.mountains + center_bw.high_mountains < 0.18
        && center_bw.plains + center_bw.forest > 0.55
    {
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

    // Vegetation V2.
    //
    // Tree placement is climate-, altitude-, slope-, and cluster-aware.
    // This avoids uniform scatter and creates clearer forest regions, dry open
    // areas, alpine tree lines, and natural clearings.
    let center_climate = sample_climate((bx + IDIM / 2) as f32, (bz + IDIM / 2) as f32, seed);

    let forest_noise = fbm(
        (bx as f32) / 220.0,
        (bz as f32) / 220.0,
        seed.wrapping_add(91_337),
        4,
        2.0,
        0.5,
    ) * 0.5
        + 0.5;

    let clearing_noise = fbm(
        (bx as f32) / 90.0,
        (bz as f32) / 90.0,
        seed.wrapping_add(91_338),
        3,
        2.0,
        0.5,
    ) * 0.5
        + 0.5;

    let forest_cluster = clamp01((forest_noise - 0.36) * 2.15);
    let clearing = clamp01((clearing_noise - 0.58) * 2.0);

    let climate_tree_factor = clamp01(
        center_climate.humidity * 0.75 + center_climate.forestation * 0.85
            - (center_climate.temperature - 0.82).max(0.0) * 0.9,
    );

    let alpine_penalty = if center_bw.high_mountains > 0.15 {
        0.55
    } else {
        1.0
    };
    let dry_penalty = if center_climate.humidity < 0.24 {
        0.25
    } else {
        1.0
    };

    let tree_density = (center_bw.forest * 18.0 * forest_cluster * climate_tree_factor
        + center_bw.rolling_hills * 3.2 * climate_tree_factor
        + center_bw.plains * 1.2 * climate_tree_factor
        + center_bw.smooth_plains * 0.25 * climate_tree_factor)
        * alpine_penalty
        * dry_penalty
        * (1.0 - clearing * 0.65);

    let count = (tree_density as usize).min(22);

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
            if ddx * ddx + ddz * ddz < 100 {
                continue;
            }
        }

        let wx = bx + tx;
        let wz = bz + tz;
        let ground = heights[tx as usize + DIM * tz as usize];
        let local_climate = sample_climate(wx as f32, wz as f32, seed);

        let slope = slope_at(wx, wz, |sx, sz| {
            terrain_height(sx as f32, sz as f32, seed) as i32
        });

        let too_steep = slope > 3.0;
        let too_high = ground > 54 || (ground > 38 && local_climate.temperature < 0.34);
        let too_dry = local_climate.humidity < 0.18;
        let too_hot_dry = local_climate.temperature > 0.82 && local_climate.humidity < 0.38;

        let micro_clear = fbm(
            wx as f32 / 42.0,
            wz as f32 / 42.0,
            seed.wrapping_add(91_339),
            2,
            2.0,
            0.5,
        ) * 0.5
            + 0.5;

        if too_steep || too_high || too_dry || too_hot_dry || micro_clear > 0.82 {
            continue;
        }

        if get_world(&sec0, &sec1, &sec2, tx, ground, tz) == ids.grass {
            place_tree(&mut sec0, &mut sec1, &mut sec2, ids, tx, ground + 1, tz, th);
        }
    }

    let mut out = WorldGenColumnBuilder::for_request(ctx.request());

    // rc8 builder mode: emit compact vertical runs using column-local X/Z.
    // The builder validates local bounds and converts local coordinates to
    // absolute world-cell positions for the requested column.
    for z in 0..DIM {
        for x in 0..DIM {
            let mut y = 0;

            while y < WORLD_H {
                while y < WORLD_H
                    && get_world(&sec0, &sec1, &sec2, x as i32, y, z as i32) == AIR as u32
                {
                    y += 1;
                }

                if y >= WORLD_H {
                    break;
                }

                let run_start = y;
                let id = get_world(&sec0, &sec1, &sec2, x as i32, y, z as i32);

                y += 1;
                while y < WORLD_H && get_world(&sec0, &sec1, &sec2, x as i32, y, z as i32) == id {
                    y += 1;
                }

                let local_x = u8::try_from(x).expect("local x fits u8");
                let local_z = u8::try_from(z).expect("local z fits u8");

                if y - run_start == 1 {
                    out.set_block_local(
                        ColumnLocalCellPos::new(local_x, run_start, local_z),
                        BlockRuntimeId(id),
                    )
                    .expect("valid column-local terrain cell");
                } else {
                    out.fill_vertical_run_local(local_x, local_z, run_start..y, BlockRuntimeId(id))
                        .expect("valid column-local terrain run");
                }
            }
        }
    }

    // rc8 supports an advisory initial spawn hint. This replaces older terrain
    // shaping workarounds near (0, 0): the worldgen provider can suggest a
    // feet position and the host may validate or adjust it before persisting it.
    if cx == 0 && cz == 0 {
        let (spawn_x, spawn_z, spawn_y) = find_world_spawn(seed);

        out.set_initial_world_spawn_hint([spawn_x as f32 + 0.5, spawn_y, spawn_z as f32 + 0.5]);
    }

    WorldGenCallResult {
        output: out.finish(),
    }
}

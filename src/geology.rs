//! Surface and geology material selection.
//!
//! V2 geology uses a small natural material palette:
//! grass / dirt / stone / sand / gravel / snow.

use crate::climate::sample_climate;
use crate::noise::{clamp01, fbm};
use crate::world::GenBlockIds;

#[derive(Clone, Copy, Debug)]
pub struct SurfaceContext {
    pub wx: i32,
    pub wz: i32,
    pub y: i32,
    pub surface_h: i32,
    pub slope: f32,
    pub is_mountain: bool,
}

pub fn terrain_material(ctx: SurfaceContext, seed: u64, ids: GenBlockIds) -> u32 {
    let climate = sample_climate(ctx.wx as f32, ctx.wz as f32, seed);
    let depth = ctx.surface_h - ctx.y;

    let cold = climate.temperature < 0.45;
    let dry = climate.humidity < 0.34;
    let alpine = ctx.surface_h > 38 || (ctx.surface_h > 30 && cold);
    let steep = ctx.slope > 2.0;
    let very_steep = ctx.slope > 3.8;

    let patch = fbm(
        ctx.wx as f32 / 38.0,
        ctx.wz as f32 / 38.0,
        seed.wrapping_add(44_201),
        3,
        2.0,
        0.5,
    ) * 0.5
        + 0.5;

    if depth >= 8 {
        return ids.stone;
    }

    if depth >= 3 {
        if ctx.is_mountain && ctx.slope > 2.0 {
            return ids.stone;
        }
        return ids.dirt;
    }

    if depth == 0 {
        if alpine && cold && !very_steep {
            return ids.snow;
        }

        if very_steep {
            return ids.stone;
        }

        if steep || ctx.is_mountain {
            return if patch > 0.32 { ids.gravel } else { ids.stone };
        }

        if ctx.is_mountain && patch > 0.28 {
            return ids.gravel;
        }

        if dry && ctx.surface_h <= 28 && patch > 0.30 {
            return ids.sand;
        }

        let grass_chance = clamp01(climate.humidity * 0.70 + climate.forestation * 0.35);
        if grass_chance > 0.14 {
            ids.grass
        } else if dry {
            ids.sand
        } else {
            ids.dirt
        }
    } else {
        if alpine && cold && depth <= 1 {
            return ids.snow;
        }

        if steep && depth <= 2 {
            return ids.gravel;
        }

        if dry && ctx.surface_h <= 28 && depth <= 2 {
            return ids.sand;
        }

        ids.dirt
    }
}

pub fn slope_at<F>(x: i32, z: i32, mut height_at: F) -> f32
where
    F: FnMut(i32, i32) -> i32,
{
    let h = height_at(x, z);
    let dx = (height_at(x + 1, z) - h)
        .abs()
        .max((height_at(x - 1, z) - h).abs());
    let dz = (height_at(x, z + 1) - h)
        .abs()
        .max((height_at(x, z - 1) - h).abs());
    dx.max(dz) as f32
}

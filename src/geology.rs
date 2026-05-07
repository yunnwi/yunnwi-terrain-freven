//! Surface and geology material selection.
//!
//! This is still intentionally small, but it separates material decisions from
//! terrain shape so future rock strata, ores, gravel, sand, snow, and sediment
//! systems can evolve independently.

use crate::climate::sample_climate;
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

    if depth >= 7 {
        return ids.stone;
    }

    if depth >= 3 {
        return ids.dirt;
    }

    let cold = climate.temperature < 0.28;
    let dry = climate.humidity < 0.28;
    let exposed_slope = ctx.slope > 3.5;
    let alpine = ctx.surface_h > 48 || (ctx.surface_h > 34 && cold);

    if exposed_slope || ctx.is_mountain && (ctx.surface_h > 32 || ctx.slope > 2.2) {
        return ids.stone;
    }

    if alpine && cold {
        return ids.stone;
    }

    if dry && ctx.surface_h <= 18 {
        return ids.dirt;
    }

    ids.grass
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

//! Deterministic fake river / valley field.
//!
//! This is not full hydrology yet. It is a first river-shaped signal used by
//! terrain height and surface rules. Later it can be replaced with real
//! downhill flow tracing while keeping callers mostly unchanged.

use crate::noise::*;

#[derive(Clone, Copy, Debug)]
pub struct RiverSample {
    pub river: f32,
    pub valley: f32,
}

pub fn sample_river(wx: f32, wz: f32, seed: u64) -> RiverSample {
    // Domain warp prevents rivers from looking like a regular noise grid.
    let warp_x = fbm(
        wx / 380.0,
        wz / 380.0,
        seed.wrapping_add(70_001),
        3,
        2.0,
        0.5,
    ) * 42.0;
    let warp_z = fbm(
        wx / 380.0,
        wz / 380.0,
        seed.wrapping_add(70_002),
        3,
        2.0,
        0.5,
    ) * 42.0;

    let x = wx + warp_x;
    let z = wz + warp_z;

    // Two crossing low-frequency fields. Where either approaches zero, we
    // treat it as a river corridor.
    let r1 = perlin2(x / 260.0, z / 260.0, seed.wrapping_add(70_003)).abs();
    let r2 = perlin2(
        (x + 911.0) / 420.0,
        (z - 337.0) / 420.0,
        seed.wrapping_add(70_004),
    )
    .abs();

    let channel = r1.min(r2 * 0.85);

    let river = 1.0 - smoothstep(0.012, 0.040, channel);
    let valley = 1.0 - smoothstep(0.030, 0.150, channel);

    RiverSample {
        river: clamp01(river),
        valley: clamp01(valley),
    }
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp01((x - edge0) / (edge1 - edge0));
    t * t * (3.0 - 2.0 * t)
}

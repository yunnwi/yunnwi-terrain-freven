//! Biome blending and terrain height functions.
//!
//! The terrain is intentionally deterministic: the same seed and world-space
//! coordinates always produce the same biome weights and height.

use crate::climate::sample_climate;
use crate::noise::*;
use crate::rivers::sample_river;

pub struct BiomeWeights {
    pub smooth_plains: f32,
    pub rolling_hills: f32,
    pub plains: f32,
    pub forest: f32,
    pub mountains: f32,
    pub high_mountains: f32,
}

pub fn biome_weights(wx: f32, wz: f32, seed: u64) -> BiomeWeights {
    let c = sample_climate(wx, wz, seed);

    // V2 macro landforms.
    //
    // This intentionally creates stronger regional identity than the first rc8
    // port: plains, forests, rolling hills, mountains, and high mountain belts.
    let ridge_raw = ridged(wx / 520.0, wz / 520.0, seed.wrapping_add(20_001), 4);
    let ridge = clamp01((ridge_raw - 0.42) * 2.4);

    let uplift = clamp01(
        (c.continentalness - 0.34) * 2.2
            + ridge * 0.55
            + clamp01((-c.weirdness - 0.05) * 1.6) * 0.35,
    );

    let rugged = clamp01((0.72 - c.erosion) * 1.8 + ridge * 0.45);
    let wet_forest = clamp01(c.humidity * 1.25 + c.forestation * 0.85 - 0.35);

    let high_mountains =
        clamp01((uplift - 0.62) * 2.8) * clamp01((rugged - 0.48) * 2.5) * (0.55 + ridge * 0.45);

    let mountains = clamp01((uplift - 0.42) * 2.4)
        * clamp01((rugged - 0.32) * 2.0)
        * (1.0 - high_mountains * 0.55);

    let rolling_hills = clamp01((uplift - 0.22) * 2.2)
        * clamp01((rugged - 0.18) * 1.8)
        * (1.0 - high_mountains * 0.8)
        * (1.0 - mountains * 0.45);

    let forest = wet_forest
        * (1.0 - high_mountains * 0.55)
        * (1.0 - mountains * 0.25)
        * (0.75 + c.erosion * 0.25);

    let smooth_plains =
        clamp01(c.erosion * 1.15) * clamp01(1.0 - uplift * 0.75) * clamp01(1.0 - wet_forest * 0.35);

    let plains =
        clamp01(1.0 - uplift * 0.55) * clamp01(1.0 - forest * 0.25) * (0.65 + c.erosion * 0.35);

    let sum = high_mountains + mountains + rolling_hills + forest + smooth_plains + plains + 0.0001;

    BiomeWeights {
        smooth_plains: smooth_plains / sum,
        rolling_hills: rolling_hills / sum,
        plains: plains / sum,
        forest: forest / sum,
        mountains: mountains / sum,
        high_mountains: high_mountains / sum,
    }
}

pub fn terrain_height(wx: f32, wz: f32, seed: u64) -> f32 {
    let bw = biome_weights(wx, wz, seed);

    // Each biome has its own height function. The final terrain height is a
    // weighted blend of these functions.
    let smooth_h = {
        let n = fbm(wx / 200.0, wz / 200.0, seed, 2, 2.0, 0.3);
        6.0 + (n * 0.5 + 0.5) * 2.0
    };

    let rolling_h = {
        let n1 = fbm(wx / 120.0, wz / 120.0, seed.wrapping_add(50), 4, 2.0, 0.5);
        let n2 = fbm(wx / 45.0, wz / 45.0, seed.wrapping_add(51), 2, 2.0, 0.4);
        8.0 + (n1 * 0.5 + 0.5) * 9.0 + (n2 * 0.5 + 0.5) * 3.0
    };

    let plains_h = {
        let n = fbm(wx / 110.0, wz / 110.0, seed.wrapping_add(100), 3, 2.0, 0.4);
        6.0 + (n * 0.5 + 0.5) * 4.0
    };

    let forest_h = {
        let n = fbm(wx / 95.0, wz / 95.0, seed.wrapping_add(200), 4, 2.0, 0.45);
        8.0 + (n * 0.5 + 0.5) * 9.0
    };

    let mountain_h = {
        let r = ridged(wx / 140.0, wz / 140.0, seed.wrapping_add(300), 5);
        let warp = fbm(wx / 60.0, wz / 60.0, seed.wrapping_add(302), 2, 2.0, 0.5) * 8.0;
        let base = fbm(wx / 260.0, wz / 260.0, seed.wrapping_add(303), 2, 2.0, 0.5) * 0.5 + 0.5;
        let foot = fbm(wx / 180.0, wz / 180.0, seed.wrapping_add(304), 2, 2.0, 0.5) * 0.5 + 0.5;
        20.0 + foot * 12.0 + base * 6.0 + r * 36.0 + warp * r
    };

    let high_mountain_h = {
        let ox = fbm(wx / 55.0, wz / 55.0, seed.wrapping_add(403), 3, 2.0, 0.5) * 20.0;
        let oz = fbm(wx / 55.0, wz / 55.0, seed.wrapping_add(404), 3, 2.0, 0.5) * 20.0;
        let wx2 = wx + ox;
        let wz2 = wz + oz;
        let r1 = ridged(wx2 / 190.0, wz2 / 190.0, seed.wrapping_add(400), 6);
        let r2 = ridged(wx / 65.0, wz / 65.0, seed.wrapping_add(401), 4);
        let r3 = ridged(wx2 / 110.0, wz2 / 110.0, seed.wrapping_add(406), 3);
        let warp = fbm(wx / 80.0, wz / 80.0, seed.wrapping_add(402), 2, 2.0, 0.5) * 10.0;
        let base = fbm(wx / 350.0, wz / 350.0, seed.wrapping_add(405), 2, 2.0, 0.5) * 0.5 + 0.5;
        42.0 + base * 6.0 + (r1 * 0.6 + r3 * 0.4) * 40.0 + r2 * 6.0 + warp * r1
    };

    let macro_relief = {
        let broad = fbm(wx / 420.0, wz / 420.0, seed.wrapping_add(900), 3, 2.0, 0.5) * 0.5 + 0.5;
        let ridge = ridged(wx / 360.0, wz / 360.0, seed.wrapping_add(901), 4);
        let mountain_force = bw.mountains * 0.75 + bw.high_mountains * 1.25;
        (broad * 10.0 + ridge * 18.0) * mountain_force
    };

    let river = sample_river(wx, wz, seed);

    let valley_cut = {
        let valley_noise = 1.0 - ridged(wx / 300.0, wz / 300.0, seed.wrapping_add(902), 3);
        let lowland_force = bw.plains * 0.35 + bw.forest * 0.25 + bw.smooth_plains * 0.45;
        valley_noise * lowland_force * 4.0 + river.valley * 9.0 + river.river * 3.0
    };

    let h = (smooth_h * bw.smooth_plains
        + rolling_h * bw.rolling_hills
        + plains_h * bw.plains
        + forest_h * bw.forest
        + mountain_h * bw.mountains
        + high_mountain_h * bw.high_mountains
        + macro_relief
        - valley_cut)
        .max(4.0)
        .min(WORLD_H as f32 - 5.0);

    h
}

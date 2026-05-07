//! Seed-deterministic spawn selection.
//!
//! Spawn selection should validate the generated world, not terraform it.

use crate::biomes::{biome_weights, terrain_height};
use crate::noise::{WORLD_H, hash2};

pub fn find_world_spawn(seed: u64) -> (i32, i32, f32) {
    let mut fallback = (16i32, 16i32, terrain_height(16.0, 16.0, seed) + 2.0);
    let mut best = fallback;
    let mut best_score = f32::MIN;

    for ring in 0..24 {
        let radius = 16 + ring * 24;

        for i in 0..224 {
            let h = hash2(i + ring * 1549, ring * 733, seed.wrapping_add(0xA11CE));
            let angle = (h as f32 / u32::MAX as f32) * std::f32::consts::TAU;
            let jitter_x = ((h >> 8) % 41) as i32 - 20;
            let jitter_z = ((h >> 17) % 41) as i32 - 20;

            let x = (angle.cos() * radius as f32) as i32 + jitter_x;
            let z = (angle.sin() * radius as f32) as i32 + jitter_z;

            let surface = terrain_height(x as f32, z as f32, seed);
            let sy = surface.round() as i32;

            if !(6..WORLD_H - 8).contains(&sy) {
                continue;
            }

            let mut max_delta_3 = 0.0f32;
            let mut avg_delta_3 = 0.0f32;
            let mut samples_3 = 0.0f32;

            for dz in -1..=1 {
                for dx in -1..=1 {
                    let nh = terrain_height((x + dx) as f32, (z + dz) as f32, seed);
                    let d = (nh - surface).abs();
                    max_delta_3 = max_delta_3.max(d);
                    avg_delta_3 += d;
                    samples_3 += 1.0;
                }
            }

            avg_delta_3 /= samples_3;

            if max_delta_3 > 3.5 {
                continue;
            }

            let mut max_delta_9 = 0.0f32;
            for dz in -4..=4 {
                for dx in -4..=4 {
                    let nh = terrain_height((x + dx) as f32, (z + dz) as f32, seed);
                    max_delta_9 = max_delta_9.max((nh - surface).abs());
                }
            }

            if max_delta_9 > 18.0 {
                continue;
            }

            let bw = biome_weights(x as f32, z as f32, seed);
            let mountain_weight = bw.mountains + bw.high_mountains;

            let biome_variety = bw.plains * 10.0
                + bw.forest * 10.0
                + bw.rolling_hills * 9.0
                + bw.smooth_plains * 8.0
                + bw.mountains * 7.0
                + bw.high_mountains * 5.0;

            let footing_score = 30.0 - max_delta_3 * 6.0 - avg_delta_3 * 8.0;
            let dramatic_score = mountain_weight * 8.0 + max_delta_9.min(12.0) * 0.6;
            let height_score = if sy >= 8 && sy <= 48 { 8.0 } else { 0.0 };
            let distance_penalty = ((x * x + z * z) as f32).sqrt() * 0.004;

            let score =
                biome_variety + footing_score + dramatic_score + height_score - distance_penalty;

            if best_score == f32::MIN {
                fallback = (x, z, surface + 2.0);
            }

            if score > best_score {
                best_score = score;
                best = (x, z, surface + 2.0);
            }
        }
    }

    if best_score == f32::MIN {
        fallback
    } else {
        best
    }
}

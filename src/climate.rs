//! Climate and macro landform sampling for terrain generation.
//!
//! This is the V2 worldgen foundation. It separates broad world parameters
//! from final terrain height so future systems can reuse the same fields:
//! vegetation, rivers, caves, geology, resources, structures, and spawn.

use crate::noise::*;

#[derive(Clone, Copy, Debug)]
pub struct ClimateSample {
    pub temperature: f32,
    pub humidity: f32,
    pub continentalness: f32,
    pub erosion: f32,
    pub weirdness: f32,
    pub forestation: f32,
}

pub fn sample_climate(wx: f32, wz: f32, seed: u64) -> ClimateSample {
    let latitude = temperature_latitude(wx, wz, seed);

    let temperature_noise = fbm(
        wx / 900.0,
        wz / 900.0,
        seed.wrapping_add(10_001),
        4,
        2.0,
        0.5,
    ) * 0.22;

    let humidity = norm01(fbm(
        wx / 650.0,
        wz / 650.0,
        seed.wrapping_add(10_002),
        4,
        2.0,
        0.5,
    ));

    let continentalness = norm01(fbm(
        wx / 1200.0,
        wz / 1200.0,
        seed.wrapping_add(10_003),
        5,
        2.0,
        0.5,
    ));

    let erosion = norm01(fbm(
        wx / 700.0,
        wz / 700.0,
        seed.wrapping_add(10_004),
        4,
        2.0,
        0.5,
    ));

    let weirdness = fbm(
        wx / 500.0,
        wz / 500.0,
        seed.wrapping_add(10_005),
        4,
        2.0,
        0.5,
    );

    let forest_noise = norm01(fbm(
        wx / 360.0,
        wz / 360.0,
        seed.wrapping_add(10_006),
        4,
        2.0,
        0.5,
    ));

    let temperature = clamp01(latitude + temperature_noise);
    let forestation =
        clamp01(humidity * 0.72 + forest_noise * 0.35 - dryness(temperature, humidity));

    ClimateSample {
        temperature,
        humidity,
        continentalness,
        erosion,
        weirdness,
        forestation,
    }
}

fn temperature_latitude(wx: f32, wz: f32, seed: u64) -> f32 {
    // Large-scale north/south climate bands with world-space rotation.
    // This gives a more Vintage-Story-like regional feel than pure patch noise.
    let angle = (seed as f32 * 0.000001).sin() * 0.45;
    let axis = wx * angle.sin() + wz * angle.cos();
    let band = (axis / 4200.0).sin() * 0.5 + 0.5;
    band
}

fn dryness(temperature: f32, humidity: f32) -> f32 {
    clamp01((temperature - 0.62) * 0.55 + (0.35 - humidity) * 0.45)
}

fn norm01(v: f32) -> f32 {
    clamp01(v * 0.5 + 0.5)
}

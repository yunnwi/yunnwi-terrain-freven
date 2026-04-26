use crate::noise::*;

pub struct BiomeWeights {
    pub smooth_plains: f32,
    pub rolling_hills: f32,
    pub plains: f32,
    pub forest: f32,
    pub mountains: f32,
    pub high_mountains: f32,
}

pub fn biome_weights(wx: f32, wz: f32, seed: u64) -> BiomeWeights {
    // у спавна принудительно plains
    let spawn_dist = (wx * wx + wz * wz).sqrt();
    let spawn_blend = clamp01((spawn_dist - 80.0) / 120.0);

    let cont = clamp01(
        fbm(
            wx / 700.0,
            wz / 700.0,
            seed.wrapping_add(33333),
            4,
            2.0,
            0.5,
        ) * 0.5
            + 0.5,
    );
    let humid = clamp01(
        fbm(
            wx / 450.0,
            wz / 450.0,
            seed.wrapping_add(22222),
            3,
            2.0,
            0.5,
        ) * 0.5
            + 0.5,
    );
    let erosion = clamp01(
        fbm(
            wx / 500.0,
            wz / 500.0,
            seed.wrapping_add(44444),
            3,
            2.0,
            0.5,
        ) * 0.5
            + 0.5,
    );

    let high_mountains = clamp01((cont - 0.45) * 4.5) * clamp01((1.0 - erosion) * 3.0);
    let mountains =
        clamp01((cont - 0.28) * 3.5) * (1.0 - high_mountains) * clamp01((1.0 - erosion) * 2.5);
    let forest = clamp01(humid * 2.5 - 0.2) * (1.0 - mountains - high_mountains);
    let rolling_hills = clamp01((cont - 0.15) * 3.0) * (1.0 - mountains - high_mountains - forest);
    let smooth_p =
        clamp01((1.0 - humid) * 2.0) * (1.0 - mountains - high_mountains - forest - rolling_hills);
    let plains = (1.0 - high_mountains - mountains - forest - rolling_hills - smooth_p).max(0.0);

    let sum = high_mountains + mountains + forest + rolling_hills + smooth_p + plains + 0.0001;
    let bw = BiomeWeights {
        smooth_plains: smooth_p / sum,
        rolling_hills: rolling_hills / sum,
        plains: plains / sum,
        forest: forest / sum,
        mountains: mountains / sum,
        high_mountains: high_mountains / sum,
    };

    let t = spawn_blend;
    BiomeWeights {
        smooth_plains: lerp(0.0, bw.smooth_plains, t),
        rolling_hills: lerp(0.0, bw.rolling_hills, t),
        plains: lerp(1.0, bw.plains, t),
        forest: lerp(0.0, bw.forest, t),
        mountains: lerp(0.0, bw.mountains, t),
        high_mountains: lerp(0.0, bw.high_mountains, t),
    }
}

pub fn terrain_height(wx: f32, wz: f32, seed: u64) -> f32 {
    let bw = biome_weights(wx, wz, seed);

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

    let h = (smooth_h * bw.smooth_plains
        + rolling_h * bw.rolling_hills
        + plains_h * bw.plains
        + forest_h * bw.forest
        + mountain_h * bw.mountains
        + high_mountain_h * bw.high_mountains)
        .max(4.0)
        .min(WORLD_H as f32 - 4.0);

    h
}

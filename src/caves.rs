use crate::noise::*;

pub fn is_cave(wx: i32, wy: i32, wz: i32, seed: u64, surface_h: i32) -> bool {
    // минимум 6 блоков под поверхностью
    if wy <= 1 || wy > 28 {
        return false;
    }
    if surface_h - wy < 6 {
        return false;
    }
    let scale = 16.0f32;
    let sy = scale * 0.6;
    let n1 = perlin3(
        wx as f32 / scale,
        wy as f32 / sy,
        wz as f32 / scale,
        seed.wrapping_add(7777),
    );
    let n2 = perlin3(
        wx as f32 / scale,
        wy as f32 / sy,
        wz as f32 / scale,
        seed.wrapping_add(8888),
    );
    let thr = if wy < 8 { 0.11 } else { 0.13 };
    n1.abs() < thr && n2.abs() < thr
}

pub fn is_cheese_cave(wx: i32, wy: i32, wz: i32, seed: u64, surface_h: i32) -> bool {
    // минимум 8 блоков под поверхностью
    if wy <= 2 || wy > 20 {
        return false;
    }
    if surface_h - wy < 8 {
        return false;
    }
    let scale = 24.0f32;
    let n = perlin3(
        wx as f32 / scale,
        wy as f32 / (scale * 0.7),
        wz as f32 / scale,
        seed.wrapping_add(5555),
    );
    n > 0.47
}

pub fn is_cave_hall(wx: i32, wy: i32, wz: i32, seed: u64, surface_h: i32) -> bool {
    // минимум 10 блоков под поверхностью
    if wy <= 2 || wy > 16 {
        return false;
    }
    if surface_h - wy < 10 {
        return false;
    }
    let scale = 22.0f32;
    let n = perlin3(
        wx as f32 / scale,
        wy as f32 / scale,
        wz as f32 / scale,
        seed.wrapping_add(6666),
    );
    n > 0.50
}

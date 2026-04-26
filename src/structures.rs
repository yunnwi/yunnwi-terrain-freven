//! Small deterministic structures placed after base terrain generation.
//!
//! Structures write into the local column buffer before it is converted into
//! Freven `WorldTerrainWrite`s.

use crate::noise::*;
use crate::world::{get_world, set_world};

pub fn place_tree(
    s0: &mut [u32],
    s1: &mut [u32],
    s2: &mut [u32],
    ids: crate::world::GenBlockIds,
    cx: i32,
    y: i32,
    cz: i32,
    variant: u32,
) {
    let trunk_h = 4 + (variant % 3) as i32;
    if y + trunk_h + 4 >= WORLD_H {
        return;
    }
    for dy in 0..trunk_h {
        set_world(s0, s1, s2, cx, y + dy, cz, ids.log);
    }
    let layers: &[(i32, i32)] = &[
        (trunk_h - 1, 2),
        (trunk_h, 2),
        (trunk_h + 1, 1),
        (trunk_h + 2, 1),
        (trunk_h + 3, 0),
    ];
    for &(dy, r) in layers {
        let ly = y + dy;
        for dz in -r..=r {
            for dx in -r..=r {
                if get_world(s0, s1, s2, cx + dx, ly, cz + dz) == AIR as u32 {
                    set_world(s0, s1, s2, cx + dx, ly, cz + dz, ids.leaves);
                }
            }
        }
    }
}

pub fn place_house(
    s0: &mut [u32],
    s1: &mut [u32],
    s2: &mut [u32],
    ids: crate::world::GenBlockIds,
    x: i32,
    y: i32,
    z: i32,
    variant: u32,
) {
    let w = 6 + (variant % 4) as i32;
    let d = 6 + ((variant >> 2) % 4) as i32;
    let h = 5i32;
    if x + w >= IDIM || z + d >= IDIM || y + h >= WORLD_H || y < 1 {
        return;
    }
    for dz in 0..d {
        for dx in 0..w {
            set_world(s0, s1, s2, x + dx, y - 1, z + dz, ids.cobblestone);
        }
    }
    for dy in 0..h {
        for dz in 0..d {
            for dx in 0..w {
                let wall = dx == 0 || dx == w - 1 || dz == 0 || dz == d - 1;
                let floor = dy == 0;
                let roof = dy == h - 1;
                let door = dz == 0 && (dx == w / 2 || dx == w / 2 - 1) && dy >= 1 && dy <= 3;
                if door {
                    continue;
                }
                if wall || floor || roof {
                    set_world(s0, s1, s2, x + dx, y + dy, z + dz, ids.cobblestone);
                }
            }
        }
    }
    for dz in 1..d - 1 {
        for dx in 1..w - 1 {
            set_world(s0, s1, s2, x + dx, y, z + dz, ids.dirt);
        }
    }
}

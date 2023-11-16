use std::collections::HashMap;

use def::{Block, BlockIndex, ChunkCoords};
use noise::{Fbm, NoiseFn, Perlin};

pub struct Generator {
    fbm: Fbm,
    perlin: Perlin,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            fbm: Fbm::new(),
            perlin: Perlin::new(),
        }
    }

    // determines the altitude at given position
    fn altitude(&self, x: i32, z: i32) -> i32 {
        let v1 = self.fbm.get([x as f64 / 100.0, z as f64 / 100.0]);
        let v1 = (v1 + 1.0) / 2.0;
        let v2 = self.perlin.get([x as f64 / 500.0, z as f64 / 500.0]);
        let v2 = (v2 + 1.0) / 2.0;
        let v = v1 * v2 * v2 * 100.0;
        v as i32
    }

    pub fn gen_chunk(
        &self,
        ChunkCoords { x: cx, z: cz }: ChunkCoords,
        blocks: &mut HashMap<BlockIndex, Block>,
    ) {
        for bx in 0..16 {
            for bz in 0..16 {
                let altitude = self.altitude(cx * 16 + bx, cz * 16 + bz);
                for y in 0..=altitude {
                    blocks.insert([bx, y, bz].try_into().unwrap(), {
                        let deep = (altitude - y) * altitude;
                        match altitude {
                            0..=10 => match deep {
                                0..=30 => Block::Sand,
                                _ => Block::Stone,
                            },
                            11..=35 => match deep {
                                0 => Block::Grass,
                                1..=30 => Block::Dirt,
                                _ => Block::Stone,
                            },
                            _ => match deep {
                                0..=40 => Block::Stone,
                                _ => Block::Brick,
                            },
                        }
                    });
                }
            }
        }
    }
}

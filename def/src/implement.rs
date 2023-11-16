use mat::VectorTrait;

use crate::*;

impl BlockCoords {
    /// Return the neighbour coordinate
    ///
    /// Returns `None` if the coordinate is out of the world (`0 <= y < 256`).
    pub fn step(self, direction: Direction) -> Option<Self> {
        <[i32; 3]>::from(self)
            .vector_add(direction.into())
            .try_into()
            .ok()
    }
}

impl From<[i32; 2]> for ChunkCoords {
    fn from([x, z]: [i32; 2]) -> Self {
        ChunkCoords { x, z }
    }
}
impl From<ChunkCoords> for [i32; 2] {
    fn from(ChunkCoords { x, z }: ChunkCoords) -> Self {
        [x, z]
    }
}

impl From<BlockIndex> for [i32; 3] {
    /// Decompress the block index to its position in chunk
    ///
    /// The compression is as follow: `[y:8][z:4][x:4] == [index:16]`
    fn from(BlockIndex { index }: BlockIndex) -> Self {
        [
            (index >> 0 & 0xf) as i32,
            (index >> 8 & 0xff) as i32,
            (index >> 4 & 0xf) as i32,
        ]
    }
}

impl TryFrom<[i32; 3]> for BlockIndex {
    type Error = ();

    /// Compress the block position in chunk to its index
    ///
    /// The compression is as follow: `[y:8][z:4][x:4] == [index:16]`
    fn try_from([x, y, z]: [i32; 3]) -> Result<Self, Self::Error> {
        match [x, y, z] {
            [0..=15, 0..=255, 0..=15] => Ok(BlockIndex {
                index: ((x as u16) << 0) | ((z as u16) << 4) | ((y as u16) << 8),
            }),
            _ => Err(()),
        }
    }
}

impl From<BlockCoords> for [i32; 3] {
    fn from(BlockCoords(ChunkCoords { x, z }, index): BlockCoords) -> Self {
        let [ix, iy, iz]: [i32; 3] = index.into();
        [x * 16 + ix, iy, z * 16 + iz]
    }
}

impl From<Direction> for [i32; 3] {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::East => [1, 0, 0],
            Direction::West => [-1, 0, 0],
            Direction::Up => [0, 1, 0],
            Direction::Down => [0, -1, 0],
            Direction::South => [0, 0, 1],
            Direction::North => [0, 0, -1],
        }
    }
}

impl TryFrom<[i32; 3]> for BlockCoords {
    type Error = ();

    fn try_from([x, y, z]: [i32; 3]) -> Result<Self, ()> {
        let by: u8 = y.try_into().map_err(|_| ())?;
        let [cx, cz] = [x >> 4, z >> 4];
        let [bx, bz] = [x & 0xf, z & 0xf];
        Ok(BlockCoords(
            ChunkCoords { x: cx, z: cz },
            BlockIndex {
                index: (bx as u16) << 0 | (bz as u16) << 4 | (by as u16) << 8,
            },
        ))
    }
}

impl From<BlockCoords> for [f32; 3] {
    fn from(bc: BlockCoords) -> Self {
        let [x, y, z]: [i32; 3] = bc.into();
        [x as f32, y as f32, z as f32]
    }
}

impl TryFrom<[f32; 3]> for BlockCoords {
    type Error = ();

    fn try_from(vector: [f32; 3]) -> Result<Self, Self::Error> {
        vector.map(|v| v.floor() as i32).try_into()
    }
}

impl Direction {
    pub fn oposit(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
    pub const ALL: [Self; 6] = [
        Self::North,
        Self::South,
        Self::East,
        Self::West,
        Self::Up,
        Self::Down,
    ];
    pub const CARDINAL: [Self; 4] = [Self::North, Self::South, Self::East, Self::West];
    pub const fn face_vertices(self) -> [[i32; 3]; 4] {
        match self {
            Self::North => [[0, 0, 0], [0, 1, 0], [1, 1, 0], [1, 0, 0]],
            Self::West => [[0, 0, 1], [0, 1, 1], [0, 1, 0], [0, 0, 0]],
            Self::South => [[1, 0, 1], [1, 1, 1], [0, 1, 1], [0, 0, 1]],
            Self::East => [[1, 0, 0], [1, 1, 0], [1, 1, 1], [1, 0, 1]],
            Self::Up => [[0, 1, 0], [0, 1, 1], [1, 1, 1], [1, 1, 0]],
            Self::Down => [[0, 0, 1], [0, 0, 0], [1, 0, 0], [1, 0, 1]],
        }
    }
    pub fn light(self) -> f32 {
        match self {
            Self::North => 0.7,
            Self::South => 0.1,
            Self::East => 0.1,
            Self::West => 0.4,
            Self::Up => 1.0,
            Self::Down => 0.0,
        }
    }

    pub fn from_vector([x, y, z]: [f32; 3]) -> [Option<(Self, f32)>; 3] {
        [
            if x < 0.0 {
                Some((Self::West, x.abs()))
            } else if x > 0.0 {
                Some((Self::East, x))
            } else {
                None
            },
            if y < 0.0 {
                Some((Self::Down, y.abs()))
            } else if y > 0.0 {
                Some((Self::Up, y))
            } else {
                None
            },
            if z < 0.0 {
                Some((Self::North, z.abs()))
            } else if z > 0.0 {
                Some((Self::South, z))
            } else {
                None
            },
        ]
    }
}

impl ChunkCoords {
    pub fn iter_range(self, range: u8) -> ChunkRangeIter {
        let x_start = self.x - range as i32;
        let z_start = self.z - range as i32;
        let x_end = self.x + range as i32 + 1;
        let z_end = self.z + range as i32 + 1;
        ChunkRangeIter {
            x: x_start,
            z: z_start,
            x_start,
            x_end,
            z_end,
        }
    }
    pub fn neighbor(self, direction: Direction) -> Self {
        let [dx, dy, dz]: [i32; 3] = direction.into();
        debug_assert_eq!(dy, 0);
        let [cx, cz]: [i32; 2] = self.into();
        [cx + dx, cz + dz].into()
    }
    pub fn neighbors(self) -> [ChunkCoords; 4] {
        Direction::CARDINAL.map(|d| self.neighbor(d))
    }
    pub fn from_position([x, _, z]: [f32; 3]) -> Self {
        Self {
            x: x.floor() as i32 >> 4,
            z: z.floor() as i32 >> 4,
        }
    }
    pub fn in_range(self, other: Self, range: i32) -> bool {
        let dx = self.x - other.x;
        let dz = self.z - other.z;
        dx * dx + dz * dz <= range * range
    }
}
impl Iterator for ChunkRangeIter {
    type Item = ChunkCoords;

    fn next(&mut self) -> Option<Self::Item> {
        if self.z == self.z_end {
            None
        } else if self.x == self.x_end {
            self.x = self.x_start;
            self.z += 1;
            self.next()
        } else {
            let item = ChunkCoords {
                x: self.x,
                z: self.z,
            };
            self.x += 1;
            Some(item)
        }
    }
}

impl BlockIndex {
    pub const ALL: BlockIndexIter = BlockIndexIter {
        index: 0,
        fused: false,
    };
}

impl Iterator for BlockIndexIter {
    type Item = BlockIndex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.fused {
            None
        } else {
            let result = self.index;
            if let Some(index) = self.index.checked_add(1) {
                self.index = index;
            } else {
                self.fused = true;
            }
            Some(BlockIndex { index: result })
        }
    }
}

impl Block {
    pub fn color(self, direction: Direction) -> [f32; 3] {
        let [sun_r, sun_g, sun_b] = [1.0, 0.8, 0.5];
        let sun = match direction {
            Direction::Down => 0.0,
            Direction::Up => 1.0,
            Direction::South => 0.7,
            Direction::North => 0.1,
            Direction::East => 0.5,
            Direction::West => 0.3,
        };
        let [r, g, b] = match self {
            Self::Brick => [0.2, 0.2, 0.2],
            Self::Dirt => [0.5, 0.3, 0.2],
            Self::Grass => [0.1, 0.6, 0.2],
            Self::Sand => [0.7, 0.7, 0.4],
            Self::Stone => [0.4, 0.4, 0.4],
            _ => unimplemented!(),
        };
        [
            0.6 * r + 0.4 * (sun_r * r * sun),
            0.6 * g + 0.4 * (sun_g * g * sun),
            0.6 * b + 0.4 * (sun_b * b * sun),
        ]
    }
    pub fn sprite(self, direction: Direction) -> Sprite {
        match (self, direction) {
            (Self::Grass, Direction::Up) => Sprite::GrassTop,
            (Self::Grass, Direction::Down) => Sprite::Dirt,
            (Self::Grass, _) => Sprite::GrassSide,
            (Self::Stone, _) => Sprite::Stone,
            (Self::Sand, _) => Sprite::Sand,
            (Self::Dirt, _) => Sprite::Dirt,
            (Self::Brick, _) => Sprite::Brick,
            (Self::Glass, _) => Sprite::Glass,
            (Self::Trunk, Direction::Up | Direction::Down) => Sprite::TrunkTop,
            (Self::Trunk, _) => Sprite::TrunkSide,
            (Self::Water, _) => Sprite::Water,
            _ => unimplemented!(),
        }
    }
}

impl Boxel {
    pub fn new(dimensions: [f32; 3], center: [f32; 3], pos: [f32; 3]) -> Self {
        Self {
            dimensions,
            pos: pos.vector_sub(center),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_coords_convert() {
        for x in -128..128 {
            for z in -128..128 {
                for y in 0..=255 {
                    let bc: BlockCoords = [x, y, z].try_into().unwrap();
                    let vector: [i32; 3] = bc.into();
                    assert_eq!(vector, [x, y, z]);
                }
            }
        }
    }
}

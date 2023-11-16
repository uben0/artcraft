pub mod cube;
mod implement;
mod ray_travel;

pub use ray_travel::RayTravel;

/// Any block can be identified by its chunk and index
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockCoords(pub ChunkCoords, pub BlockIndex);

/// The block index in its chunk
///
/// It is the concatenation of x (4 bits), z (4 bits) and y (8 bits).
/// Because a chunk is 16x16x256 blocks.
/// The compression is as follow: `[y:8][z:4][x:4] == [index:16]`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockIndex {
    pub index: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockIndexIter {
    index: u16,
    fused: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoords {
    pub x: i32,
    pub z: i32,
}
pub struct ChunkRangeIter {
    x: i32,
    z: i32,
    x_start: i32,
    x_end: i32,
    z_end: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Block {
    Stone,
    Dirt,
    Grass,
    Sand,
    Water,
    Glass,
    Brick,
    Trunk,
    Leaves,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Sprite {
    Stone = 0,
    Dirt = 1,
    GrassTop = 2,
    GrassSide = 3,
    Sand = 4,
    Brick = 5,
    Glass = 6,
    Water = 7,
    TrunkTop = 8,
    TrunkSide = 9,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

/// Defines how a given voxel occupies space
///
/// A conversion from integer voxel coordinates to decimal vector is
/// necessary when representing a block in OpenGL
#[derive(Debug, Clone, Copy)]
pub struct Boxel {
    pub pos: [f32; 3],
    pub dimensions: [f32; 3],
}

pub mod constant {
    pub const GRAVITY: f32 = -0.01;
    pub const JUMP: f32 = 0.15;
    pub const COLLISION_EPSILON: f32 = 0.001;
}

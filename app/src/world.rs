use std::{collections::HashMap, sync::RwLock};

use arrayvec::ArrayVec;
use dashmap::DashMap;
use def::{Block, BlockCoords, BlockIndex, Boxel, ChunkCoords, Direction};
use mat::VectorTrait;

mod generator;
use generator::Generator;
use tokio::sync::mpsc::Sender;

use crate::AristideCmd;
use crate::{camera::Camera, Cmd};

#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub camera: Camera,
    pub fly: bool,
    pub gravity: f32,
    pub on_ground: bool,
    pub block_placing: Block,
}

/// State of a chunk
///
/// First, the chunk data is loaded (generated), next
/// its mesh is built. Different stage are requiered as
/// building the mesh requires to know neighbours chunk data
/// which would themself require their neighbour to be
/// loaded. Seperating the state in two stages breaks this
/// infinite dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChunkStage {
    None,
    Loaded,
    Meshed,
}
impl ChunkStage {
    fn previous(self) -> Option<Self> {
        match self {
            Self::None => None,
            Self::Loaded => Some(Self::None),
            Self::Meshed => Some(Self::Loaded),
        }
    }
}

pub enum ChunkState {
    Loaded(BlocksChunk),
    Meshed(BlocksChunk, FacesChunk),
}
impl ChunkState {
    fn get_block(&self, bi: BlockIndex) -> Option<Block> {
        match self {
            ChunkState::Loaded(blocks_chunk) => blocks_chunk.get(&bi).copied(),
            ChunkState::Meshed(blocks_chunk, _) => blocks_chunk.get(&bi).copied(),
        }
    }
    fn get_stage(&self) -> ChunkStage {
        match self {
            ChunkState::Loaded(_) => ChunkStage::Loaded,
            ChunkState::Meshed(_, _) => ChunkStage::Meshed,
        }
    }
}

pub struct World {
    /// send command to the supervisor (Beatrice)
    pub sender_cmd: Sender<Cmd>,
    /// send command to the rendering loop (Aristide)
    pub aristide_cmd: Sender<AristideCmd>,
    // a concurrent hashmap is used here (dashmap), allowing
    // different threads to read and update the chunks.
    pub chunks: DashMap<ChunkCoords, ChunkState>,
    player: RwLock<Player>,
    /// terrain generator (holds perlin noise configuration)
    pub generator: Generator,
}

pub type BlocksChunk = HashMap<BlockIndex, Block>;
pub type FacesChunk = HashMap<(BlockIndex, Direction), Block>;

impl World {
    /// create a new world
    pub fn new(sender_cmd: Sender<Cmd>, update_chunk_mesh: Sender<AristideCmd>) -> Self {
        Self {
            sender_cmd,
            aristide_cmd: update_chunk_mesh,
            chunks: DashMap::new(),
            player: RwLock::new(Player {
                camera: Camera {
                    pos: [0.0, 20.0, 0.0],
                    h_angle: 0.0,
                    v_angle: 0.0,
                },
                fly: true,
                gravity: 0.0,
                on_ground: false,
                block_placing: Block::Stone,
            }),
            generator: Generator::new(),
        }
    }

    pub fn player_set_block_placing(&self, block: Block) {
        self.player.write().unwrap().block_placing = block;
    }

    pub fn player_fly(&self, b: bool) {
        self.player.write().unwrap().fly = b;
        println!("player.fly set to {:?}", b);
    }

    /// fetch player data
    pub fn pull_player(&self) -> Player {
        *self.player.read().unwrap()
    }
    /// update player data
    pub fn push_player(&self, player: Player) {
        *self.player.write().unwrap() = player;
    }

    /// When chunk data is altered (block placed or removed) its meshed is recomputed
    ///
    /// This function only update the given block position, but returns true or false
    /// if a changed took place, then a cascading effect on neighbours is applied
    pub fn update_block_mesh(&self, BlockCoords(cc, bi): BlockCoords) -> bool {
        let mut updated = false;
        // as the meshed is optimized to not render hidden faces,
        // neighbour blocks are check if present or not
        let neighbours = Direction::ALL.map(|direction| {
            (
                direction,
                BlockCoords(cc, bi)
                    .step(direction)
                    .map(|position| self.get_block(position))
                    .flatten()
                    .flatten(),
            )
        });
        if let Some(mut chunk) = self.chunks.get_mut(&cc) {
            if let ChunkState::Meshed(ref mut blocks, ref mut faces) = *chunk {
                // a block has been placed
                if let Some(&block) = blocks.get(&bi) {
                    for (direction, neighbour) in neighbours {
                        if neighbour.is_some() {
                            if faces.remove(&(bi, direction)).is_some() {
                                updated = true;
                            }
                        } else {
                            if faces.insert((bi, direction), block).is_none() {
                                updated = true;
                            }
                        }
                    }
                } else {
                    // a block has been removed
                    for direction in Direction::ALL {
                        if faces.remove(&(bi, direction)).is_some() {
                            updated = true;
                        }
                    }
                }
            }
        }
        // if the current block has changed, return true
        updated
    }

    pub fn remove_block(&self, bc: BlockCoords) {
        // converts block coordinates to chunk coordinates and block index
        let BlockCoords(cc, bi) = bc;
        // at most 7 updated block (6 neighbour and the block itself)
        // an ArrayVec is a dynamic array on the stack (max sized)
        let mut updates = ArrayVec::<BlockCoords, 7>::new();
        if let Some(mut chunk) = self.chunks.get_mut(&cc) {
            if let ChunkState::Meshed(ref mut blocks, _) = *chunk {
                if blocks.remove(&bi).is_some() {
                    if !updates.contains(&bc) {
                        // only add update if not yet present in list
                        updates.push(bc);
                    }
                    for direction in Direction::ALL {
                        // cascading effect on neighbours
                        if let Some(neighbour) = bc.step(direction) {
                            updates.push(neighbour);
                        }
                    }
                }
            }
        }
        // which chunks where updated (theorical maximum is 3, but
        // for some complicated reasons, it's better to put 7)
        let mut updated = ArrayVec::<ChunkCoords, 7>::new();
        for bc in updates {
            if self.update_block_mesh(bc) {
                let BlockCoords(cc, _) = bc;
                if !updated.contains(&cc) {
                    updated.push(cc);
                }
            }
        }
        for chunk in updated {
            self.aristide_cmd
                .try_send(AristideCmd::RenderChunk(chunk, true))
                .ok();
        }
    }
    // similar to remove_block
    pub fn place_block(&self, bc: BlockCoords, block: Block) {
        let BlockCoords(cc, bi) = bc;
        let mut updates = ArrayVec::<BlockCoords, 7>::new();
        if let Some(mut chunk) = self.chunks.get_mut(&cc) {
            if let ChunkState::Meshed(ref mut blocks, _) = *chunk {
                if blocks.insert(bi, block).is_none() {
                    if !updates.contains(&bc) {
                        updates.push(bc);
                    }
                    for direction in Direction::ALL {
                        if let Some(neighbour) = bc.step(direction) {
                            updates.push(neighbour);
                        }
                    }
                }
            }
        }
        let mut updated = ArrayVec::<ChunkCoords, 7>::new();
        for bc in updates {
            if self.update_block_mesh(bc) {
                let BlockCoords(cc, _) = bc;
                if !updated.contains(&cc) {
                    updated.push(cc);
                }
            }
        }
        for chunk in updated {
            self.aristide_cmd
                .try_send(AristideCmd::RenderChunk(chunk, true))
                .ok();
        }
    }

    pub fn get_chunk_stage(&self, cc: ChunkCoords) -> ChunkStage {
        self.chunks
            .get(&cc)
            .map(|chunk| chunk.get_stage())
            .unwrap_or(ChunkStage::None)
    }

    pub fn get_block(&self, BlockCoords(cc, bi): BlockCoords) -> Option<Option<Block>> {
        self.chunks.get(&cc).map(|chunk| chunk.get_block(bi))
    }

    /// Load the given chunk
    pub fn chunk_stage_none_to_loaded(&self, cc: ChunkCoords) {
        let mut chunk = BlocksChunk::new();
        self.generator.gen_chunk(cc, &mut chunk);
        self.chunks.insert(cc, ChunkState::Loaded(chunk));
    }

    /// Build mesh of given chunk
    pub fn chunk_stage_loaded_to_meshed(&self, cc: ChunkCoords) {
        let mut faces_chunk = FacesChunk::new();
        // TODO: very inefficient to iterate over all possible indices
        // should only iterate over stored block
        for bi in BlockIndex::ALL {
            let bc = BlockCoords(cc, bi);
            if let Some(Some(block)) = self.get_block(bc) {
                for direction in Direction::ALL {
                    if let Some(Some(None)) = bc.step(direction).map(|bc| self.get_block(bc)) {
                        faces_chunk.insert((bi, direction), block);
                    }
                }
            }
        }
        // TODO: this is bad, between the time the chunk is removed then
        // reinserted, the chunk loader could decide to load it again
        // beleiving it is not.
        if let Some((_, ChunkState::Loaded(chunk))) = self.chunks.remove(&cc) {
            self.chunks
                .insert(cc, ChunkState::Meshed(chunk, faces_chunk));
        } else {
            unreachable!()
        }
    }

    // apply dependency of chunk stages to given chunk and its neighbours
    pub fn request_chunk_stage(&self, cc: ChunkCoords, stage: ChunkStage) {
        let chunk_stage = self.get_chunk_stage(cc);
        if chunk_stage < stage {
            let previous = stage.previous().unwrap();
            self.request_chunk_stage(cc, previous);
            for neighbour in cc.neighbors() {
                self.request_chunk_stage(neighbour, previous);
            }
            match previous {
                ChunkStage::None => self.chunk_stage_none_to_loaded(cc),
                ChunkStage::Loaded => self.chunk_stage_loaded_to_meshed(cc),
                ChunkStage::Meshed => unreachable!(),
            }
        }
    }

    pub async fn aristide_cmd(&self, cmd: AristideCmd) {
        self.aristide_cmd.send(cmd).await.unwrap()
    }

    // it workds, don't ask me to explain it XD
    fn find_collision_tranch<const X: usize, const Y: usize, const Z: usize>(
        &self,
        x: i32,
        t: f32,
        boxel: Boxel,
        vector: [f32; 3],
    ) -> bool {
        const E: f32 = def::constant::COLLISION_EPSILON;
        // COMPUTE TRANCH (move the hitbox to future position)
        let pos_min = boxel.pos.vector_add(vector.vector_scale(t));
        let pos_max = pos_min.vector_add(boxel.dimensions);

        // COVER DISCRET TRANCH (let X be the progression axis)
        // then find out the rectangle the hitbox is producing on Y and Z axis

        let y_begin = (pos_min[Y] + E).floor() as i32;
        let y_end = (pos_max[Y] - E).ceil() as i32;
        for y in y_begin..y_end {
            // iterate over all crossed integer values of Y axis

            let z_begin = (pos_min[Z] + E).floor() as i32;
            let z_end = (pos_max[Z] - E).ceil() as i32;
            for z in z_begin..z_end {
                // iterate over all crossed integer values of Z axis

                let mut bc = [0; 3];
                bc[X] = x;
                bc[Y] = y;
                bc[Z] = z;
                // if one of those values is the coordinate of solid block
                if let Ok(bc) = BlockCoords::try_from(bc) {
                    if let Some(Some(_)) = self.get_block(bc) {
                        // then YES a collision occurs
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn find_collision_x(&self, boxel: Boxel, vector: [f32; 3]) -> f32 {
        // axis map: [x, y, z]
        self.find_collision::<0, 1, 2>(boxel, vector)
    }

    pub fn find_collision_y(&self, boxel: Boxel, vector: [f32; 3]) -> f32 {
        // axis map: [y, x, z]
        self.find_collision::<1, 0, 2>(boxel, vector)
    }

    pub fn find_collision_z(&self, boxel: Boxel, vector: [f32; 3]) -> f32 {
        // axis map:: [z, x, y]
        self.find_collision::<2, 0, 1>(boxel, vector)
    }

    // to avoid repetition, this function is agnostic over the axis
    fn find_collision<const X: usize, const Y: usize, const Z: usize>(
        &self,
        boxel: Boxel,
        vector: [f32; 3],
    ) -> f32 {
        const E: f32 = def::constant::COLLISION_EPSILON;
        let mut min_time = 1.0;
        let vx = vector[X];

        // toward positive X
        if vx > 0.0 {
            let x_begin = boxel.pos[X] + boxel.dimensions[X];
            let x_end = x_begin + vx;

            // find min time
            for x in (x_begin - E).ceil() as i32..=(x_end + E).floor() as i32 {
                let time = (x as f32 - x_begin) / (x_end - x_begin);
                if self.find_collision_tranch::<X, Y, Z>(x, time, boxel, vector) {
                    min_time = time.min(min_time);
                }
            }
        }

        // toward negative X
        if vx < 0.0 {
            let x_begin = boxel.pos[X];
            let x_end = x_begin + vx;

            // find min time
            for x in (x_end - E).ceil() as i32..=(x_begin + E).floor() as i32 {
                let time = (x as f32 - x_begin) / (x_end - x_begin);
                if self.find_collision_tranch::<X, Y, Z>(x - 1, time, boxel, vector) {
                    min_time = time.min(min_time);
                }
            }
        }

        min_time
    }
}

#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub grammar);

use std::{sync::Arc, thread};

use def::{Block, BlockCoords, ChunkCoords};
use tokio::sync::mpsc;
use world::World;

mod aristide;
mod beatrice;
mod camera;
mod cassiope;
mod mesh;
mod world;

#[derive(Debug, Clone)]
pub enum Cmd {
    RemoveBlock(BlockCoords),
    PlaceBlock(BlockCoords, Block),
    Fly(bool),
    BlockPlacing(Block),
}

#[derive(Debug, Clone)]
pub enum AristideCmd {
    RenderChunk(ChunkCoords, bool),
}

fn main() {
    let (sender_chunk_mesh, receiver_chunk_mesh) = mpsc::channel(40);
    let (sender_cmd, receiver_cmd) = mpsc::channel(40);

    let world_a = Arc::new(World::new(sender_cmd, sender_chunk_mesh));
    let world_b = world_a.clone();
    let world_c = world_a.clone();

    thread::spawn(move || beatrice::beatrice(receiver_cmd, world_b));
    thread::spawn(move || cassiope::cassiope(world_c));
    aristide::aristide(receiver_chunk_mesh, world_a);
}

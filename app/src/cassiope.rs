use std::collections::HashSet;
use std::{sync::Arc, time::Duration};

use def::ChunkCoords;
use tokio::{runtime, task::LocalSet};

use crate::world::{ChunkStage, World};
use crate::AristideCmd;

async fn chunk_loader(world: &World) -> Option<()> {
    let mut rendered_chunk: HashSet<ChunkCoords> = HashSet::new();

    // loop every 200 milliseconds and check for player pos to load or unload chunks
    loop {
        // player pos
        let center = ChunkCoords::from_position({
            let player = world.pull_player();
            player.camera.pos
        });

        // unload is further than 16 chunks
        const POP_OUT: i32 = 16;
        // load if clother than 8 chunks
        const POP_IN: i32 = 8;

        for chunk in rendered_chunk
            .iter()
            .filter(|v| !v.in_range(center, POP_OUT))
        {
            // ask Aristide to drop associated mesh
            // only Aristide can do it as the handle to OpenGL
            // cannot be shared between threads
            world
                .aristide_cmd(AristideCmd::RenderChunk(*chunk, false))
                .await;
        }

        // now forgot about them
        rendered_chunk.retain(|v| v.in_range(center, POP_OUT));

        // iterate over visible area (square area)
        for x in center.x - POP_IN..=center.x + POP_IN {
            for z in center.z - POP_IN..=center.z + POP_IN {
                let chunk = ChunkCoords { x, z };
                // only take if inside inscribed circle (circular area)
                if chunk.in_range(center, POP_IN) {
                    if !rendered_chunk.contains(&chunk) {
                        // if not rendered, generate mesh
                        rendered_chunk.insert(chunk);
                        world.request_chunk_stage(chunk, ChunkStage::Meshed);
                        // and inform Aristide it can upload mesh to GPU and render it
                        world
                            .aristide_cmd(AristideCmd::RenderChunk(chunk, true))
                            .await;
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(200)).await
    }
}

pub fn cassiope(world: Arc<World>) {
    // use asynchronous runtime simulating multiple threads with only one system thread
    //
    // in the current state, it's not mandatory because only one task is spawned,
    // but it allows additional tasks to be added in future
    let rt = runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    rt.block_on(async {
        let local = LocalSet::new();
        let world_ref = world.as_ref();
        local
            .run_until(async move {
                chunk_loader(world_ref).await;
            })
            .await;
    })
}

use std::sync::Arc;

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    runtime,
    sync::mpsc::Receiver,
    task::LocalSet,
};

use crate::{grammar::CmdParser, world::World, Cmd};

pub fn beatrice(mut cmd_receiver: Receiver<Cmd>, world: Arc<World>) {
    // use asynchronous runtime to simulate multiple threads in one system thread
    let rt = runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    rt.block_on(async {
        let local = LocalSet::new();
        let world2 = world.clone();

        local.spawn_local(async move {
            // receive global program command and dispatch them
            while let Some(cmd) = cmd_receiver.recv().await {
                match cmd {
                    Cmd::BlockPlacing(block) => {
                        world.player_set_block_placing(block);
                    }
                    Cmd::RemoveBlock(bc) => {
                        world.remove_block(bc);
                    }
                    Cmd::PlaceBlock(bc, block) => {
                        world.place_block(bc, block);
                    }
                    Cmd::Fly(b) => {
                        world.player_fly(b);
                    }
                }
            }
        });

        local.spawn_local(async move {
            // listen for terminal user input and parse it as a command
            let mut buffer = String::new();
            let parser = CmdParser::new();
            let mut reader = BufReader::new(tokio::io::stdin());
            while let Ok(_) = reader.read_line(&mut buffer).await {
                match parser.parse(buffer.as_str()) {
                    Ok(cmd) => world2.sender_cmd.send(cmd).await.unwrap(),
                    Err(err) => println!("{err}"),
                }
                buffer.clear();
            }
        });

        local.await;
    });
}

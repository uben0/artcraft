use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use def::{cube, Boxel, ChunkCoords, RayTravel};
use glium::{
    glutin::{
        event::{
            DeviceEvent, ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode,
            WindowEvent,
        },
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        ContextBuilder,
    },
    index::PrimitiveType,
    texture::RawImage2d,
    DepthTest, Display, Frame, Surface,
};
use glium::{texture::SrgbTexture2dArray, Program};
use mat::{Affine, AffineTrait, MatrixTrait, VectorTrait};
use tokio::sync::mpsc::Receiver;

mod control;
use control::Control;
mod chunk_loader;
use chunk_loader::ChunkLoader;

use crate::{
    mesh::{ColoredMesh, Drawable, TexturedMesh},
    world::World,
    AristideCmd, Cmd,
};

const FRAME_DURATION: Duration = Duration::from_nanos(16_666_667);

fn aspect_ratio((width, height): (u32, u32)) -> [[f32; 4]; 4] {
    [
        [(height as f32 / width as f32), 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 1.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn perspective(fov: f32) -> [[f32; 4]; 4] {
    let f = 1.0 / (fov / 2.0).tan();
    let zfar = 1024.0;
    let znear = 0.1;
    let deno = zfar - znear;
    [
        [f, 0.0, 0.0, 0.0],
        [0.0, -f, 0.0, 0.0],
        [0.0, 0.0, (zfar + znear) / deno, 1.0],
        [0.0, 0.0, -(2.0 * zfar * znear) / deno, 0.0],
    ]
}

fn load_textures(display: &Display) -> SrgbTexture2dArray {
    // Textures are directly embeded in the executable
    SrgbTexture2dArray::new(
        display,
        [
            include_bytes!("aristide/textures/0.png").as_slice(),
            include_bytes!("aristide/textures/1.png").as_slice(),
            include_bytes!("aristide/textures/2.png").as_slice(),
            include_bytes!("aristide/textures/3.png").as_slice(),
            include_bytes!("aristide/textures/4.png").as_slice(),
            include_bytes!("aristide/textures/5.png").as_slice(),
            include_bytes!("aristide/textures/6.png").as_slice(),
            include_bytes!("aristide/textures/7.png").as_slice(),
            include_bytes!("aristide/textures/8.png").as_slice(),
            include_bytes!("aristide/textures/9.png").as_slice(),
        ]
        .iter()
        .map(std::io::Cursor::new)
        .map(|v| {
            let v = image::load(v, image::ImageFormat::Png).unwrap().to_rgba8();
            let dimensions = v.dimensions();
            RawImage2d::from_raw_rgba_reversed(&v.into_raw(), dimensions)
        })
        .collect(),
    )
    .unwrap()
}

struct Renderer {
    cursor: ColoredMesh, // A mesh is a bundle of vertices and indices (triangles)
    block_select: ColoredMesh,
    colored_program: Program,  // Fragment shader
    textured_program: Program, // Fragment shader
    world: Arc<World>,
    receiver_cmd: Receiver<AristideCmd>, // Receive commands from other threads
    chunk_loader: ChunkLoader,
    rendered_chunk: HashMap<ChunkCoords, TexturedMesh>,
    textures: SrgbTexture2dArray,
}
impl Renderer {
    fn new(
        display: &Display,
        world: Arc<World>,
        receiver_from_cassiope_chunk: Receiver<AristideCmd>,
    ) -> Self {
        Self {
            // Load shader for colored mesh
            colored_program: ColoredMesh::program(display),
            // Load shader for textured mesh
            textured_program: TexturedMesh::program(display),
            // Load mesh for cube highlighting
            block_select: {
                ColoredMesh::new(
                    &display,
                    &cube::LINE_VERTICES.map(|v| (v.map(|c| c as f32), [0.0, 0.0, 0.0]).into()),
                    &cube::LINE_INDICES,
                    PrimitiveType::LinesList,
                )
                .depth_test(DepthTest::IfLessOrEqual)
                .line_width(2.0)
            },
            // Load cursor mesh
            cursor: ColoredMesh::new(
                &display,
                &[([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]).into()],
                &[0],
                PrimitiveType::Points,
            )
            .point_size(4.0),
            world,
            receiver_cmd: receiver_from_cassiope_chunk,
            chunk_loader: ChunkLoader::new(),
            rendered_chunk: HashMap::new(),
            textures: load_textures(&display),
        }
    }

    fn render(&self, mut target: Frame) {
        // it's definitely not the field of view
        // the field of view can be tweaked with it
        // but it's not actual degrees
        const FOV: f32 = 80.6;

        // window dimension in pixels
        let (width, height) = target.get_dimensions();
        target.clear_color_and_depth((0.5, 0.5, 1.0, 1.0), 1.0);

        // fetch player info (because it's memory shared between threads)
        let camera = self.world.pull_player().camera;
        let camera_project = camera.projector();

        // render all the chunks
        for (&cc, mesh) in self.rendered_chunk.iter() {
            let [cx, cz]: [i32; 2] = cc.into();
            mesh.draw(
                &self.textured_program, // The shader handling textured mesh
                &mut target,            // the window (OpenGL canvas)
                aspect_ratio((width, height)) // The transform matrix
                    .matrix_mul(perspective(FOV)) // Apply screen view (with field of view)
                    .matrix_mul(camera_project) // Apply camera transform (player position and orientation)
                    .affine_translate([cx * 16, 0, cz * 16].map(|v| v as f32)), // Apply local transform (chunk position)
                &self.textures,
            )
        }
        {
            // This wall part is only there to render the highlight on the pointed cube
            // When the player points a cube and the cube is at reach (less than 10 meters)
            // A black grid appear around the cube

            // Player's forward vector (where player is looking at)
            let [cx, cy, cz, _] = camera.matrix().vector_z();

            // Iterate over all voxel coordinates the vector is traversing
            for position in RayTravel::new(camera.pos, [cx, cy, cz], 10.0) {
                // Check if the obtained coordinate is not out of the world
                if let Some((position, _direction)) = position {
                    // Check if a block is present at this coordinate
                    if let Some(Some(_)) = self.world.get_block(position) {
                        // If yes, draw the highlight
                        self.block_select.draw(
                            &self.colored_program,
                            &mut target,
                            aspect_ratio((width, height))
                                .matrix_mul(perspective(FOV))
                                .matrix_mul(camera_project)
                                .affine_translate(position.into())
                                .affine_translate([0.5; 3])
                                .affine_scale(1.001)
                                .affine_translate([-0.5; 3]),
                            (),
                        );
                        break;
                    }
                }
            }
        }
        self.cursor
            .draw(&self.colored_program, &mut target, Affine::identity(), ());
        target.finish().unwrap();
    }

    fn update(&mut self, control: &Control, display: &Display) {
        // Fetch player data because it is shared by multiple threads
        let mut player = self.world.pull_player();
        let camera = player.camera;
        let speed = if player.fly {
            1.0
        } else if control.shift {
            0.15
        } else {
            0.075
        };

        // Given user input, player movement is determined
        let mut vector = [0.0; 3];
        if control.front {
            vector.vector_add_assign([0.0, 0.0, speed]);
        }
        if control.back {
            vector.vector_sub_assign([0.0, 0.0, speed]);
        }
        if control.left {
            vector.vector_add_assign([speed, 0.0, 0.0]);
        }
        if control.right {
            vector.vector_sub_assign([speed, 0.0, 0.0]);
        }
        if player.fly {
            if control.up {
                vector.vector_add_assign([0.0, speed, 0.0]);
            }
            if control.down {
                vector.vector_sub_assign([0.0, speed, 0.0]);
            }
        } else {
            if control.up && player.on_ground {
                player.gravity = def::constant::JUMP;
                player.on_ground = false;
            }

            vector.vector_add_assign([0.0, player.gravity, 0.0]);
            player.gravity += def::constant::GRAVITY;
        }

        let [vector] = camera.move_matrix().matrix_mul([vector]);

        let vector = if player.fly {
            // If player is flying, ignore collisions
            vector
        } else {
            // If player is walking, compute collisions
            let hit_box = Boxel::new([0.6, 1.8, 0.6], [0.3, 1.6, 0.3], camera.pos);
            // Because it is a voxel terrain, hit box overlapping only occurs on bases axis
            // Here tx, ty and tz are the time where a collision was found (from 0.0 to 1.0)
            let tx = self.world.find_collision_x(hit_box, vector);
            let ty = self.world.find_collision_y(hit_box, vector);
            let tz = self.world.find_collision_z(hit_box, vector);
            if ty < 1.0 {
                player.on_ground = true;
                player.gravity = 0.0;
            }
            // The last statement is returned from the block
            [
                vector.vector_x() * tx,
                vector.vector_y() * ty,
                vector.vector_z() * tz,
            ]
        };
        // Apply player movement
        player.camera.delta_pos(vector);
        // Update player data to all threads
        self.world.push_player(player);

        // Unload out of range chunks (fawer then 256 meters)
        self.rendered_chunk.retain(|&k, _| {
            let x = (player.camera.pos.vector_x().floor() as i32 >> 4) - k.x;
            let z = (player.camera.pos.vector_z().floor() as i32 >> 4) - k.z;
            x * x + z * z < 16 * 16 // Thank you Pythagoras ! Thank you bro :)
        });

        // Process incoming commands from other threads
        while let Ok(cmd) = self.receiver_cmd.try_recv() {
            match cmd {
                AristideCmd::RenderChunk(cc, true) => {
                    // The given chunk is in range for rendering (less then ? meters)
                    // The appropriate mesh has been generated and sent to the GPU
                    self.rendered_chunk
                        .insert(cc, self.chunk_loader.build_mesh(cc, &self.world, display));
                }
                AristideCmd::RenderChunk(cc, false) => {
                    // The given chunk is out of range for rendering (more then 256 meters)
                    // It's mesh is freed from GPU memory
                    self.rendered_chunk.remove(&cc);
                }
            }
        }
    }

    fn click_left(&mut self) {
        let camera = self.world.pull_player().camera;
        let [cx, cy, cz, _] = camera.matrix().vector_z();

        for position in RayTravel::new(camera.pos, [cx, cy, cz], 10.0) {
            if let Some((position, _direction)) = position {
                if let Some(Some(_)) = self.world.get_block(position) {
                    self.world
                        .sender_cmd
                        .try_send(Cmd::RemoveBlock(position))
                        .ok();
                    break;
                }
            }
        }
    }

    fn click_right(&mut self) {
        let player = self.world.pull_player();
        let camera = player.camera;
        let [cx, cy, cz, _] = camera.matrix().vector_z();

        for position in RayTravel::new(camera.pos, [cx, cy, cz], 10.0) {
            if let Some((position, direction)) = position {
                if let Some(Some(_)) = self.world.get_block(position) {
                    if let Some(position) = position.step(direction) {
                        self.world
                            .sender_cmd
                            .try_send(Cmd::PlaceBlock(position, player.block_placing))
                            .ok();
                    }
                    break;
                }
            }
        }
    }
}

pub fn aristide(receiver_chunk_mesh: Receiver<AristideCmd>, world: Arc<World>) {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new().with_maximized(true);
    let cb = ContextBuilder::new().with_depth_buffer(24);
    let display = Display::new(wb, cb, &event_loop).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    let mut control = Control::default();
    let mut renderer = Renderer::new(&display, world, receiver_chunk_mesh);

    event_loop.run(move |ev, _, control_flow| match ev {
        Event::NewEvents(start_cause) => match start_cause {
            StartCause::Init => {
                *control_flow = ControlFlow::WaitUntil(Instant::now() + FRAME_DURATION);
            }
            StartCause::ResumeTimeReached {
                requested_resume, ..
            } => {
                *control_flow = ControlFlow::WaitUntil(requested_resume + FRAME_DURATION);
                display.gl_window().window().request_redraw();
                renderer.update(&control, &display);
            }
            StartCause::WaitCancelled {
                requested_resume, ..
            } => {
                *control_flow = if let Some(requested_resume) = requested_resume {
                    ControlFlow::WaitUntil(requested_resume)
                } else {
                    ControlFlow::Wait
                }
            }
            StartCause::Poll => {}
        },
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        scancode,
                        state,
                        virtual_keycode,
                        ..
                    },
                ..
            } => {
                control.update(
                    scancode,
                    match state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    },
                );

                if let ElementState::Pressed = state {
                    if let Some(keycode) = virtual_keycode {
                        use VirtualKeyCode as Key;
                        let player = renderer.world.pull_player();
                        match keycode {
                            Key::F => {
                                renderer.world.player_fly(!player.fly);
                            }
                            Key::Key1 => {
                                renderer.world.player_set_block_placing(def::Block::Brick);
                            }
                            Key::Key2 => {
                                renderer.world.player_set_block_placing(def::Block::Sand);
                            }
                            Key::Key3 => {
                                renderer.world.player_set_block_placing(def::Block::Glass);
                            }
                            Key::Key4 => {
                                renderer.world.player_set_block_placing(def::Block::Trunk);
                            }
                            Key::Key5 => {
                                renderer.world.player_set_block_placing(def::Block::Grass);
                            }
                            Key::Key6 => {
                                renderer.world.player_set_block_placing(def::Block::Water);
                            }
                            _ => (),
                        }
                    }
                }
            }
            _ => {}
        },
        Event::RedrawRequested { .. } => renderer.render(display.draw()),
        Event::DeviceEvent { event, .. } => match event {
            DeviceEvent::Motion { axis, value } => {
                let mut player = renderer.world.pull_player();
                match axis {
                    0 => player.camera.delta_angle_h(value as f32 * 0.005),
                    1 => player.camera.delta_angle_v(-value as f32 * 0.005),
                    _ => {}
                }
                renderer.world.push_player(player);
            }
            DeviceEvent::Button {
                button: 1,
                state: ElementState::Pressed,
            } => {
                renderer.click_left();
            }
            DeviceEvent::Button {
                button: 3,
                state: ElementState::Pressed,
            } => {
                renderer.click_right();
            }
            _ => {}
        },
        _ => {}
    });
}

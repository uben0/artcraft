use def::{
    cube::{self, FACE_INDICES},
    ChunkCoords,
};
use glium::{index::PrimitiveType, Display};
use mat::VectorTrait;

use crate::{
    mesh::{TexturedMesh, TexturedMeshVertex},
    world::{ChunkState, World},
};

/// Allocated buffers used to build meshes
///
/// Keeping the buffers avoid reallocating new ones every time
pub struct ChunkLoader {
    vertices: Vec<TexturedMeshVertex>,
    indices: Vec<u32>,
}

impl ChunkLoader {
    pub fn new() -> Self {
        Self {
            vertices: Vec::with_capacity(1024),
            indices: Vec::with_capacity(1024),
        }
    }
    /// Build the mesh (vertices and triangles) of specified chunk
    pub fn build_mesh(
        &mut self,
        cc: ChunkCoords,
        world: &World,
        display: &Display,
    ) -> TexturedMesh {
        if let ChunkState::Meshed(ref _blocks_chunk, ref faces_chunk) =
            *world.chunks.get(&cc).unwrap()
        {
            for (&(bi, d), &block) in faces_chunk.iter() {
                // block pos
                let vector: [i32; 3] = bi.into();
                // new vertex's index (will be pushed at the end of the list)
                let indice = self.vertices.len() as u32;
                // iterate over all faces of a cube
                for (i, vertice) in d.face_vertices().into_iter().enumerate() {
                    // how texture is map on cube side
                    let [u, v] = cube::FACE_TEXTURE[i];
                    // create a new vertex (position and texture info and light info)
                    let vertex = TexturedMeshVertex {
                        position: vertice.vector_add(vector).map(|v| v as f32),
                        tex_pos: [u, v, block.sprite(d) as u32].map(|v| v as f32),
                        light: d.light(),
                    };
                    self.vertices.push(vertex);
                }
                // add the cube face (one side, with 4 vertices and 2 triangles)
                self.indices
                    .extend(FACE_INDICES.into_iter().map(|n| n + indice));
            }
            // the mesh is sent to the graphic card
            let result = TexturedMesh::new(
                display,
                &self.vertices,
                &self.indices,
                PrimitiveType::TrianglesList,
            );
            // clear the buffers for future use
            self.vertices.clear();
            self.indices.clear();
            result
        } else {
            unreachable!()
        }
    }
}

use glium::{
    implement_vertex, index::PrimitiveType, texture::SrgbTexture2dArray, uniform, Blend, DepthTest,
    Display, Frame, IndexBuffer, Program, Surface, VertexBuffer,
};

#[derive(Debug, Clone, Copy)]
pub struct ColoredMeshVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}
implement_vertex!(ColoredMeshVertex, position, color);

impl From<([f32; 3], [f32; 3])> for ColoredMeshVertex {
    fn from((position, color): ([f32; 3], [f32; 3])) -> Self {
        Self { position, color }
    }
}

pub struct ColoredMesh {
    vertices: VertexBuffer<ColoredMeshVertex>,
    indices: IndexBuffer<u32>,
    point_size: Option<f32>,
    line_width: Option<f32>,
    depth_test: DepthTest,
}

const COLORED_MESH_VERTEX_PROGRAM: &str = r#"
    #version 140

    in vec3 position;
    in vec3 color;

    out vec3 v_color;

    uniform mat4 projection;

    void main() {
        v_color = color;
        gl_Position = projection * vec4(position, 1.0);
    }
"#;

const COLORED_MESH_FRAGMENT_PROGRAM: &str = r#"
    #version 140

    in vec3 v_color;
    out vec4 color;

    void main() {
        color = vec4(v_color, 1.0);
    }
"#;

impl ColoredMesh {
    pub fn new(
        display: &Display,
        vertices: &[ColoredMeshVertex],
        indices: &[u32],
        primitive: PrimitiveType,
    ) -> Self {
        Self {
            vertices: VertexBuffer::new(display, &vertices).unwrap(),
            indices: IndexBuffer::new(display, primitive, indices).unwrap(),
            point_size: None,
            line_width: None,
            depth_test: DepthTest::IfLess,
        }
    }
    pub fn program(display: &Display) -> Program {
        Program::from_source(
            display,
            COLORED_MESH_VERTEX_PROGRAM,
            COLORED_MESH_FRAGMENT_PROGRAM,
            None,
        )
        .unwrap()
    }
    pub fn point_size(self, point_size: f32) -> Self {
        Self {
            point_size: Some(point_size),
            ..self
        }
    }
    pub fn line_width(self, line_width: f32) -> Self {
        Self {
            line_width: Some(line_width),
            ..self
        }
    }
    pub fn depth_test(self, depth_test: DepthTest) -> Self {
        Self { depth_test, ..self }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TexturedMeshVertex {
    pub position: [f32; 3],
    pub tex_pos: [f32; 3],
    pub light: f32,
}
implement_vertex!(TexturedMeshVertex, position, tex_pos, light);

pub struct TexturedMesh {
    vertices: VertexBuffer<TexturedMeshVertex>,
    indices: IndexBuffer<u32>,
    point_size: Option<f32>,
    line_width: Option<f32>,
    depth_test: DepthTest,
}

const TEXTURED_MESH_VERTEX_PROGRAM: &str = r#"
    #version 140

    in vec3 position;
    in vec3 tex_pos;
    in float light;

    out vec3 v_tex_pos;
    out float v_light;

    uniform mat4 projection;

    void main() {
        v_tex_pos = tex_pos;
        v_light = light;
        gl_Position = projection * vec4(position, 1.0);
    }
"#;

const TEXTURED_MESH_FRAGMENT_PROGRAM: &str = r#"
    #version 140

    in vec3 v_tex_pos;
    in float v_light;
    out vec4 color;

    uniform sampler2DArray textures;

    void main() {
        vec4 rgba = texture(textures, v_tex_pos);
    
        float rl = rgba.r * ((1.0 * v_light) * 0.8 + (0.4) * 0.2);
        float gl = rgba.g * ((0.6 * v_light) * 0.8 + (0.8) * 0.2);
        float bl = rgba.b * ((0.3 * v_light) * 0.8 + (1.0) * 0.2);

        float rd = 1.0 - (1.0 - rl) * (1.0 - v_light);
        float gd = 1.0 - (1.0 - gl) * (1.0 - v_light);
        float bd = 1.0 - (1.0 - bl) * (1.0 - v_light);
    
        float rf = 0.7 * rl + 0.3 * rd;
        float gf = 0.8 * gl + 0.2 * gd;
        float bf = 0.9 * bl + 0.1 * bd;

        color = vec4(
            rf,
            gf,
            bf,
            rgba.a
        );
    }
"#;

impl TexturedMesh {
    pub fn new(
        display: &Display,
        vertices: &[TexturedMeshVertex],
        indices: &[u32],
        primitive: PrimitiveType,
    ) -> Self {
        Self {
            vertices: VertexBuffer::new(display, &vertices).unwrap(),
            indices: IndexBuffer::new(display, primitive, indices).unwrap(),
            point_size: None,
            line_width: None,
            depth_test: DepthTest::IfLess,
        }
    }
    pub fn program(display: &Display) -> Program {
        Program::from_source(
            display,
            TEXTURED_MESH_VERTEX_PROGRAM,
            TEXTURED_MESH_FRAGMENT_PROGRAM,
            None,
        )
        .unwrap()
    }
    // pub fn point_size(self, point_size: f32) -> Self {
    //     Self {point_size: Some(point_size), .. self }
    // }
    // pub fn line_width(self, line_width: f32) -> Self {
    //     Self {line_width: Some(line_width), .. self }
    // }
    // pub fn depth_test(self, depth_test: DepthTest) -> Self {
    //     Self {depth_test, .. self}
    // }
}

pub trait Drawable<T> {
    fn draw(&self, program: &Program, target: &mut Frame, projection: [[f32; 4]; 4], uniform: T);
}

impl Drawable<()> for ColoredMesh {
    fn draw(&self, program: &Program, target: &mut Frame, projection: [[f32; 4]; 4], _uniform: ()) {
        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: self.depth_test,
                write: true,
                ..Default::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            // polygon_mode: PolygonMode::Line,
            point_size: self.point_size,
            line_width: self.line_width,
            ..Default::default()
        };
        target
            .draw(
                &self.vertices,
                &self.indices,
                program,
                &uniform! {
                    projection: projection,
                },
                &params,
            )
            .unwrap();
    }
}

impl Drawable<&SrgbTexture2dArray> for TexturedMesh {
    fn draw(
        &self,
        program: &Program,
        target: &mut Frame,
        projection: [[f32; 4]; 4],
        uniform: &SrgbTexture2dArray,
    ) {
        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: self.depth_test,
                write: true,
                ..Default::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            // polygon_mode: PolygonMode::Line,
            point_size: self.point_size,
            line_width: self.line_width,
            blend: Blend::alpha_blending(),
            ..Default::default()
        };
        target
            .draw(
                &self.vertices,
                &self.indices,
                program,
                &uniform! {
                    projection: projection,
                    textures: uniform,
                },
                &params,
            )
            .unwrap();
    }
}

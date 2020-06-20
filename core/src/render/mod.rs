use luminance_derive::{Semantics, Vertex, UniformInterface};
use luminance::{
    tess::{Tess, TessBuilder, Mode},
    context::GraphicsContext,
    shader::program::{
        Program,
        Uniform,
        Uniformable
    },
    linear::M44,
};

use glow::HasContext;

pub mod camera;

#[derive(UniformInterface)]
pub struct ShaderInterface {
    #[uniform(name = "inv_projview_matrix")]
    pub inv_projection_view: Uniform<M44>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Semantics)]
pub enum VertexSemantics {
    #[sem(name = "v_pos", repr = "[f32; 3]", wrapper = "VertexPosition")]
    Position,
    #[sem(name = "v_uv", repr = "[f32; 2]", wrapper = "VertexUV")]
    UV,
}

#[derive(Vertex, Copy, Clone)]
#[vertex(sem = "VertexSemantics")]
pub struct Vertex {
    pub position: VertexPosition,
    pub uv: VertexUV,
}

pub type VertexIndex = u32;

pub fn initialize(gl: &glow::Context) {
    unsafe {
        gl.enable(glow::TEXTURE_3D);
        // gl.active_texture(glow::TEXTURE0);
    }
}

pub fn get_3d_texture(gl: &glow::Context, w: i32, h: i32, d: i32) -> <glow::Context as glow::HasContext>::Texture {
    // let random_data: [u8; 64*64*64*4] = [1; 64*64*64*4];
    let random_data: Vec<u8> = vec![0; 64*64*64*4];

    unsafe {
        let gl_texture = gl.create_texture().expect("Failed to create texture!");
        // let mut gl_texture = 0;
        // gl::GenTextures(1, &mut gl_texture);
        // gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_3D, gl_texture);

        gl.tex_image_3d(glow::TEXTURE_3D, 0, glow::RGBA32F as i32, w, h, d, 0, glow::RGBA, glow::UNSIGNED_BYTE, Some(&random_data));

        gl.tex_parameter_i32(glow::TEXTURE_3D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
        gl.tex_parameter_i32(glow::TEXTURE_3D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
        // gl.tex_parameter_i32(glow::TEXTURE_3D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
        // gl.tex_parameter_i32(glow::TEXTURE_3D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
        // gl.tex_parameter_i32(glow::TEXTURE_3D, glow::TEXTURE_WRAP_R, glow::REPEAT as i32);
        gl.tex_parameter_i32(glow::TEXTURE_3D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
        gl.tex_parameter_i32(glow::TEXTURE_3D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
        gl.tex_parameter_i32(glow::TEXTURE_3D, glow::TEXTURE_WRAP_R, glow::CLAMP_TO_EDGE as i32);

        gl::BindImageTexture(0, gl_texture, 0, gl::TRUE, 0, gl::READ_WRITE, gl::RGBA32F);

        gl_texture
    }
}

pub fn get_workgroup_count(gl: &glow::Context) -> (i32, i32, i32) {
    unsafe {(
        gl.get_parameter_indexed_i32(glow::MAX_COMPUTE_WORK_GROUP_COUNT, 0),
        gl.get_parameter_indexed_i32(glow::MAX_COMPUTE_WORK_GROUP_COUNT, 1),
        gl.get_parameter_indexed_i32(glow::MAX_COMPUTE_WORK_GROUP_COUNT, 2)
    )}
}

pub fn get_workgroup_size(gl: &glow::Context) -> (i32, i32, i32) {
    unsafe {(
        gl.get_parameter_indexed_i32(glow::MAX_COMPUTE_WORK_GROUP_SIZE, 0),
        gl.get_parameter_indexed_i32(glow::MAX_COMPUTE_WORK_GROUP_SIZE, 1),
        gl.get_parameter_indexed_i32(glow::MAX_COMPUTE_WORK_GROUP_SIZE, 2)
    )}
}

pub fn get_workgroup_invocations(gl: &glow::Context) -> i32 {
    unsafe {
        gl.get_parameter_i32(glow::MAX_COMPUTE_WORK_GROUP_INVOCATIONS)
    }
}

pub fn get_compute_program(gl: &glow::Context, cs: &str) -> <glow::Context as glow::HasContext>::Program {
    unsafe {
        let shader = match gl.create_shader(glow::COMPUTE_SHADER) {
            Ok(shader) => shader,
            Err(err) => {
                error!("{}", err);
                panic!("Failed to compile compute shader!");
            }
        };
        gl.shader_source(shader, cs);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            error!("{}", gl.get_shader_info_log(shader));
            panic!("Failed to compile compute shader!");
        }

        let program = gl.create_program().expect("Failed to create compute shader program!");
        gl.attach_shader(program, shader);
        gl.link_program(program);

        if !gl.get_program_link_status(program) {
            error!("{}", gl.get_program_info_log(program));
            panic!("Failed to compile compute program!");
        }

        gl.detach_shader(program, shader);
        gl.delete_shader(shader);

        program
    }
}

pub fn get_program(vs: &str, fs: &str) -> Program<VertexSemantics, (), ShaderInterface> {
    let program: Program<VertexSemantics, (), ShaderInterface> = match Program::from_strings(None, vs, None, fs) {
            Ok(program) => program.ignore_warnings(),
            Err(err) => {
                error!("{}", err);
                panic!("Failed to compile shaders!");
            }
        };

    program
}

pub fn get_screen_rect<C>(ctx: &mut C) -> Tess
where
    C: GraphicsContext,
{
    let verts: [Vertex; 4] = [
        Vertex {
            position: VertexPosition::new([-1.0, -1.0, 0.0]),
            uv: VertexUV::new([0.0, 1.0]),
        },
        Vertex {
            position: VertexPosition::new([1.0, -1.0, 0.0]),
            uv: VertexUV::new([1.0, 1.0]),
        },
        Vertex {
            position: VertexPosition::new([1.0, 1.0, 0.0]),
            uv: VertexUV::new([1.0, 0.0]),
        },
        Vertex {
            position: VertexPosition::new([-1.0, 1.0, 0.0]),
            uv: VertexUV::new([0.0, 0.0]),
        }
    ];

    let tess = TessBuilder::new(ctx).set_mode(Mode::TriangleFan).add_vertices(verts).build().expect("Failed to build mesh!");

    tess
}

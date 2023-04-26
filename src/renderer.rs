#![allow(dead_code)]
#![allow(unused_variables)]

use crate::world::World;
use glow::HasContext;
use std::fs;
use std::mem::size_of;

const MAX_N_INSTANCED_PRIMITIVES: usize = 1 << 12;
const COMMON_GLSL_SHADER_FP: &str = "./assets/shaders/common.glsl";
const PRIMITIVE_VERT_SHADER_FP: &str = "./assets/shaders/primitive.vert";
const PRIMITIVE_FRAG_SHADER_FP: &str = "./assets/shaders/primitive.frag";
const SCREEN_RECT_VERT_SHADER_FP: &str =
    "./assets/shaders/screen_rect.vert";
const HDR_RESOLVE_FRAG_SHADER_FP: &str =
    "./assets/shaders/hdr_resolve.frag";

pub struct Renderer {
    window: sdl2::video::Window,
    gl: glow::Context,
    _gl_context: sdl2::video::GLContext,

    primitive_renderer: PrimitiveRenderer,
    hdr_resolve_renderer: HDRResolveRenderer,
}

impl Renderer {
    pub fn create(
        sdl: &sdl2::Sdl,
        window_name: &str,
        window_width: u32,
        window_height: u32,
    ) -> Self {
        let video = sdl.video().unwrap();

        let window = video
            .window(window_name, window_width, window_height)
            .opengl()
            .resizable()
            .build()
            .unwrap();

        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(4, 6);

        let _gl_context = window.gl_create_context().unwrap();
        let gl: glow::Context;
        unsafe {
            gl = glow::Context::from_loader_function(|s| {
                video.gl_get_proc_address(s) as *const _
            });
        }

        video.gl_set_swap_interval(1).unwrap();

        let primitive_renderer = PrimitiveRenderer::create(&gl);
        let hdr_resolve_renderer =
            HDRResolveRenderer::create(&gl, window_width, window_height);

        Self {
            window: window,
            gl: gl,
            _gl_context: _gl_context,
            primitive_renderer: primitive_renderer,
            hdr_resolve_renderer: hdr_resolve_renderer,
        }
    }

    pub fn render(&mut self, world: &World) {
        let camera = &world.camera;
        let lift = &world.lift;

        let primitives = [
            DrawPrimitive {
                xywh: [0.5, 0.5, 0.2, 0.2],
                rgba: [1.0, 0.0, 0.0, 1.0],
                orientation: 1.0,
            },
            DrawPrimitive {
                xywh: [-0.5, -0.5, 0.1, 0.1],
                rgba: [0.0, 0.0, 1.0, 1.0],
                orientation: 0.0,
            },
        ];

        self.hdr_resolve_renderer.bind_framebuffer(&self.gl);

        unsafe {
            self.gl.clear_color(0.1, 0.05, 0.3, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }

        self.primitive_renderer.render(&self.gl, &primitives);

        self.bind_screen_framebuffer();
        self.hdr_resolve_renderer.render(&self.gl);

        self.window.gl_swap_window();
    }

    fn bind_screen_framebuffer(&self) {
        let (width, height) = self.window.size();
        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }
}

struct DrawPrimitive {
    pub xywh: [f32; 4],
    pub rgba: [f32; 4],
    pub orientation: f32,
}

struct PrimitiveRenderer {
    program: glow::NativeProgram,

    vao: glow::NativeVertexArray,
    a_xywh: Attribute,
    a_rgba: Attribute,
    a_orientation: Attribute,
}

impl PrimitiveRenderer {
    pub fn create(gl: &glow::Context) -> Self {
        let program = create_program(
            gl,
            Some(COMMON_GLSL_SHADER_FP),
            PRIMITIVE_VERT_SHADER_FP,
            PRIMITIVE_FRAG_SHADER_FP,
        );

        let vao = create_vao(gl);
        unsafe {
            gl.bind_vertex_array(Some(vao));
        }

        let a_xywh = Attribute::create(gl, 4, "a_xywh", glow::FLOAT, 1);
        let a_rgba = Attribute::create(gl, 4, "a_rgba", glow::FLOAT, 1);
        let a_orientation =
            Attribute::create(gl, 1, "a_orientation", glow::FLOAT, 1);

        Self {
            program,
            vao,
            a_xywh,
            a_rgba,
            a_orientation,
        }
    }

    pub fn render(
        &mut self,
        gl: &glow::Context,
        primitives: &[DrawPrimitive],
    ) {

        self.a_xywh.clear();
        self.a_rgba.clear();
        self.a_orientation.clear();

        for primitive in primitives {
            self.a_xywh.data.extend(&primitive.xywh);
            self.a_rgba.data.extend(&primitive.rgba);
            self.a_orientation.data.push(primitive.orientation);
        }

        unsafe {
            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(self.vao));
        }

        self.a_xywh.set(gl, self.program);
        self.a_rgba.set(gl, self.program);
        self.a_orientation.set(gl, self.program);

        unsafe {
            gl.draw_arrays_instanced(
                glow::TRIANGLE_STRIP,
                0,
                4,
                primitives.len() as i32,
            );
        }
    }
}

struct HDRResolveRenderer {
    program: glow::NativeProgram,
    fbo: glow::NativeFramebuffer,
    tex: glow::Texture,
    width: i32,
    height: i32,
}

impl HDRResolveRenderer {
    pub fn create(gl: &glow::Context, width: u32, height: u32) -> Self {
        let program = create_program(
            gl,
            Some(COMMON_GLSL_SHADER_FP),
            SCREEN_RECT_VERT_SHADER_FP,
            HDR_RESOLVE_FRAG_SHADER_FP,
        );

        let fbo;
        let tex;
        let width = width as i32;
        let height = height as i32;
        unsafe {
            fbo = gl.create_framebuffer().unwrap();
            tex = create_texture(
                gl,
                glow::RGBA32F as i32,
                width,
                height,
                glow::RGBA,
                glow::FLOAT,
                None,
                glow::NEAREST,
            );
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(tex),
                0,
            );
            gl.draw_buffer(glow::COLOR_ATTACHMENT0);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }

        Self {
            program,
            fbo,
            tex,
            width,
            height,
        }
    }

    pub fn bind_framebuffer(&self, gl: &glow::Context) {
        unsafe {
            gl.viewport(0, 0, self.width, self.height);
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo));
        }
    }

    pub fn render(&self, gl: &glow::Context) {
        unsafe {
            gl.use_program(Some(self.program));
            set_uniform_1_i32(gl, self.program, "tex", 0);

            gl.active_texture(glow::TEXTURE0 + 0);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.tex));

            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        }
    }
}

pub struct Attribute {
    pub size: usize,
    pub name: &'static str,
    pub data_type: u32,
    pub divisor: u32,
    pub data: Vec<f32>,
    pub vbo: glow::NativeBuffer,
}

impl Attribute {
    pub fn create(
        gl: &glow::Context,
        size: usize,
        name: &'static str,
        data_type: u32,
        divisor: u32,
    ) -> Self {
        let vbo_size = MAX_N_INSTANCED_PRIMITIVES * size_of::<f32>() * size;
        let data = Vec::<f32>::with_capacity(MAX_N_INSTANCED_PRIMITIVES);
        let vbo = create_vbo(gl, vbo_size, glow::DYNAMIC_DRAW);

        Self {
            size,
            name,
            data_type,
            divisor,
            data,
            vbo,
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn set(&self, gl: &glow::Context, program: glow::NativeProgram) {
        unsafe {
            let loc = match gl.get_attrib_location(program, &self.name) {
                Some(loc) => loc,
                None => panic!("Can't obtain attribute location: {}", self.name),
            };

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.enable_vertex_attrib_array(loc);

            match self.data_type {
                glow::FLOAT => {
                    gl.vertex_attrib_pointer_f32(
                        loc,
                        self.size as i32,
                        self.data_type,
                        false,
                        0,
                        0,
                    );
                }
                glow::INT | glow::UNSIGNED_INT => {
                    gl.vertex_attrib_pointer_i32(
                        loc,
                        self.size as i32,
                        self.data_type,
                        0,
                        0,
                    );
                }
                _ => {
                    panic!(
                        "Unsopported vertex attrib data type: {}",
                        self.data_type
                    );
                }
            }

            gl.vertex_attrib_divisor(loc, self.divisor);
            gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                0,
                cast_slice_to_u8(&self.data),
            );
        }
    }
}

fn create_vbo(
    gl: &glow::Context,
    size: usize,
    usage: u32,
) -> glow::NativeBuffer {
    let vbo;

    unsafe {
        vbo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_size(glow::ARRAY_BUFFER, size as i32, usage);
    }

    vbo
}

fn create_vao(gl: &glow::Context) -> glow::NativeVertexArray {
    let vao;

    unsafe {
        vao = gl.create_vertex_array().unwrap();
    }

    vao
}

fn create_program(
    gl: &glow::Context,
    common_shader_fp: Option<&str>,
    vert_shader_fp: &str,
    frag_shader_fp: &str,
) -> glow::NativeProgram {
    let program;

    unsafe {
        program = gl.create_program().expect("Cannot create program");

        let common_shader_src: String;
        if let Some(common_shader_fp) = common_shader_fp {
            common_shader_src =
                fs::read_to_string(common_shader_fp).unwrap();
        } else {
            common_shader_src = "".to_string();
        }

        let mut vert_shader_src =
            fs::read_to_string(vert_shader_fp).unwrap();
        vert_shader_src = common_shader_src.clone() + &vert_shader_src;

        let mut frag_shader_src =
            fs::read_to_string(frag_shader_fp).unwrap();
        frag_shader_src = common_shader_src.clone() + &frag_shader_src;

        let shaders_src = [
            (glow::VERTEX_SHADER, vert_shader_src),
            (glow::FRAGMENT_SHADER, frag_shader_src),
        ];

        let mut shaders = Vec::with_capacity(shaders_src.len());

        for (shader_type, shader_src) in shaders_src.iter() {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(shader, shader_src);
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!("{}", gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }

        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("{}", gl.get_program_info_log(program));
        }

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }
    }

    program
}

fn create_texture(
    gl: &glow::Context,
    internal_format: i32,
    width: i32,
    height: i32,
    format: u32,
    ty: u32,
    pixels: Option<&[u8]>,
    filter: u32,
) -> glow::Texture {
    let tex;

    unsafe {
        tex = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(tex));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            internal_format,
            width,
            height,
            0,
            format,
            ty,
            pixels,
        );

        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            filter as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            filter as i32,
        );
    }

    tex
}

fn set_uniform_1_f32(
    gl: &glow::Context,
    program: glow::NativeProgram,
    name: &str,
    value: f32,
) {
    unsafe {
        let loc = gl.get_uniform_location(program, name);
        gl.uniform_1_f32(loc.as_ref(), value)
    }
}

fn set_uniform_1_i32(
    gl: &glow::Context,
    program: glow::NativeProgram,
    name: &str,
    value: i32,
) {
    unsafe {
        let loc = gl.get_uniform_location(program, name);
        gl.uniform_1_i32(loc.as_ref(), value)
    }
}

fn set_uniform_2_f32(
    gl: &glow::Context,
    program: glow::NativeProgram,
    name: &str,
    value: &[f32],
) {
    unsafe {
        let loc = gl.get_uniform_location(program, name);
        gl.uniform_2_f32_slice(loc.as_ref(), value)
    }
}

fn set_uniform_4_f32(
    gl: &glow::Context,
    program: glow::NativeProgram,
    name: &str,
    value: &[f32],
) {
    unsafe {
        let loc = gl.get_uniform_location(program, name);
        gl.uniform_4_f32_slice(loc.as_ref(), value)
    }
}

fn cast_slice_to_u8<T>(slice: &[T]) -> &[u8] {
    let casted: &[u8];
    unsafe {
        casted = core::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            slice.len() * core::mem::size_of::<T>(),
        );
    }

    casted
}

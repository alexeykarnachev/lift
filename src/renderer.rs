#![allow(dead_code)]
#![allow(unused_variables)]

use crate::entity::Light;
use crate::vec::*;
use core::fmt::Debug;
use enum_iterator::{all, Sequence};
use image::imageops::flip_vertical_in_place;
use image::io::Reader as ImageReader;
// use crate::world::*;
use glow::HasContext;
use std::fs;
use std::mem::size_of;

const MAX_N_INSTANCED_PRIMITIVES: usize = 1 << 12;
const MAX_N_LIGHTS: usize = 32;
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

    // Primitive renderer
    primitive_program: glow::NativeProgram,
    primitive_vao: glow::NativeVertexArray,
    a_xywh: Attribute<f32>,
    a_space: Attribute<u32>,
    a_effect: Attribute<u32>,
    a_tex_uvwh: Attribute<f32>,
    a_rgba: Attribute<f32>,
    a_tex_id: Attribute<u32>,
    a_flip: Attribute<f32>,

    // HDR resolve renderer
    hdr_resolve_program: glow::NativeProgram,
    hdr_buffer_size: Vec2<u32>,
    hdr_fbo: glow::NativeFramebuffer,
    hdr_tex: glow::Texture,

    // Resource textures
    sprite_atlas_tex: glow::Texture,
    // glyph_atlas_tex: Option<glow::Texture>,

    // World
    camera_position: Vec2<f32>,
    camera_view_size: Vec2<f32>,
    primitives: Vec<DrawPrimitive>,
    lights: Vec<Light>,
}

impl Renderer {
    pub fn new(
        sdl: &sdl2::Sdl,
        window_name: &str,
        window_size: Vec2<u32>,
        sprite_atlas_image_fp: &str,
    ) -> Self {
        // ---------------------------------------------------------------
        // Initialize gl and window
        let video = sdl.video().unwrap();

        let window = video
            .window(window_name, window_size.x, window_size.y)
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

        // ---------------------------------------------------------------
        // Initialize primitive renderer
        let primitive_program = create_program(
            &gl,
            Some(COMMON_GLSL_SHADER_FP),
            PRIMITIVE_VERT_SHADER_FP,
            PRIMITIVE_FRAG_SHADER_FP,
        );
        let primitive_vao = create_vao(&gl);
        unsafe {
            gl.bind_vertex_array(Some(primitive_vao));
        }

        let a_xywh = Attribute::new(
            &gl,
            primitive_program,
            4,
            "a_xywh",
            glow::FLOAT,
            MAX_N_INSTANCED_PRIMITIVES,
            1,
        );
        let a_space = Attribute::new(
            &gl,
            primitive_program,
            1,
            "a_space",
            glow::UNSIGNED_INT,
            MAX_N_INSTANCED_PRIMITIVES,
            1,
        );
        let a_effect = Attribute::new(
            &gl,
            primitive_program,
            1,
            "a_effect",
            glow::UNSIGNED_INT,
            MAX_N_INSTANCED_PRIMITIVES,
            1,
        );
        let a_tex_uvwh = Attribute::new(
            &gl,
            primitive_program,
            4,
            "a_tex_uvwh",
            glow::FLOAT,
            MAX_N_INSTANCED_PRIMITIVES,
            1,
        );
        let a_rgba = Attribute::new(
            &gl,
            primitive_program,
            4,
            "a_rgba",
            glow::FLOAT,
            MAX_N_INSTANCED_PRIMITIVES,
            1,
        );
        let a_tex_id = Attribute::new(
            &gl,
            primitive_program,
            1,
            "a_tex_id",
            glow::UNSIGNED_INT,
            MAX_N_INSTANCED_PRIMITIVES,
            1,
        );
        let a_flip = Attribute::new(
            &gl,
            primitive_program,
            1,
            "a_flip",
            glow::FLOAT,
            MAX_N_INSTANCED_PRIMITIVES,
            1,
        );

        // ---------------------------------------------------------------
        // Initialize HDR resolve renderer
        let hdr_buffer_size = window_size;
        let hdr_resolve_program = create_program(
            &gl,
            Some(COMMON_GLSL_SHADER_FP),
            SCREEN_RECT_VERT_SHADER_FP,
            HDR_RESOLVE_FRAG_SHADER_FP,
        );
        let hdr_tex;
        let hdr_fbo;
        unsafe {
            hdr_fbo = gl.create_framebuffer().unwrap();
            hdr_tex = create_texture(
                &gl,
                glow::RGBA32F as i32,
                hdr_buffer_size.x as i32,
                hdr_buffer_size.y as i32,
                glow::RGBA,
                glow::FLOAT,
                None,
                glow::NEAREST,
            );
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(hdr_fbo));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(hdr_tex),
                0,
            );
            gl.draw_buffer(glow::COLOR_ATTACHMENT0);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }

        // ---------------------------------------------------------------
        // Initialize texture (sprites and glyphs)
        let mut sprite_atlas_image =
            ImageReader::open(sprite_atlas_image_fp)
                .unwrap()
                .decode()
                .unwrap();
        flip_vertical_in_place(&mut sprite_atlas_image);

        let sprite_atlas_tex = create_texture(
            &gl,
            glow::RGBA as i32,
            sprite_atlas_image.width() as i32,
            sprite_atlas_image.height() as i32,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            Some(&sprite_atlas_image.as_bytes().to_vec()),
            glow::LINEAR,
        );

        Self {
            window,
            gl,
            _gl_context,
            primitive_program,
            primitive_vao,
            a_xywh,
            a_space,
            a_effect,
            a_tex_uvwh,
            a_rgba,
            a_tex_id,
            a_flip,
            hdr_resolve_program,
            hdr_buffer_size,
            hdr_fbo,
            hdr_tex,
            sprite_atlas_tex,
            camera_position: Vec2::zeros(),
            camera_view_size: Vec2::zeros(),
            primitives: Vec::with_capacity(MAX_N_INSTANCED_PRIMITIVES),
            lights: Vec::with_capacity(MAX_N_LIGHTS),
        }
    }

    pub fn clear_queue(&mut self) {
        self.primitives.clear();
        self.lights.clear();
    }

    pub fn push_primitive(&mut self, primitive: DrawPrimitive) {
        self.primitives.push(primitive);
    }

    pub fn push_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    pub fn set_camera(
        &mut self,
        camera_position: Vec2<f32>,
        camera_view_size: Vec2<f32>,
    ) {
        self.camera_position = camera_position;
        self.camera_view_size = camera_view_size;
    }

    pub fn render(&mut self) {
        let screen_size =
            [self.window.size().0 as f32, self.window.size().1 as f32];
        let camera_xywh = [
            self.camera_position.to_array(),
            self.camera_view_size.to_array(),
        ]
        .concat();
        // Sort draw primitives by their z-value
        self.primitives
            .sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap());

        // Fill up draw primitives related shader vao attributes
        for primitive in self.primitives.iter() {
            self.a_xywh.push_data(&primitive.rect.to_xywh());
            self.a_space.push_data(&[primitive.space as u32]);
            self.a_effect.push_data(&[primitive.effect]);
            self.a_flip.push_data(&[(primitive.flip as i32) as f32]);
            self.a_tex_id.push_data(&[primitive.tex as u32]);
            self.a_tex_uvwh.push_data(&primitive.xywh);
            self.a_rgba.push_data(&primitive.rgba);
        }

        // Render primitives
        unsafe {
            self.gl.bind_vertex_array(Some(self.primitive_vao));
            self.a_xywh.sync_data(&self.gl);
            self.a_space.sync_data(&self.gl);
            self.a_effect.sync_data(&self.gl);
            self.a_rgba.sync_data(&self.gl);
            self.a_tex_uvwh.sync_data(&self.gl);
            self.a_tex_id.sync_data(&self.gl);
            self.a_flip.sync_data(&self.gl);

            self.gl.use_program(Some(self.primitive_program));
            set_uniform_2_f32(
                &self.gl,
                self.primitive_program,
                "screen_size",
                &screen_size,
            );
            set_uniform_4_f32(
                &self.gl,
                self.primitive_program,
                "camera_xywh",
                &camera_xywh,
            );

            set_uniform_1_i32(
                &self.gl,
                self.primitive_program,
                "sprite_atlas_tex",
                0,
            );
            self.gl.active_texture(glow::TEXTURE0 + 0);
            self.gl.bind_texture(
                glow::TEXTURE_2D,
                Some(self.sprite_atlas_tex),
            );

            // if let Some(&self.glyph_atlas_tex) = self.&self.glyph_atlas_tex {
            //     set_uniform_1_i32(&self.gl, self.program, "&self.glyph_atlas_tex", 1);
            //     &self.gl.active_texture(glow::TEXTURE0 + 1);
            //     &self.gl.bind_texture(glow::TEXTURE_2D, Some(&self.glyph_atlas_tex));
            // }

            set_uniform_1_i32(
                &self.gl,
                self.primitive_program,
                "n_lights",
                self.lights.len() as i32,
            );
            for (i, light) in self.lights.iter().enumerate() {
                let name = format!("lights[{}]", i).clone();
                set_uniform_2_f32(
                    &self.gl,
                    self.primitive_program,
                    &format!("{}.{}", name, "position"),
                    &light.position.to_array(),
                );
                set_uniform_3_f32(
                    &self.gl,
                    self.primitive_program,
                    &format!("{}.{}", name, "color"),
                    &light.get_color().to_rgb_array(),
                );
                set_uniform_3_f32(
                    &self.gl,
                    self.primitive_program,
                    &format!("{}.{}", name, "attenuation"),
                    &light.attenuation,
                );
            }

            self.gl.enable(glow::BLEND);
            self.gl
                .blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            self.gl.viewport(
                0,
                0,
                self.hdr_buffer_size.x as i32,
                self.hdr_buffer_size.y as i32,
            );
            self.gl
                .bind_framebuffer(glow::FRAMEBUFFER, Some(self.hdr_fbo));
            self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            self.gl.draw_arrays_instanced(
                glow::TRIANGLE_STRIP,
                0,
                4,
                self.primitives.len() as i32,
            );

            // Resolve hdr buffer
            self.gl.use_program(Some(self.hdr_resolve_program));
            set_uniform_1_i32(
                &self.gl,
                self.hdr_resolve_program,
                "tex",
                0,
            );

            self.gl.active_texture(glow::TEXTURE0 + 0);
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.hdr_tex));

            self.gl.viewport(
                0,
                0,
                screen_size[0] as i32,
                screen_size[1] as i32,
            );
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            self.gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        }

        self.window.gl_swap_window();
    }

    /*
    pub fn load_resources(&mut self, world: &World) {
        if self.primitive_renderer.glyph_atlas_tex.is_none() {
            let tex = create_texture(
                &self.gl,
                glow::RED as i32,
                world.glyph_atlas.size[0] as i32,
                world.glyph_atlas.size[1] as i32,
                glow::RED,
                glow::UNSIGNED_BYTE,
                Some(&world.glyph_atlas.image),
                glow::LINEAR,
            );
            self.primitive_renderer.glyph_atlas_tex = Some(tex);
        }
    }

    pub fn fill_primitives(&mut self, world: &World) {
        self.primitives.clear();

        draw_level(&world.level, &mut self.primitives);

        for light in world.level.lights.iter() {
            draw_entity(light, &mut self.primitives);
        }

        world.level.enemies.iter().for_each(|enemy| {
            draw_entity(enemy, &mut self.primitives);
        });
        draw_entity(&world.level.player, &mut self.primitives);

        // world.attacks.iter().for_each(|attack| {
        //     draw_attack(attack, &mut self.primitives);
        // });
        // world.level.enemies.iter().for_each(|enemy| {
        //     draw_collider(
        //         enemy.get_collider().unwrap(),
        //         &mut self.primitives,
        //     );
        // });
        // draw_collider(&world.player, &mut self.primitives);

        world.gui.draw(&mut self.primitives);
    }

    pub fn fill_lights(&mut self, world: &World) {
        self.lights.clear();

        if let Some(light) = world.level.player.get_light() {
            self.lights.push(light);
        }

        for light in
            world.level.enemies.iter().filter_map(|e| e.get_light())
        {
            self.lights.push(light);
        }

        for entity in world.level.lights.iter() {
            let light = entity.get_light().unwrap();
            self.lights.push(light);
        }
    }
    */
}

pub struct Attribute<T> {
    pub data: Vec<T>,
    pub vbo: glow::NativeBuffer,
}

impl<T> Attribute<T>
where
    for<'a> &'a [T]: IntoIterator<Item = &'a T>,
    T: Clone,
{
    pub fn new(
        gl: &glow::Context,
        program: glow::NativeProgram,
        size: usize,
        name: &'static str,
        data_type: u32,
        max_n_instances: usize,
        divisor: u32,
    ) -> Self {
        let max_n_elements = max_n_instances * size;
        let vbo_size = size_of::<T>() * max_n_elements;
        let data = Vec::<T>::with_capacity(max_n_elements);
        let vbo = create_vbo(gl, vbo_size, glow::DYNAMIC_DRAW);

        unsafe {
            let loc = match gl.get_attrib_location(program, name) {
                Some(loc) => loc,
                None => {
                    panic!("Can't obtain attribute location: {}", name)
                }
            };

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.enable_vertex_attrib_array(loc);

            match data_type {
                glow::FLOAT => {
                    gl.vertex_attrib_pointer_f32(
                        loc,
                        size as i32,
                        data_type,
                        false,
                        0,
                        0,
                    );
                }
                glow::INT | glow::UNSIGNED_INT => {
                    gl.vertex_attrib_pointer_i32(
                        loc,
                        size as i32,
                        data_type,
                        0,
                        0,
                    );
                }
                _ => {
                    panic!(
                        "Unsopported vertex attrib data type: {}",
                        data_type
                    );
                }
            }

            gl.vertex_attrib_divisor(loc, divisor);
        }

        Self { data, vbo }
    }

    pub fn push_data(&mut self, data: &[T]) {
        self.data.extend_from_slice(data);
    }

    pub fn sync_data(&mut self, gl: &glow::Context) {
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                0,
                cast_slice_to_u8(&self.data),
            );
        }
        self.data.clear();
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

        let mut common_shader_src: String;
        if let Some(common_shader_fp) = common_shader_fp {
            common_shader_src =
                fs::read_to_string(common_shader_fp).unwrap();
        } else {
            common_shader_src = "".to_string();
        }

        common_shader_src.push_str(&enum_to_shader_source::<SpaceType>());
        common_shader_src
            .push_str(&enum_to_shader_source::<TextureType>());
        common_shader_src.push_str(&enum_to_shader_source::<EffectType>());

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

fn set_uniform_3_f32(
    gl: &glow::Context,
    program: glow::NativeProgram,
    name: &str,
    value: &[f32],
) {
    unsafe {
        let loc = gl.get_uniform_location(program, name);
        gl.uniform_3_f32_slice(loc.as_ref(), value)
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

pub struct DrawPrimitive {
    pub z: f32,
    pub rect: Rect,
    pub space: SpaceType,
    pub tex: TextureType,
    pub xywh: [f32; 4],
    pub rgba: [f32; 4],
    pub effect: u32,
    pub flip: bool,
}

impl DrawPrimitive {
    pub fn world_sprite(
        xywh: [f32; 4],
        pivot: Pivot,
        apply_light: bool,
        flip: bool,
    ) -> Self {
        let size = Vec2::new(xywh[2], xywh[3]);
        let rect = Rect::from_pivot(pivot, size);
        let effect = if apply_light {
            EffectType::ApplyLightEffect as u32
        } else {
            0
        };

        Self {
            z: 0.0,
            rect,
            space: SpaceType::WorldSpace,
            tex: TextureType::SpriteTexture,
            xywh,
            rgba: [0.0, 0.0, 0.0, 1.0],
            effect: 0,
            flip: false,
        }
    }
}

#[derive(Copy, Clone, Debug, Sequence)]
pub enum SpaceType {
    WorldSpace = 1,
    CameraSpace = 2,
    ScreenSpace = 3,
}
impl From<SpaceType> for u32 {
    fn from(e: SpaceType) -> u32 {
        e as u32
    }
}

#[derive(Copy, Clone, Debug, Sequence)]
pub enum TextureType {
    ProceduralTexture = 1,
    SpriteTexture = 2,
    GlyphTexture = 3,
}
impl From<TextureType> for u32 {
    fn from(e: TextureType) -> u32 {
        e as u32
    }
}

#[derive(Copy, Clone, Debug, Sequence)]
pub enum EffectType {
    ApplyLightEffect = 1 << 0,
    StoneWallEffect = 1 << 1,
}
impl From<EffectType> for u32 {
    fn from(e: EffectType) -> u32 {
        e as u32
    }
}

pub fn enum_to_shader_source<T: Sequence + Debug + Copy + Into<u32>>(
) -> String {
    let mut source = String::new();

    for variant in all::<T>().collect::<Vec<_>>() {
        let definition =
            format!("const uint {:?} = {:?};\n", variant, variant.into());
        source.push_str(&definition);
    }

    source
}

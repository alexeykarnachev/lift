#![allow(dead_code)]
#![allow(unused_variables)]

use crate::graphics::*;
use crate::vec::Vec2;
use crate::world::*;
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

    primitive_renderer: PrimitiveRenderer,
    hdr_resolve_renderer: HDRResolveRenderer,

    primitives: Vec<DrawPrimitive>,
    lights: Vec<Light>,
}

impl Renderer {
    pub fn new(
        sdl: &sdl2::Sdl,
        window_name: &str,
        window_size: Vec2<u32>,
    ) -> Self {
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

        let primitive_renderer = PrimitiveRenderer::new(&gl);
        let hdr_resolve_renderer =
            HDRResolveRenderer::new(&gl, window_size);

        Self {
            window,
            gl,
            _gl_context,
            primitive_renderer,
            hdr_resolve_renderer,
            primitives: Vec::with_capacity(MAX_N_INSTANCED_PRIMITIVES),
            lights: Vec::with_capacity(MAX_N_LIGHTS),
        }
    }

    pub fn render(&mut self, world: &World) {
        self.load_resources(world);
        self.fill_primitives(world);
        self.fill_lights(world);

        self.hdr_resolve_renderer.bind_framebuffer(&self.gl);

        unsafe {
            self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        let screen_size =
            [self.window.size().0 as f32, self.window.size().1 as f32];
        self.primitive_renderer.render(
            &self.gl,
            &world.camera,
            &screen_size,
            &self.lights,
            &self.primitives,
        );

        self.bind_screen_framebuffer();
        self.hdr_resolve_renderer.render(&self.gl);

        self.window.gl_swap_window();
    }

    pub fn load_resources(&mut self, world: &World) {
        if self.primitive_renderer.sprite_atlas_tex.is_none() {
            let tex = create_texture(
                &self.gl,
                glow::RGBA as i32,
                world.sprite_atlas.size[0] as i32,
                world.sprite_atlas.size[1] as i32,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(&world.sprite_atlas.image),
                glow::LINEAR,
            );
            self.primitive_renderer.sprite_atlas_tex = Some(tex);
        }

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

        world.attacks.iter().for_each(|attack| {
            draw_attack(attack, &mut self.primitives);
        });
        // world.level.enemies.iter().for_each(|enemy| {
        //     draw_collider(
        //         enemy.get_collider().unwrap(),
        //         &mut self.primitives,
        //     );
        // });
        // draw_collider(&world.player, &mut self.primitives);

        use WorldState::*;
        match world.state {
            MainMenu => {
                draw_ui(&world.main_menu_ui, &mut self.primitives);
            }
            Play => {
                draw_ui(&world.play_ui, &mut self.primitives);
            }
            GameOver => {
                draw_ui(&world.game_over_ui, &mut self.primitives);
            }
            _ => {}
        }

        self.primitives
            .sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap());
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

    fn bind_screen_framebuffer(&self) {
        let (width, height) = self.window.size();
        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }
}

struct PrimitiveRenderer {
    program: glow::NativeProgram,

    sprite_atlas_tex: Option<glow::Texture>,
    glyph_atlas_tex: Option<glow::Texture>,

    vao: glow::NativeVertexArray,
    a_xywh: Attribute<f32>,
    a_space: Attribute<u32>,
    a_effect: Attribute<u32>,
    a_tex_uvwh: Attribute<f32>,
    a_rgba: Attribute<f32>,
    a_tex_id: Attribute<u32>,
    a_flip: Attribute<f32>,
}

impl PrimitiveRenderer {
    pub fn new(gl: &glow::Context) -> Self {
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

        let a_xywh = Attribute::new(
            gl,
            program,
            4,
            "a_xywh",
            glow::FLOAT,
            MAX_N_INSTANCED_PRIMITIVES,
            1,
        );
        let a_space = Attribute::new(
            gl,
            program,
            1,
            "a_space",
            glow::UNSIGNED_INT,
            MAX_N_INSTANCED_PRIMITIVES,
            1,
        );
        let a_effect = Attribute::new(
            gl,
            program,
            1,
            "a_effect",
            glow::UNSIGNED_INT,
            MAX_N_INSTANCED_PRIMITIVES,
            1,
        );
        let a_tex_uvwh = Attribute::new(
            gl,
            program,
            4,
            "a_tex_uvwh",
            glow::FLOAT,
            MAX_N_INSTANCED_PRIMITIVES,
            1,
        );
        let a_rgba = Attribute::new(
            gl,
            program,
            4,
            "a_rgba",
            glow::FLOAT,
            MAX_N_INSTANCED_PRIMITIVES,
            1,
        );
        let a_tex_id = Attribute::new(
            gl,
            program,
            1,
            "a_tex_id",
            glow::UNSIGNED_INT,
            MAX_N_INSTANCED_PRIMITIVES,
            1,
        );
        let a_flip = Attribute::new(
            gl,
            program,
            1,
            "a_flip",
            glow::FLOAT,
            MAX_N_INSTANCED_PRIMITIVES,
            1,
        );

        Self {
            program,
            sprite_atlas_tex: None,
            glyph_atlas_tex: None,
            vao,
            a_xywh,
            a_space,
            a_effect,
            a_tex_uvwh,
            a_rgba,
            a_tex_id,
            a_flip,
        }
    }

    pub fn render(
        &mut self,
        gl: &glow::Context,
        camera: &Camera,
        screen_size: &[f32; 2],
        lights: &Vec<Light>,
        primitives: &Vec<DrawPrimitive>,
    ) {
        primitives.iter().for_each(|p| self.push_primitive(p));

        unsafe {
            gl.use_program(Some(self.program));
            set_uniform_2_f32(
                gl,
                self.program,
                "screen_size",
                screen_size,
            );

            if let Some(sprite_atlas_tex) = self.sprite_atlas_tex {
                set_uniform_1_i32(gl, self.program, "sprite_atlas_tex", 0);
                gl.active_texture(glow::TEXTURE0 + 0);
                gl.bind_texture(glow::TEXTURE_2D, Some(sprite_atlas_tex));
            }

            if let Some(glyph_atlas_tex) = self.glyph_atlas_tex {
                set_uniform_1_i32(gl, self.program, "glyph_atlas_tex", 1);
                gl.active_texture(glow::TEXTURE0 + 1);
                gl.bind_texture(glow::TEXTURE_2D, Some(glyph_atlas_tex));
            }

            set_uniform_lights(gl, self.program, lights)
        }

        self.sync_data(gl);

        set_uniform_camera(gl, self.program, camera);
        unsafe {
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.draw_arrays_instanced(
                glow::TRIANGLE_STRIP,
                0,
                4,
                primitives.len() as i32,
            );
        }
    }

    fn push_primitive(&mut self, primitive: &DrawPrimitive) {
        self.a_xywh.push_data(&primitive.rect.to_xywh());
        self.a_space.push_data(&[primitive.space as u32]);
        self.a_effect.push_data(&[primitive.effect]);
        self.a_flip.push_data(&[(primitive.flip as i32) as f32]);

        if let Some(tex) = primitive.tex {
            self.a_tex_id.push_data(&[tex as u32]);
        } else {
            self.a_tex_id.push_data(&[0]);
        }

        if let Some(sprite) = &primitive.sprite {
            self.a_tex_uvwh.push_data(&sprite.to_tex_xywh());
        } else {
            self.a_tex_uvwh.push_data(&[0.0; 4]);
        }

        if let Some(color) = &primitive.color {
            self.a_rgba.push_data(&color.to_array());
        } else {
            self.a_rgba.push_data(&[0.0; 4]);
        }
    }

    fn sync_data(&mut self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
        }
        self.a_xywh.sync_data(gl);
        self.a_space.sync_data(gl);
        self.a_effect.sync_data(gl);
        self.a_rgba.sync_data(gl);
        self.a_tex_uvwh.sync_data(gl);
        self.a_tex_id.sync_data(gl);
        self.a_flip.sync_data(gl);
    }
}

struct HDRResolveRenderer {
    program: glow::NativeProgram,
    fbo: glow::NativeFramebuffer,
    tex: glow::Texture,
    buffer_size: Vec2<u32>,
}

impl HDRResolveRenderer {
    pub fn new(gl: &glow::Context, buffer_size: Vec2<u32>) -> Self {
        let program = create_program(
            gl,
            Some(COMMON_GLSL_SHADER_FP),
            SCREEN_RECT_VERT_SHADER_FP,
            HDR_RESOLVE_FRAG_SHADER_FP,
        );

        let fbo;
        let tex;
        let width = buffer_size.x as i32;
        let height = buffer_size.y as i32;
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
            buffer_size,
        }
    }

    pub fn bind_framebuffer(&self, gl: &glow::Context) {
        unsafe {
            gl.viewport(
                0,
                0,
                self.buffer_size.x as i32,
                self.buffer_size.y as i32,
            );
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

fn set_uniform_camera(
    gl: &glow::Context,
    program: glow::NativeProgram,
    camera: &Camera,
) {
    let view_size = camera.get_view_size();
    let world_xywh =
        [camera.position.to_array(), view_size.to_array()].concat();

    set_uniform_4_f32(gl, program, "camera.world_xywh", &world_xywh);
}

fn set_uniform_lights(
    gl: &glow::Context,
    program: glow::NativeProgram,
    lights: &[Light],
) {
    set_uniform_1_i32(gl, program, "n_lights", lights.len() as i32);

    for (i, light) in lights.iter().enumerate() {
        let name = format!("lights[{}]", i).clone();
        set_uniform_2_f32(
            gl,
            program,
            &format!("{}.{}", name, "position"),
            &light.position.to_array(),
        );
        set_uniform_3_f32(
            gl,
            program,
            &format!("{}.{}", name, "color"),
            &light.color.to_rgb_array(),
        );
        set_uniform_3_f32(
            gl,
            program,
            &format!("{}.{}", name, "attenuation"),
            &light.attenuation,
        );
    }
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

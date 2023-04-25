use glow::HasContext;
use std::fs;

pub struct SDLGL {
    pub gl: glow::Context,
    pub sdl: sdl2::Sdl,
    pub window: sdl2::video::Window,
    _gl_context: sdl2::video::GLContext,
}

impl SDLGL {
    pub fn create(
        window_name: &str,
        window_width: u32,
        window_height: u32,
    ) -> Self {
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();

        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(4, 6);

        let window = video
            .window(window_name, window_width, window_height)
            .opengl()
            .resizable()
            .build()
            .unwrap();

        let _gl_context = window.gl_create_context().unwrap();
        let gl: glow::Context;
        unsafe {
            gl = glow::Context::from_loader_function(|s| {
                video.gl_get_proc_address(s) as *const _
            });
        }

        Self {
            gl: gl,
            sdl: sdl,
            window: window,
            _gl_context: _gl_context,
        }
    }
}

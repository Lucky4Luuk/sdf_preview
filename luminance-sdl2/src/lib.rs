use std::convert::TryInto;
use luminance::texture::Dim2;
use luminance::texture::Flat;
use luminance::framebuffer::Framebuffer;
use gl;
use luminance::context::GraphicsContext;
use luminance::state::GraphicsState;
pub use luminance::state::StateQueryError;
pub use luminance_windowing::{CursorMode, Surface, WindowDim, WindowOpt};
use std::cell::RefCell;
use std::fmt;
use std::os::raw::c_void;
use std::rc::Rc;
use std::ffi::CString;
use std::ops::DerefMut;

#[macro_use] extern crate log;

//Error that can be risen while creating a surface
#[derive(Debug)]
pub enum SDL2SurfaceError {
    GraphicsStateError(StateQueryError)
}

impl fmt::Display for SDL2SurfaceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            SDL2SurfaceError::GraphicsStateError(sqe) => write!(f, "Failed to get graphics state: {}", sqe)
        }
    }
}

pub struct SDL2Surface {
    pub sdl: sdl2::Sdl,
    pub video: sdl2::VideoSubsystem,
    pub window: sdl2::video::Window,
    pub gfx_state: Rc<RefCell<GraphicsState>>,
}

unsafe impl GraphicsContext for SDL2Surface {
    fn state(&self) -> &Rc<RefCell<GraphicsState>> {
        &self.gfx_state
    }
}

//TODO: Implement better error checking using error enum above (e.g. notify user that sdl2.dll is missing)
//TODO: Perhaps implement wrappers for a bunch of window-related functions?
impl SDL2Surface {
    pub fn new(gl_version: (u8, u8), window_title: &str, window_size: (u32, u32), vsync: bool) -> Result<Self, SDL2SurfaceError> {
        debug!("Initializing sdl...");
        let sdl = sdl2::init().expect("Failed to load SDL!");
        let video = sdl.video().expect("Failed to load video subsystem!");
        debug!("sdl initialized!");

        debug!("Opening a window...");
        let window = video.window(window_title, window_size.0, window_size.1)
                        .position_centered()
                        .opengl() //TODO: Set opengl version somehow
                        .build()
                        .map_err(|e| e.to_string()).expect("Failed to open window!");
        debug!("Window opened!");

        {
            let gl_attr = video.gl_attr();
            gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
            gl_attr.set_context_version(gl_version.0, gl_version.1);
            if vsync {
                gl_attr.set_double_buffer(true);
            } else {
                gl_attr.set_double_buffer(false);
            }
        }

        let _gl_context = window.gl_create_context().expect("Failed to create GL context!");
        gl::load_with(|s| video.gl_get_proc_address(s) as *const c_void);

        let swap_interval = if vsync {
            sdl2::video::SwapInterval::VSync
        } else {
            sdl2::video::SwapInterval::Immediate
        };

        video.gl_set_swap_interval(swap_interval).expect("Failed to set swap interval!");

        let gfx_state = GraphicsState::new().map_err(SDL2SurfaceError::GraphicsStateError)?;
        let surface = Self {
            sdl: sdl,
            video: video,
            window: window,
            gfx_state: Rc::new(RefCell::new(gfx_state)),
        };

        Ok(surface)
    }

    pub fn size(&self) -> (u32, u32) {
        self.window.drawable_size()
    }

    pub fn size_array(&self) -> [u32; 2] {
        let size = self.size();
        [size.0.try_into().expect("Failed to turn size into i32"), size.1.try_into().expect("Failed to turn size into i32")]
    }

    pub fn width(&self) -> u32 {
        self.window.drawable_size().0
    }

    pub fn height(&self) -> u32 {
        self.window.drawable_size().1
    }

    pub fn back_buffer(&mut self) -> Result<Framebuffer<Flat, Dim2, (), ()>, SDL2SurfaceError> {
        Ok(Framebuffer::back_buffer(self, self.size_array()))
    }

    pub fn swap_buffer(&self) {
        self.window.gl_swap_window();
    }

    pub fn set_title(&mut self, title: &str) {
        self.window.set_title(title); //TODO: Return the error?
    }
}

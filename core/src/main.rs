#[macro_use] extern crate log;
#[macro_use] extern crate imgui;

use std::os::raw::c_void;
use std::time::Instant;
use std::ops::Deref;

use glow::HasContext;

use cgmath::*;

use sdl2::{
    event::Event,
};

use luminance::{
    context::GraphicsContext,
    pipeline::PipelineState,
    render_state::RenderState,
    tess::TessSliceIndex,
};

mod render;

fn main() {
    pretty_env_logger::formatted_builder()
        .filter(None, log::LevelFilter::max())
        .init();

    debug!("Hello, world!");

    let (mut surface, gl, _gl_context) = open_window(1280, 720).expect("Failed to open window!");

    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);

    let mut imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &surface.window);

    gl::load_with(|s| surface.video.gl_get_proc_address(s) as _);

    let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| surface.video.gl_get_proc_address(s) as *const c_void);
    let mut camera = render::camera::Camera::default();
    camera.position.y = 0.5;

    let mut event_pump = surface.sdl.event_pump().expect("Failed to get event pump!");

    let mut last_frame = Instant::now();
    let mut delta_s = 0.0;

    render::initialize(&gl);

    let screen_rect = render::get_screen_rect(&mut surface);
    let program = render::get_program(include_str!("vertex.glsl"), include_str!("fragment.glsl"));
    let render_state = RenderState::default();

    let scene_tex = render::get_3d_texture(&gl, 64, 64, 64);
    let depth_shader = render::get_compute_program(&gl, include_str!("compute.glsl"));

    debug!("Setup complete!");

    let work_group_count = render::get_workgroup_count(&gl);
    debug!("Max global work group counts: [x: {}; y: {}; z: {}]", work_group_count.0, work_group_count.1, work_group_count.2);
    let work_group_size = render::get_workgroup_size(&gl);
    debug!("Max local work group size: [x: {}; y: {}; z: {}]", work_group_size.0, work_group_size.1, work_group_size.2);
    let work_group_invoc = render::get_workgroup_invocations(&gl);
    debug!("Max local work group invocations: {}", work_group_invoc);

    unsafe {
        // gl::BindImageTexture(0, scene_tex, 0, gl::TRUE, 0, gl::WRITE_ONLY, gl::RGBA32F);
        gl.use_program(Some(depth_shader));
        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_3D, Some(scene_tex));
        gl.uniform_1_i32(gl.get_uniform_location(depth_shader, "img_output"), 0);
        gl.dispatch_compute(64, 64, 64);
        gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
        // gl.use_program(None);
        // gl.bind_texture(glow::TEXTURE_3D, None);
    }

    'main: loop {
        let back_buffer = surface.back_buffer().expect("Couldn't get the back buffer!");

        //beep

        for event in event_pump.poll_iter() {
            imgui_sdl2.handle_event(&mut imgui, &event);
            if imgui_sdl2.ignore_event(&event) { continue; }

            match event {
                Event::Quit { .. } => {
                    debug!("Bye!");
                    break 'main;
                },
                _ => {}
            }
        }

        unsafe {
            gl.clear_color(127.0 / 255.0, 103.0 / 255.0, 181.0 / 255.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        //Rendering
        let inv_projview_matrix = (camera.get_proj(1280, 720) * camera.get_view()).invert().expect("Failed to invert projection view matrix!");

        surface.pipeline_builder().pipeline(
            &back_buffer,
            &PipelineState::default(),
            |_, mut shd_gate| {
                shd_gate.shade(&program, |iface, mut rdr_gate| {
                    let handle = program.deref();
                    unsafe {
                        gl.use_program(Some(handle.handle()));

                        // gl.uniform_1_i32(gl.get_uniform_location(handle.handle(), "depth_tex"), 0);

                        // gl::BindImageTexture(0, scene_tex, 0, gl::TRUE, 0, gl::READ_WRITE, gl::RGBA32F);
                        gl.active_texture(glow::TEXTURE0);
                        gl.bind_texture(glow::TEXTURE_3D, Some(scene_tex));
                    }

                    iface.inv_projection_view.update(inv_projview_matrix.into());

                    rdr_gate.render(&render_state, |mut tess_gate| {
                        tess_gate.render(screen_rect.slice(..))
                    })
                })
            }
        );

        //End of loop
        imgui_sdl2.prepare_frame(imgui.io_mut(), &surface.window, &event_pump.mouse_state());
        let now = Instant::now();
        let delta = now - last_frame;
        delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        last_frame = now;
        imgui.io_mut().delta_time = delta_s;

        //UI
        let ui = imgui.frame();

        let stats_window = imgui::Window::new(im_str!("Metrics"))
            .position([10.0, 10.0], imgui::Condition::Appearing)
            .size([120.0, 120.0], imgui::Condition::Appearing)
            .focused(false)
            .collapsible(true);

        stats_window.build(&ui, || {
            ui.text(format!("FPS: {:.1}", 1000.0 / (delta_s * 1000.0)));
            ui.text(format!("MS: {:.2}", delta_s * 1000.0));
        });

        imgui_sdl2.prepare_render(&ui, &surface.window);
        renderer.render(ui);

        surface.swap_buffer();
    }
}

fn open_window(width: u32, height: u32) -> Result<(luminance_sdl2::SDL2Surface, glow::Context, sdl2::video::GLContext), &'static str> {
    let surface = luminance_sdl2::SDL2Surface::new(
        (4, 5), //Opengl version
        "SDF Modelling",
        (width, height),
        false //VSync
    );

    match surface {
        Err(e) => {
            error!("Couldn't initialize photic!\n{}", e);
            return Err("Couldn't initialize photic!")
        },
        Ok(surface) => {
            let gl_context = surface.window.gl_create_context().expect("Couldn't create GL context");
            let gl = glow::Context::from_loader_function(|s| {
                    surface.video.gl_get_proc_address(s) as *const c_void
                });
            debug!("Photic initialized!");
            return Ok((surface, gl, gl_context));
        }
    }
}

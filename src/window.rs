use chrono::Timelike;
use std::{sync::Arc, time::SystemTime};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::gpu::{GpuState, Renderer};

pub struct App<'a> {
    window: Option<Arc<Window>>,
    state: Option<GpuState<'a>>,
    default_fragment_code: &'a str,
    renderer: Option<Renderer>,
    // viewport size
    viewport_size: [f32; 2],
    // time
    time_from_start_up: std::time::Instant,
    time_from_update: std::time::Instant,
    // update time
    updated_time: Option<String>,
}

impl<'a> App<'a> {
    pub fn new(default_fragment_code: &'a str) -> Self {
        Self {
            window: None,
            state: None,
            default_fragment_code,
            renderer: None,
            viewport_size: [0.0, 0.0],
            time_from_start_up: std::time::Instant::now(),
            time_from_update: std::time::Instant::now(),
            updated_time: None,
        }
    }
}

impl App<'_> {
    pub fn render(&mut self) {
        let surface_texture = self.state.as_ref().unwrap().get_current_texture();
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let multi_sample_texture = self.state.as_ref().unwrap().get_multisample_texture();
        let multi_sample_view =
            multi_sample_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let timer = std::time::Instant::now();

        self.renderer.as_ref().unwrap().render(
            self.state.as_ref().unwrap().get_device(),
            self.state.as_ref().unwrap().get_queue(),
            &surface_view,
            &multi_sample_view,
            crate::gpu::renderer::ViewportInfo {
                size: self.viewport_size,
                time_from_start_up: self.time_from_start_up.elapsed().as_secs_f32(),
                time_from_update: self.time_from_update.elapsed().as_secs_f32(),
            },
        );

        let render_time = timer.elapsed().as_micros();

        // print!("\r(updated: {:?})Render time: {:>6}μs", self.updated_time, render_time);

        if let Some(updated_time) = self.updated_time.as_deref() {
            print!(
                "\r(updated: {}) Render time:{:>5}μs",
                updated_time, render_time
            );
        } else {
            print!("\rRender time:{:>5}μs", render_time);
        }
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        surface_texture.present();
    }
}

impl ApplicationHandler<(Option<SystemTime>, String)> for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        ));

        // rename the window
        self.window.as_ref().unwrap().set_title("Live wgsl");

        // self.window.as_ref().unwrap().set_decorations(false);

        // make gpu state
        self.state = Some(pollster::block_on(GpuState::new(
            self.window.as_ref().unwrap().clone(),
        )));

        // prepare renderer
        self.renderer = Some(Renderer::new(
            self.state.as_ref().unwrap().get_device(),
            self.state.as_ref().unwrap().get_queue(),
            self.state.as_ref().unwrap().get_surface_format(),
            self.default_fragment_code,
        ));

        // get the viewport size
        self.viewport_size = self.window.as_ref().unwrap().inner_size().into();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let mut redraw = false;
        match event {
            WindowEvent::CloseRequested => {
                println!();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                redraw = true;
            }
            WindowEvent::Resized(new_size) => {
                self.state.as_mut().unwrap().resize(new_size);
                self.viewport_size = new_size.into();
                redraw = true;
            }
            _ => {}
        }

        if redraw {
            self.render();
        }
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        if cause == winit::event::StartCause::Poll {
            self.render();
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: (Option<SystemTime>, String)) {
        // event is the new fragment code

        // update the fragment code and pipeline

        let (update_time, fragment_code) = event;
        self.updated_time =
            update_time.map(|update_time| format_utc_to_string(&update_time.into()));

        if let Err(e) = pollster::block_on(
            self.renderer
                .as_mut()
                .unwrap()
                .update_fragment(&fragment_code, self.state.as_ref().unwrap().get_device()),
        ) {
            eprintln!("Error:\n{}", e);
            return;
        }

        // try to render

        let surface_texture = self.state.as_ref().unwrap().get_current_texture();
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let multi_sample_texture = self.state.as_ref().unwrap().get_multisample_texture();
        let multi_sample_view =
            multi_sample_texture.create_view(&wgpu::TextureViewDescriptor::default());

        if let Err(e) = pollster::block_on(self.renderer.as_mut().unwrap().try_render(
            self.state.as_ref().unwrap().get_device(),
            self.state.as_ref().unwrap().get_queue(),
            &surface_view,
            &multi_sample_view,
            crate::gpu::renderer::ViewportInfo {
                size: self.viewport_size,
                time_from_start_up: self.time_from_start_up.elapsed().as_secs_f32(),
                time_from_update: self.time_from_update.elapsed().as_secs_f32(),
            },
        )) {
            eprintln!("Error:\n{}", e);
            return;
        }

        surface_texture.present();
        self.time_from_update = std::time::Instant::now();
    }
}

fn format_utc_to_string(utc_time: &chrono::DateTime<chrono::Local>) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        utc_time.hour(),
        utc_time.minute(),
        utc_time.second(),
    )
}

use std::{sync::Arc, time::Instant};

use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

use crate::{
    config::{self, AppConfig},
    game::Game,
};

pub struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    game: Game,
    config: AppConfig,
    last_time: Instant,
}

impl App {
    pub fn new(game: Game) -> Self {
        Self {
            game,
            window: None,
            pixels: None,
            config: Default::default(),
            last_time: Instant::now(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.listen_device_events(winit::event_loop::DeviceEvents::Always);

        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("Mi aplicación")
                        .with_inner_size(LogicalSize::new(1600.0, 1200.0)),
                )
                .unwrap(),
        );

        let size = window.inner_size();

        let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());

        let pixels = Pixels::new(size.width, size.height, surface_texture).unwrap();

        self.pixels = Some(pixels);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if let Some(pixels) = self.pixels.as_mut() {
                    pixels.resize_surface(size.width, size.height).unwrap();
                }
            }

            WindowEvent::RedrawRequested => {
                let Some(pixels) = self.pixels.as_mut() else {
                    return;
                };

                let texture = pixels.texture();
                let width = texture.width();
                let height = texture.height();
                let frame = pixels.frame_mut();

                for chunk in frame.chunks_exact_mut(4) {
                    chunk[0] = 0x00;
                    chunk[1] = 0x00;
                    chunk[2] = 0x00;
                    chunk[3] = 0xFF;
                }

                let delta_time = self.last_time.elapsed().as_secs_f64().min(0.1);
                self.last_time = Instant::now();
                self.game.update(delta_time);

                self.game
                    .player_vision(width as usize, height as usize, frame);

                pixels.render().unwrap();
            }

            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = matches!(event.state, ElementState::Pressed);
                self.game.get_keyboard_input(event.physical_key, pressed);
            }

            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let winit::event::DeviceEvent::MouseMotion { delta } = event {
            let (dx, dy) = delta;

            self.game.handle_mouse_look(dx, dy);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

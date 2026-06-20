use winit::dpi::PhysicalSize;
use winit::keyboard::KeyCode;

pub struct AppConfig {
    pub title: &'static str,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub resizable: bool,
    pub vsync: bool,
    pub fov: f32,
    pub clear_color: [f32; 3],
    pub window_size: PhysicalSize<u32>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let width = 1280;
        let height = 720;

        Self {
            title: "Doom-style Raycaster",
            width,
            height,
            fullscreen: false,
            resizable: false,
            vsync: true,
            fov: 60.0,
            clear_color: [0.0, 0.0, 0.0],
            window_size: PhysicalSize::new(width, height),
        }
    }
}

pub struct GameConfig {
    pub render_distance: f32,
    pub player_speed: f32,
    pub mouse_sensitivity: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            render_distance: 20.0,
            player_speed: 5.0,
            mouse_sensitivity: 0.15,
        }
    }
}

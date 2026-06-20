use winit::keyboard::{KeyCode, PhysicalKey};

use crate::config::GameConfig;
use std::ops::{Add, Mul, MulAssign};
const TEX_WIDTH: usize = 64;
const TEX_HEIGHT: usize = 64;
const NUM_TEXTURES: usize = 8;

pub struct ControlMap {
    pub forward: KeyCode,
    pub backward: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub turn_left: KeyCode,
    pub turn_right: KeyCode,
    pub sprint: KeyCode,
}

impl Default for ControlMap {
    fn default() -> Self {
        Self {
            forward: KeyCode::KeyW,
            backward: KeyCode::KeyS,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
            turn_left: KeyCode::ArrowLeft,
            turn_right: KeyCode::ArrowRight,
            sprint: KeyCode::ShiftLeft,
        }
    }
}

pub(crate) enum ActionKind {
    Forward,
    Backward,
    Left,
    Right,
    TurnLeft,
    TurnRight,
    Sprint,
}

impl ControlMap {
    pub fn action(&self, key: KeyCode) -> Option<ActionKind> {
        match key {
            k if k == self.forward => Some(ActionKind::Forward),
            k if k == self.backward => Some(ActionKind::Backward),
            k if k == self.left => Some(ActionKind::Left),
            k if k == self.right => Some(ActionKind::Right),
            k if k == self.turn_left => Some(ActionKind::TurnLeft),
            k if k == self.turn_right => Some(ActionKind::TurnRight),
            k if k == self.sprint => Some(ActionKind::Sprint),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Action {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub turn_left: bool,
    pub turn_right: bool,
    pub sprint: bool,
}

impl Action {
    pub fn new() -> Self {
        Self {
            forward: false,
            backward: false,
            left: false,
            right: false,
            turn_left: false,
            turn_right: false,
            sprint: false,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Vector2D {
    pub x: f64,
    pub y: f64,
}

impl Vector2D {
    fn rotate(&self, degrees: f64) -> Self {
        let degrees = degrees.to_radians();

        Self {
            x: self.x * degrees.cos() - self.y * degrees.sin(),
            y: self.x * degrees.sin() + self.y * degrees.cos(),
        }
    }

    fn distance(&self, other: &Self) -> f64 {
        f64::sqrt((self.x - other.x).powi(2) + (self.y - other.y).powi(2))
    }

    fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    fn normalize(&mut self) {
        let aux = self.magnitude() * *self;

        self.x = aux.x;
        self.y = aux.y;
    }

    fn dot(&self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }
}

impl Mul<Vector2D> for f64 {
    type Output = Vector2D;

    fn mul(self, vector: Vector2D) -> Self::Output {
        Self::Output {
            x: vector.x * self,
            y: vector.y * self,
        }
    }
}

impl Add for Vector2D {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

struct HitRecord {
    hit_pos: Vector2D,
    distance: f64,
    map_pos: (usize, usize),
    normal: Vector2D,
}

pub struct Player {
    pub position: Vector2D,
    pub direction: Vector2D,
    pub speed: f64,
    pub fov: f64,
    pub action_player: Action,
    pub mouse_look: f64,
}

impl Player {
    fn get_new_position(&self, direction: Vector2D, delta_time: f64) -> Vector2D {
        self.position + (self.speed * delta_time) * direction
    }

    fn change_direction(&mut self, dx: f64, delta_time: f64) {
        self.direction = self.direction.rotate(dx * delta_time);
        //self.direction.normalize();
    }
}

pub struct Game {
    player: Player,
    map: Vec<Vec<u8>>,
    control_map: ControlMap,
    textures: [[u32; TEX_WIDTH * TEX_HEIGHT]; NUM_TEXTURES],
}

impl Game {
    pub fn new(player: Player, map: Vec<Vec<u8>>) -> Self {
        Self {
            player,
            map,
            control_map: Default::default(),
            textures: get_textures(),
        }
    }

    pub fn player_vision(&mut self, width: usize, height: usize, frame: &mut [u8]) {
        let player = &self.player;

        let start_direction = player.direction.rotate(-player.fov / 2.0);

        let step = player.fov / width as f64;

        for x in 0..width {
            let mut direction = start_direction.rotate(step * x as f64);
            direction.normalize();

            if let Some(hit) = self.ray_cast(player.position, direction) {
                let wall_height = (height as f64 / hit.distance) as usize;

                let start_y = height.saturating_sub(wall_height) / 2;

                let end_y = (start_y + wall_height).min(height);

                let brightness = 0.5f64.max(hit.normal.dot(player.direction).abs());

                for y in start_y..end_y {
                    let index = (y * width + x) * 4;

                    let tex_y =
                        ((y - start_y) as f64 / wall_height as f64 * TEX_HEIGHT as f64) as usize;
                    let tex_x = if hit.normal.x != 0.0 {
                        (hit.hit_pos.y - hit.hit_pos.y.floor()) * TEX_WIDTH as f64
                    } else {
                        (hit.hit_pos.x - hit.hit_pos.x.floor()) * TEX_WIDTH as f64
                    } as usize;
                    let id_texture = self.map[hit.map_pos.1][hit.map_pos.0] - 1;
                    let color = self.textures[id_texture as usize][tex_y * TEX_WIDTH + tex_x];

                    let r = get_red(color);
                    let g = get_green(color);
                    let b = get_blue(color);

                    frame[index] = (r as f64 * brightness) as u8;
                    frame[index + 1] = (g as f64 * brightness) as u8;
                    frame[index + 2] = (b as f64 * brightness) as u8;
                    frame[index + 3] = 255;
                }
            }
        }
    }

    fn is_inside(&self, position: &Vector2D) -> bool {
        let height = self.map.len();
        let width = self.map[0].len();

        position.x >= 0.0
            && position.x < width as f64
            && position.y >= 0.0
            && position.y < height as f64
    }

    fn is_new_position_colliding(&self, new_position: &Vector2D) -> bool {
        self.map[new_position.y.floor() as usize][new_position.x.floor() as usize] != 0
    }

    fn ray_cast(&self, position: Vector2D, direction: Vector2D) -> Option<HitRecord> {
        let mut map_x = position.x.floor() as i32;
        let mut map_y = position.y.floor() as i32;

        let delta_dist_x = if direction.x == 0.0 {
            f64::INFINITY
        } else {
            (1.0 / direction.x).abs()
        };

        let delta_dist_y = if direction.y == 0.0 {
            f64::INFINITY
        } else {
            (1.0 / direction.y).abs()
        };

        let step_x;
        let step_y;

        let mut side_dist_x;
        let mut side_dist_y;

        if direction.x < 0.0 {
            step_x = -1;
            side_dist_x = (position.x - map_x as f64) * delta_dist_x;
        } else {
            step_x = 1;
            side_dist_x = ((map_x + 1) as f64 - position.x) * delta_dist_x;
        }

        if direction.y < 0.0 {
            step_y = -1;
            side_dist_y = (position.y - map_y as f64) * delta_dist_y;
        } else {
            step_y = 1;
            side_dist_y = ((map_y + 1) as f64 - position.y) * delta_dist_y;
        }

        loop {
            let (distance, normal) = if side_dist_x < side_dist_y {
                let dist = side_dist_x;
                side_dist_x += delta_dist_x;
                map_x += step_x;
                let normal = Vector2D {
                    x: -step_x as f64,
                    y: 0.0,
                };
                (dist, normal)
            } else {
                let dist = side_dist_y;
                side_dist_y += delta_dist_y;
                map_y += step_y;
                let normal = Vector2D {
                    x: 0.0,
                    y: -step_y as f64,
                };
                (dist, normal)
            };

            let hit_pos = position + distance * direction;

            if !self.is_inside(&hit_pos) {
                return None;
            }

            let x = map_x as usize;
            let y = map_y as usize;

            if self.map[y][x] != 0 {
                return Some(HitRecord {
                    hit_pos,
                    distance,
                    map_pos: (x, y),
                    normal,
                });
            }
        }
    }

    fn move_player(&mut self, direction: Vector2D, delta_time: f64) {
        let new_position = self.player.get_new_position(direction, delta_time);

        let move_x = Vector2D {
            x: new_position.x,
            y: self.player.position.y,
        };
        if self.is_inside(&move_x) && !self.is_new_position_colliding(&move_x) {
            self.player.position.x = new_position.x;
        }

        let move_y = Vector2D {
            x: self.player.position.x,
            y: new_position.y,
        };
        if self.is_inside(&move_y) && !self.is_new_position_colliding(&move_y) {
            self.player.position.y = new_position.y;
        }
    }

    pub fn get_keyboard_input(&mut self, physical_key: PhysicalKey, pressed: bool) {
        if let PhysicalKey::Code(key) = physical_key {
            if let Some(action) = self.control_map.action(key) {
                match action {
                    ActionKind::Forward => self.player.action_player.forward = pressed,
                    ActionKind::Backward => self.player.action_player.backward = pressed,
                    ActionKind::Left => self.player.action_player.left = pressed,
                    ActionKind::Right => self.player.action_player.right = pressed,
                    ActionKind::TurnLeft => self.player.action_player.turn_left = pressed,
                    ActionKind::TurnRight => self.player.action_player.turn_right = pressed,
                    ActionKind::Sprint => self.player.action_player.sprint = pressed,
                }
            }
        }
    }

    pub fn handle_mouse_look(&mut self, x: f64, _y: f64) {
        self.player.mouse_look += x;
    }

    pub fn update(&mut self, delta_time: f64) {
        if self.player.action_player.forward {
            self.move_player(self.player.direction, delta_time);
        }
        if self.player.action_player.backward {
            self.move_player(self.player.direction.rotate(180.0), delta_time);
        }
        if self.player.action_player.left {
            self.move_player(self.player.direction.rotate(-90.0), delta_time);
        }
        if self.player.action_player.right {
            self.move_player(self.player.direction.rotate(90.0), delta_time);
        }
        if self.player.action_player.turn_left {
            self.player.change_direction(-100.0, delta_time);
        }
        if self.player.action_player.turn_right {
            self.player.change_direction(100.0, delta_time);
        }

        if self.player.mouse_look != 0.0 {
            self.player.change_direction(self.player.mouse_look, 1.0);
            self.player.mouse_look = 0.0;
        }
    }
}

fn get_textures() -> [[u32; TEX_WIDTH * TEX_HEIGHT]; NUM_TEXTURES] {
    let mut textures = [[0u32; TEX_WIDTH * TEX_HEIGHT]; NUM_TEXTURES];

    for x in 0..TEX_WIDTH {
        for y in 0..TEX_HEIGHT {
            let xorcolor = ((x * 256 / TEX_WIDTH) ^ (y * 256 / TEX_HEIGHT)) as u32;

            let ycolor = (y * 256 / TEX_HEIGHT) as u32;

            let xycolor = (y * 128 / TEX_HEIGHT + x * 128 / TEX_WIDTH) as u32;

            let index = y * TEX_WIDTH + x;

            // textura roja con una cruz negra
            textures[0][index] = 65536 * 254 * ((x != y && x != TEX_WIDTH - y) as u32);

            // degradado gris
            textures[1][index] = xycolor + 256 * xycolor + 65536 * xycolor;

            // degradado amarillo
            textures[2][index] = 256 * xycolor + 65536 * xycolor;

            // patrón XOR en escala de grises
            textures[3][index] = xorcolor + 256 * xorcolor + 65536 * xorcolor;

            // patrón XOR verde
            textures[4][index] = 256 * xorcolor;

            // ladrillos rojos
            textures[5][index] = 65536 * 192 * ((x % 16 != 0 && y % 16 != 0) as u32);

            // degradado rojo
            textures[6][index] = 65536 * ycolor;

            // gris plano
            textures[7][index] = 128 + 256 * 128 + 65536 * 128;
        }
    }

    textures
}

fn get_red(color: u32) -> u8 {
    ((color >> 16) & 0xFF) as u8
}

fn get_green(color: u32) -> u8 {
    ((color >> 8) & 0xFF) as u8
}

fn get_blue(color: u32) -> u8 {
    (color & 0xFF) as u8
}

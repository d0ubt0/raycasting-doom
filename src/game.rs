use winit::keyboard::{KeyCode, PhysicalKey};

use crate::config::GameConfig;
use std::ops::{Add, Mul};

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

enum Action {
    Forward,
    Backward,
    Left,
    Right,
    TurnLeft,
    TurnRight,
    Sprint,
}

impl ControlMap {
    pub fn action(&self, key: KeyCode) -> Option<Action> {
        match key {
            k if k == self.forward => Some(Action::Forward),
            k if k == self.backward => Some(Action::Backward),
            k if k == self.left => Some(Action::Left),
            k if k == self.right => Some(Action::Right),
            k if k == self.turn_left => Some(Action::TurnLeft),
            k if k == self.turn_right => Some(Action::TurnRight),
            k if k == self.sprint => Some(Action::Sprint),
            _ => None,
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
}

pub struct Player {
    pub position: Vector2D,
    pub direction: Vector2D,
    pub speed: f64,
    pub fov: f64,
}

impl Player {
    fn move_player(&mut self, direction: Vector2D, delta_time: f64) {
        self.position = self.position + (self.speed * delta_time) * direction;
    }

    fn change_direction(&mut self, dx: f64, delta_time: f64) {
        self.direction = self.direction.rotate(dx * delta_time)
    }
}

pub struct Game {
    player: Player,
    map: Vec<Vec<u8>>,
    control_map: ControlMap,
}

impl Game {
    pub fn new(player: Player, map: Vec<Vec<u8>>) -> Self {
        Self {
            player,
            map,
            control_map: Default::default(),
        }
    }

    pub fn player_vision(&mut self, width: usize, height: usize, frame: &mut [u8]) {
        let player = &self.player;

        let start_direction = player.direction.rotate(-player.fov / 2.0);

        let step = player.fov / width as f64;

        for x in 0..width {
            let direction = start_direction.rotate(step * x as f64);

            if let Some(hit) = self.ray_cast(player.position, direction) {
                let wall_height = (height as f64 / hit.distance) as usize;

                let start_y = height.saturating_sub(wall_height) / 2;

                let end_y = (start_y + wall_height).min(height);

                for y in start_y..end_y {
                    let index = (y * width + x) * 4;

                    frame[index] = 255;
                    frame[index + 1] = 255;
                    frame[index + 2] = 255;
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

    fn ray_cast(&self, position: Vector2D, direction: Vector2D) -> Option<HitRecord> {
        let mut scalar = 1.0;
        let mut ray_position = position + scalar * direction;

        while self.is_inside(&ray_position) {
            let index_x = ray_position.x.floor() as usize;
            let index_y = ray_position.y.floor() as usize;

            if self.map[index_y][index_x] != 0 {
                return Some(HitRecord {
                    hit_pos: (ray_position),
                    distance: (ray_position.distance(&position)),
                    map_pos: (index_x, index_y),
                });
            }
            scalar += 0.1;
            ray_position = position + scalar * direction;
        }

        None
    }

    pub fn get_keyboard_input(&mut self, physical_key: PhysicalKey) {
        let delta_time = 0.1;

        if let PhysicalKey::Code(key) = physical_key {
            if let Some(action) = self.control_map.action(key) {
                match action {
                    Action::Forward => self.player.move_player(self.player.direction, delta_time),
                    Action::Backward => self
                        .player
                        .move_player(self.player.direction.rotate(180.0), delta_time),
                    Action::Left => self
                        .player
                        .move_player(self.player.direction.rotate(-90.0), delta_time),
                    Action::Right => self
                        .player
                        .move_player(self.player.direction.rotate(90.0), delta_time),
                    Action::TurnLeft => {
                        self.player.change_direction(-100.0, 0.1);
                    }
                    Action::TurnRight => {
                        self.player.change_direction(100.0, 0.1);
                    }
                    Action::Sprint => {}
                }
            }
        }
    }

    pub fn handle_mouse_look(&mut self, x: f64, y: f64) {
        let delta_time = 1.0;
        let vector = Vector2D { x, y };

        self.player.change_direction(vector.x, delta_time);
    }
}

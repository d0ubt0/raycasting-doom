use std::ops::{Add, Mul};

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

pub struct Game {
    player: Player,
    map: Vec<Vec<u8>>,
}

impl Game {
    pub fn new(player: Player, map: Vec<Vec<u8>>) -> Self {
        Self { player, map }
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
}

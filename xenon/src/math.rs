use std::ops::{Add, AddAssign};

use sdl2::rect::Rect as SDLRect;

#[derive(Copy, Clone, Debug)]
pub struct Vec2D {
    x: f32,
    y: f32,
}

impl Vec2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Add for Vec2D {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Vec2D {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

impl Rect {
    pub fn from_components(position: Vec2D, dimensions: Vec2D) -> Self {
        Self {
            left: position.x,
            top: position.y,
            width: dimensions.x,
            height: dimensions.y,
        }
    }
}

impl Into<SDLRect> for Rect {
    fn into(self) -> SDLRect {
        SDLRect::new(self.left as i32, self.top as i32, self.width as u32, self.height as u32)
    }
}

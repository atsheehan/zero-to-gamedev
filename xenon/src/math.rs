use std::ops::{Add, AddAssign, Sub};

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

    pub fn normalize(self) -> Self {
        let magnitude = self.magnitude();

        Self {
            x: self.x / magnitude,
            y: self.y / magnitude,
        }
    }

    pub fn magnitude(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
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

impl Sub for Vec2D {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
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

    pub fn translate(self, vec: Vec2D) -> Self {
        Self {
            left: self.left + vec.x,
            top: self.top + vec.y,
            width: self.width,
            height: self.height,
        }
    }

    pub fn center(&self) -> Vec2D {
        Vec2D {
            x: self.left + self.width / 2.0,
            y: self.top + self.height / 2.0,
        }
    }

    pub fn position(&self) -> Vec2D {
        Vec2D {
            x: self.left,
            y: self.top,
        }
    }

    pub fn overlaps(&self, other: Self) -> bool {
        self.left() < other.right() &&
            self.right() > other.left() &&
            self.top() < other.bottom() &&
            self.bottom() > other.top()
    }

    fn left(&self) -> f32 {
        self.left
    }

    fn right(&self) -> f32 {
        self.left + self.width
    }

    fn top(&self) -> f32 {
        self.top
    }

    fn bottom(&self) -> f32 {
        self.top + self.height
    }
}

impl Into<SDLRect> for Rect {
    fn into(self) -> SDLRect {
        SDLRect::new(self.left as i32, self.top as i32, self.width as u32, self.height as u32)
    }
}

use std::ops::{Add, AddAssign, Sub};

use sdl2::rect::Rect as SDLRect;

#[derive(Copy, Clone, Debug)]
pub struct Vec2D {
    pub x: f32,
    pub y: f32,
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

    pub fn y_vec(self) -> Self {
        Self {
            x: 0.0,
            y: self.y,
        }
    }

    pub fn x_vec(self) -> Self {
        Self {
            x: self.x,
            y: 0.0,
        }
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

    pub fn left(&self) -> f32 {
        self.left
    }

    pub fn right(&self) -> f32 {
        self.left + self.width
    }

    pub fn top(&self) -> f32 {
        self.top
    }

    pub fn bottom(&self) -> f32 {
        self.top + self.height
    }

    pub fn set_left(self, left: f32) -> Self {
        Self {
            left,
            top: self.top,
            width: self.width,
            height: self.height,
        }
    }

    pub fn set_right(self, right: f32) -> Self {
        Self {
            left: right - self.width,
            top: self.top,
            width: self.width,
            height: self.height,
        }
    }

    pub fn set_top(self, top: f32) -> Self {
        Self {
            left: self.left,
            top: top,
            width: self.width,
            height: self.height,
        }
    }

    pub fn set_bottom(self, bottom: f32) -> Self {
        Self {
            left: self.left,
            top: bottom - self.height,
            width: self.width,
            height: self.height,
        }
    }
}

impl Into<SDLRect> for Rect {
    fn into(self) -> SDLRect {
        SDLRect::new(self.left as i32, self.top as i32, self.width as u32, self.height as u32)
    }
}

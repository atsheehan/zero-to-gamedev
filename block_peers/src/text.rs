use sdl2::pixels::Color;

use std::hash::{Hash, Hasher};

use crate::render::{Dimensions, Position};

/// Text uses the builder pattern to configure properties required for rendering text with the
/// Renderer.
#[derive(Debug)]
pub struct Text {
    pub raw: String,
    pub position: Position,
    pub dimensions: Dimensions,
    pub color: Color,
}

impl Text {
    pub fn new(raw: &'static str) -> Self {
        Self {
            raw: String::from(raw),
            position: Position::LeftTop(0, 0),
            dimensions: Dimensions::Height(40),
            color: Color::RGB(255, 255, 255),
        }
    }

    // ------------
    // Raw Accessors
    // ------------
    pub fn position(&mut self, position: Position) -> &mut Self {
        self.position = position;
        self
    }

    pub fn dimensions(&mut self, dimensions: Dimensions) -> &mut Self {
        self.dimensions = dimensions;
        self
    }

    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    // -------
    // Utility
    // -------
    pub fn color_gray(&mut self) -> &mut Self {
        self.color = Color::RGB(128, 128, 128);
        self
    }

    pub fn center_xy(&mut self, x: i32, y: i32) -> &mut Self {
        self.position(Position::Center(x, y));
        self
    }

    pub fn height(&mut self, height: u32) -> &mut Self {
        self.dimensions(Dimensions::Height(height));
        self
    }

    pub fn build(&mut self) -> Self {
        Self {
            raw: self.raw.clone(),
            position: self.position,
            dimensions: self.dimensions,
            color: self.color,
        }
    }
}

impl Hash for Text {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

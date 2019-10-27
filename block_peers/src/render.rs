use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::brick::BrickType;

/// Which image to render when calling `render_image`. This module
/// maps the image to the appropriate location in the larger texture.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum Image {
    RedBrick,
    GreenBrick,
    BlueBrick,
    YellowBrick,
    OrangeBrick,
    PurpleBrick,
    TealBrick,
    SmokeBrick(u32),
    Title,
    // Temp: once we have font rendering, remove this
    SpaceText,
}

impl Image {
    fn source_rect(self) -> Rect {
        match self {
            Self::Title => Rect::new(0, 0, 440, 65),
            Self::SpaceText => Rect::new(0, 0, 99, 28),
            Self::RedBrick => Rect::new(0, 0, 32, 32),
            Self::GreenBrick => Rect::new(32, 0, 32, 32),
            Self::BlueBrick => Rect::new(64, 0, 32, 32),
            Self::YellowBrick => Rect::new(96, 0, 32, 32),
            Self::OrangeBrick => Rect::new(128, 0, 32, 32),
            Self::PurpleBrick => Rect::new(160, 0, 32, 32),
            Self::TealBrick => Rect::new(192, 0, 32, 32),
            Self::SmokeBrick(frame) => {
                if frame > 12 {
                    panic!("unavailable smoke brick, greatest index is 12")
                }
                Rect::new((frame * 32) as i32, 32, 32, 32)
            }
        }
    }

    pub fn max_smoke_frame() -> u32 {
        12
    }

    pub fn from_brick_type(brick_type: BrickType) -> Self {
        use BrickType::*;
        match brick_type {
            Red => Image::RedBrick,
            Green => Image::GreenBrick,
            Blue => Image::BlueBrick,
            Yellow => Image::YellowBrick,
            Orange => Image::OrangeBrick,
            Purple => Image::PurpleBrick,
            Teal => Image::TealBrick,
            Smoke(frame) => Image::SmokeBrick(frame),
        }
    }
}

/// Used to specify how opaque an image should be rendered.
#[derive(Copy, Clone)]
pub enum Opacity {
    Opaque,
    Translucent(u8),
}

impl Opacity {
    fn alpha(self) -> u8 {
        match self {
            Opacity::Opaque => u8::max_value(),
            Opacity::Translucent(alpha) => alpha,
        }
    }
}

pub struct Renderer {
    canvas: WindowCanvas,
    pieces: Texture,
    title: Texture,
    space: Texture,
}

#[derive(Debug)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

impl Renderer {
    pub fn new(canvas: WindowCanvas) -> Self {
        let texture_creator = canvas.texture_creator();
        let pieces = texture_creator
            .load_texture(Path::new("assets/tiles.png"))
            .unwrap();
        let title = texture_creator
            .load_texture(Path::new("assets/title.png"))
            .unwrap();
        let space = texture_creator
            .load_texture(Path::new("assets/space.png"))
            .unwrap();

        Self {
            canvas,
            pieces,
            title,
            space,
        }
    }

    pub fn size(&self) -> WindowSize {
        let result = self
            .canvas
            .output_size()
            .expect("unable to determine window size of canvas");

        WindowSize {
            width: result.0,
            height: result.1,
        }
    }

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(Color::RGB(75, 75, 75));
        self.canvas.clear();
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.canvas.set_draw_color(color);
        self.canvas.fill_rect(rect).expect("failed to fill rect");
    }

    pub fn render_image(&mut self, image: Image, dest_rect: Rect, opacity: Opacity) {
        match image {
            Image::Title => {
                self.canvas
                    .copy(&self.title, image.source_rect(), dest_rect)
                    .expect("failed to render image");
            }
            Image::SpaceText => {
                self.canvas
                    .copy(&self.space, image.source_rect(), dest_rect)
                    .expect("failed to render image");
            }
            _ => {
                self.pieces.set_alpha_mod(opacity.alpha());
                self.canvas
                    .copy(&self.pieces, image.source_rect(), dest_rect)
                    .expect("failed to render image");
            }
        }
    }

    pub fn render_title(&mut self, x: i32, y: i32) {
        self.render_image(Image::Title, Rect::new(x, y, 440, 65), Opacity::Opaque);
    }

    pub fn render_space(&mut self, x: i32, y: i32) {
        self.render_image(Image::SpaceText, Rect::new(x, y, 99, 28), Opacity::Opaque);
    }
}

// --------
// Tests
// --------

#[test]
#[should_panic]
fn test_invalid_smoke_brick() {
    Image::SmokeBrick(13).source_rect();
}

#[test]
fn test_valid_smoke_brick() {
    for i in 0..12 {
        Image::SmokeBrick(i as u32).source_rect();
    }
}

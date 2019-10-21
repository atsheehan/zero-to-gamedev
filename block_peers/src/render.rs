use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use std::path::Path;

/// Which image to render when calling `render_image`. This module
/// maps the image to the appropriate location in the larger texture.
#[derive(Copy, Clone)]
pub enum Image {
    RedBrick,
    GreenBrick,
    BlueBrick,
    YellowBrick,
    OrangeBrick,
    PurpleBrick,
    TealBrick,
    SmokeBrick(i32),
}

impl Image {
    fn source_rect(self) -> Rect {
        match self {
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
                let x = frame * 32;
                Rect::new(x, 32, 32, 32)
            }
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
    texture: Texture,
}

impl Renderer {
    pub fn new(canvas: WindowCanvas) -> Self {
        let texture_creator = canvas.texture_creator();
        let texture = texture_creator
            .load_texture(Path::new("assets/tiles.png"))
            .unwrap();

        Self { canvas, texture }
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
        self.texture.set_alpha_mod(opacity.alpha());
        self.canvas
            .copy(&self.texture, image.source_rect(), dest_rect)
            .expect("failed to render image");
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
        Image::SmokeBrick(i as i32).source_rect();
    }
}

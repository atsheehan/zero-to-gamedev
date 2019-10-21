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
}

impl Image {
    fn source_rect(self) -> Rect {
        match self {
            Image::RedBrick => Rect::new(0, 0, 32, 32),
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

        Self {
            canvas,
            texture,
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
        self.canvas.fill_rect(rect).unwrap();
    }

    pub fn render_image(&mut self, image: Image, dest_rect: Rect, opacity: Opacity) {
        self.texture.set_alpha_mod(opacity.alpha());
        self.canvas.copy(&self.texture, image.source_rect(), dest_rect).unwrap();
    }
}

use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::ttf::{Font, Sdl2TtfContext};
use serde::{Deserialize, Serialize};

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;

use crate::brick::BrickType;
use crate::text::Text;

// These constants define the logical size of the screen: whenever
// trying to position something on the screen we should use these
// coordinates rather than the actual window coordinates. SDL will
// translate the logical coordinates to window coordinates if the
// window changes size/shape.
pub const VIEWPORT_WIDTH: u32 = 800;
pub const VIEWPORT_HEIGHT: u32 = 600;

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
    SmokeBrick(u16),
    Title,
    PlayingField,
}

impl Image {
    fn source_rect(self) -> Rect {
        match self {
            Self::PlayingField => Rect::new(0, 0, 800, 600),
            Self::Title => Rect::new(0, 64, 440, 65),
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

    pub fn max_smoke_frame() -> u16 {
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

/// Used to specify how large an image should be. If we specify just
/// one dimension, the renderer can figure out how large the other
/// dimension would be to maintain the aspect ratio. This is
/// especially useful for fonts since the size varies based on the
/// text being rendered and we don't know the size until runtime.
///
/// TODO: Add Dimensions::Width when needed
#[derive(Debug, Copy, Clone, Hash)]
pub enum Dimensions {
    Height(u32),
}

/// Used to specify where to render an image.
#[derive(Debug, Copy, Clone, Hash)]
pub enum Position {
    Center(i32, i32),
    LeftTop(i32, i32),
}

pub struct Renderer<'ttf> {
    canvas: WindowCanvas,
    background: Texture,
    pieces: Texture,
    string_textures: HashMap<u64, Texture>,
    font: Font<'ttf, 'static>,
    x_offset: i32,
    y_offset: i32,
}

impl<'ttf> Renderer<'ttf> {
    pub fn new(mut canvas: WindowCanvas, ttf_context: &'ttf Sdl2TtfContext) -> Self {
        canvas
            .set_logical_size(VIEWPORT_WIDTH, VIEWPORT_HEIGHT)
            .unwrap();

        let texture_creator = canvas.texture_creator();
        let pieces = texture_creator
            .load_texture(Path::new("assets/tiles.png"))
            .unwrap();

        let background = texture_creator
            .load_texture(Path::new("assets/background.png"))
            .unwrap();

        let font = ttf_context
            .load_font(Path::new("assets/VT323-Regular.ttf"), 20)
            .unwrap();

        let string_textures = HashMap::new();

        Self {
            canvas,
            pieces,
            background,
            string_textures,
            font,
            x_offset: 0,
            y_offset: 0,
        }
    }

    pub fn set_offset(&mut self, x_offset: i32, y_offset: i32) {
        self.x_offset = x_offset;
        self.y_offset = y_offset;
    }

    pub fn get_offset(&self) -> (i32, i32) {
        (self.x_offset, self.y_offset)
    }

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(Color::RGB(75, 75, 75));
        self.canvas.clear();
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let rect = translate(rect, self.x_offset, self.y_offset);

        self.canvas.set_draw_color(color);
        self.canvas.fill_rect(rect).expect("failed to fill rect");
    }

    // TODO: update function to use Position / Dimensions similar to render_text
    pub fn render_image<R>(&mut self, image: Image, dest_rect: R, opacity: Opacity)
    where
        R: Into<Rect>,
    {
        let dest_rect = translate(dest_rect.into(), self.x_offset, self.y_offset);

        match image {
            Image::PlayingField => {
                self.canvas
                    .copy(&self.background, image.source_rect(), dest_rect)
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

    pub fn render_text(&mut self, text: Text) {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let key = hasher.finish();

        // TODO: Look into Entry API for less verbosity
        if !self.string_textures.contains_key(&key) {
            let texture = self
                .canvas
                .texture_creator()
                .create_texture_from_surface(
                    self.font
                        .render(&text.raw)
                        // We render all base fonts white and then apply a color mod for what color
                        // it 'should' be rendered. This way we only need 1 texture per text.
                        .solid(Color::RGB(255, 255, 255))
                        .unwrap(),
                )
                .unwrap();

            self.string_textures.insert(key, texture);
        }

        let texture = self
            .string_textures
            .get_mut(&key)
            .expect("text texture missing but should always be present");

        texture.set_color_mod(text.color.r, text.color.g, text.color.b);

        let dest_rect = translate(
            compute_dest_rect(&texture, text.position, text.dimensions),
            self.x_offset,
            self.y_offset,
        );

        self.canvas.copy(&texture, None, dest_rect).unwrap();
    }
}

fn compute_dest_rect(texture: &Texture, position: Position, dimensions: Dimensions) -> Rect {
    let texture_details = texture.query();

    let (width, height) = match dimensions {
        Dimensions::Height(target_height) => {
            let target_width = ((texture_details.width as f32 / texture_details.height as f32)
                * target_height as f32) as u32;
            (target_width, target_height)
        }
    };

    let (left, top) = match position {
        Position::Center(x_center, y_center) => {
            (x_center - width as i32 / 2, y_center - height as i32 / 2)
        }
        Position::LeftTop(left, top) => (left, top),
    };

    Rect::new(left, top, width, height)
}

fn translate(rect: Rect, x_offset: i32, y_offset: i32) -> Rect {
    Rect::new(
        rect.x() + x_offset,
        rect.y() + y_offset,
        rect.width(),
        rect.height(),
    )
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
        Image::SmokeBrick(i as u16).source_rect();
    }
}

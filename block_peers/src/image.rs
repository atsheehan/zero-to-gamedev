use crate::brick::BrickType;
use crate::render::Frame;
use sdl2::rect::Rect;
use serde::{Deserialize, Serialize};

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

impl Frame for Image {
    fn source_rect(self) -> Rect {
        match self {
            Self::PlayingField => Rect::new(0, 128, 800, 600),
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
}

impl Image {
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
            Attacked => Image::SmokeBrick(0),
        }
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
        Image::SmokeBrick(i as u16).source_rect();
    }
}

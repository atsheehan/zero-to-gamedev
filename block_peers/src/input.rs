// use sdl2::keyboard::Keycode;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum Keycode {
    A,
    S,
    D,
    W,
    E,
    Return,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum InputEvent {
    KeyDown(Keycode),
}

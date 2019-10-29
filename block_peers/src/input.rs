use serde::{Deserialize, Serialize};

// One possibility is sending raw input events to the server. Alternatively, we could create custom
// events which allows different clients to map different key codes to their preferred controls.

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

// Decoupled Input Events

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum GameInputEvent {
    MoveLeft,
    MoveRight,
    MoveDown,
    ForceToBottom,
    Rotate,
}

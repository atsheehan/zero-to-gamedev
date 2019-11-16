use sdl2::mixer::{self, Channel, Chunk, Music};

use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::hash::Hash;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

const INFINITELY: i32 = std::i32::MAX;

pub static SOUND_IS_ENABLED: AtomicBool = AtomicBool::new(true);

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum SoundEffect {
    SmokeOne,
    SmokeTwo,
    SmokeThree,
    SmokeFour,
}

pub struct AudioManager<'music> {
    chunks: HashMap<SoundEffect, Chunk>,
    bg_music: Music<'music>,
}

impl<'music> AudioManager<'music> {
    pub fn new() -> Self {
        // Open implementation mixer
        mixer::open_audio(44_100, mixer::AUDIO_S16LSB, mixer::DEFAULT_CHANNELS, 1_024).unwrap();
        mixer::allocate_channels(4);

        // Set up all assets
        let bg_music =
            sdl2::mixer::Music::from_file(Path::new("../../assets/background.wav")).unwrap();
        let smoke_1 = Chunk::from_file(Path::new("../../assets/smoke-1.wav")).unwrap();
        let smoke_2 = Chunk::from_file(Path::new("../../assets/smoke-2.wav")).unwrap();
        let smoke_3 = Chunk::from_file(Path::new("../../assets/smoke-3.wav")).unwrap();
        let smoke_4 = Chunk::from_file(Path::new("../../assets/smoke-4.wav")).unwrap();

        let mut chunks = HashMap::new();

        chunks.insert(SoundEffect::SmokeOne, smoke_1);
        chunks.insert(SoundEffect::SmokeTwo, smoke_2);
        chunks.insert(SoundEffect::SmokeThree, smoke_3);
        chunks.insert(SoundEffect::SmokeFour, smoke_4);

        Self { chunks, bg_music }
    }

    pub fn play_sfx(&mut self, effect: SoundEffect) {
        if !SOUND_IS_ENABLED.load(Ordering::Relaxed) {
            return;
        }

        match self.chunks.entry(effect) {
            Occupied(entry) => {
                Channel::all().play(entry.get(), 0).unwrap();
            }
            Vacant(_) => {
                panic!("missing sound effect");
            }
        }
    }

    pub fn play_bg_music(&self) {
        if !SOUND_IS_ENABLED.load(Ordering::Relaxed) {
            return;
        }

        self.bg_music
            .play(INFINITELY)
            .expect("unable to play background music");
    }

    /// Volume to play background music at. Value can be between 0 and 1, 1 being the maximum
    /// available volume and 0 being off.
    pub fn set_volume(&self, percent: f32) {
        if percent > 0.0 && percent < 1.0 {
            let amount = (percent * 128.0) as i32;
            sdl2::mixer::Music::set_volume(amount);
        } else {
            panic!("tried to set volume with invalid value, must be between 0 and 1");
        }
    }

    /// User requested to turn sound off somewhere in the UI.
    pub fn ui_turn_sound_off(&mut self) {
        if !SOUND_IS_ENABLED.load(Ordering::Relaxed) {
            sdl2::mixer::Music::halt();
        }
    }

    /// User requested to turn sound on somewhere in the UI.
    pub fn ui_turn_sound_on(&mut self) {
        if SOUND_IS_ENABLED.load(Ordering::Relaxed) {
            self.play_bg_music();
        }
    }
}

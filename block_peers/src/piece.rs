use rand::Rng;

pub type Piece = [bool; 16];

pub fn random_next_piece() -> Piece {
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0, PIECES.len());
    PIECES[idx as usize]
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub const PIECES: [Piece; 7] = [
    [
        false, true,  true,  true, 
        false, false, true,  false, 
        false, false, false, false, 
        false, false, false, false,
    ],
    [
        false, false, true, false,
        false, false, true, false,
        false, false, true, false,
        false, false, true, false
    ],
    [
        false, false, false, false,
        false, true,  true,  false,
        false, true,  true,  false,
        false, false, false, false
    ],
    [
        false, false, false, false,
        true,  true,  false,  false,
        false, true,  true,  false,
        false, false, false, false
    ],
    [
        false,  true, false, false,
        false,  true, true,  false,
        false, false, true,  false,
        false, false, false, false
    ],
    [
        false,  false, true, false,
        false,  false, true, false,
        false,   true, true, false,
        false, false, false, false
    ],
    [
        false,  true, false, false,
        false,  true, false, false,
        false,  true, true,  false,
        false, false, false, false
    ],
];

#[derive(Debug, PartialEq, Clone)]
pub enum Rotation {
    Zero,
    Ninety,
    OneEighty,
    TwoSeventy,
}

impl Rotation {
    pub fn next(&self) -> Rotation {
        use Rotation::*;
        match self {
            Zero => Ninety,
            Ninety => OneEighty,
            OneEighty => TwoSeventy,
            TwoSeventy => Zero,
        }
    }
}

pub fn rotated_index(px: usize, py: usize, rotation: Rotation) -> usize {
    use Rotation::*;

    match rotation {
        Zero => py * 4 + px,
        Ninety => 12 + py - (px * 4),
        OneEighty => 15 - (py * 4) - px,
        TwoSeventy => 3 - py + (px * 4),
    }
}


#[test]
fn test_rotation_index() {
    println!("TODO");
}

#[test]
fn test_next_rotation() {
    assert_eq!(Rotation::Zero.next(), Rotation::Ninety);
    assert_eq!(Rotation::Ninety.next(), Rotation::OneEighty);
    assert_eq!(Rotation::OneEighty.next(), Rotation::TwoSeventy);
    assert_eq!(Rotation::TwoSeventy.next(), Rotation::Zero);
}

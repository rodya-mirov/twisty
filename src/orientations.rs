use enum_iterator::Sequence;
use rand::Rng;

/// A 3-variant orientation enum which matches corners on many common types of twist puzzles.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Ord, PartialOrd, Sequence)]
pub enum CornerOrientation {
    Normal,
    CW,
    CCW,
}

impl Default for CornerOrientation {
    fn default() -> Self {
        Self::Normal
    }
}

impl CornerOrientation {
    /// A simple cast to u8 for encoding. Guaranteed to have minimal size, that is,
    /// using at most two bits.
    #[inline(always)]
    pub fn as_u8_two_bits(self) -> u8 {
        match self {
            CornerOrientation::Normal => 0,
            CornerOrientation::CW => 1,
            CornerOrientation::CCW => 2,
        }
    }

    #[inline(always)]
    pub fn cw(self) -> Self {
        match self {
            CornerOrientation::Normal => CornerOrientation::CW,
            CornerOrientation::CW => CornerOrientation::CCW,
            CornerOrientation::CCW => CornerOrientation::Normal,
        }
    }

    #[inline(always)]
    pub fn ccw(self) -> Self {
        match self {
            CornerOrientation::Normal => CornerOrientation::CCW,
            CornerOrientation::CCW => CornerOrientation::CW,
            CornerOrientation::CW => CornerOrientation::Normal,
        }
    }
}

/// A two-variant orientation enum which behaves like edges in many common types of twist puzzles.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Sequence)]
pub enum EdgeOrientation {
    Normal,
    Flipped,
}

impl Default for EdgeOrientation {
    fn default() -> Self {
        Self::Normal
    }
}

impl EdgeOrientation {
    #[inline(always)]
    pub fn flipped(&self) -> Self {
        match self {
            EdgeOrientation::Normal => EdgeOrientation::Flipped,
            EdgeOrientation::Flipped => EdgeOrientation::Normal,
        }
    }

    pub fn random<R: Rng>(r: &mut R) -> Self {
        if r.gen_bool(0.5) {
            Self::Normal
        } else {
            Self::Flipped
        }
    }
    /// A simple cast to u8 for encoding. Guaranteed to have minimal size, that is,
    /// using at most one bits.
    #[inline(always)]
    pub fn as_u8_one_bit(self) -> u8 {
        match self {
            EdgeOrientation::Normal => 0,
            EdgeOrientation::Flipped => 1,
        }
    }
}

/// A 3-variant orientation enum which matches corners on many common types of twist puzzles.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub enum CornerOrientation {
    Normal,
    CW,
    CCW,
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
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub enum EdgeOrientation {
    Normal,
    Flipped,
}

impl EdgeOrientation {
    #[inline(always)]
    pub fn flipped(&self) -> Self {
        match self {
            EdgeOrientation::Normal => EdgeOrientation::Flipped,
            EdgeOrientation::Flipped => EdgeOrientation::Normal,
        }
    }
}

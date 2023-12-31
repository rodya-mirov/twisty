/// A 3-variant orientation enum which matches corners on many common types of twist puzzles.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub enum CornerOrientation {
    Normal,
    CW,
    CCW,
}

impl CornerOrientation {
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

use enum_iterator::Sequence;
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::Rng;

/// A 3-variant orientation enum which matches corners on many common types of twist puzzles.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Ord, PartialOrd, Sequence)]
pub enum CornerOrientation {
    Normal,
    CW,
    CCW,
}

impl CornerOrientation {
    pub fn flip(&self) -> Self {
        match *self {
            CornerOrientation::Normal => CornerOrientation::Normal,
            CornerOrientation::CW => CornerOrientation::CCW,
            CornerOrientation::CCW => CornerOrientation::CW,
        }
    }

    pub fn total(orientations: &[CornerOrientation]) -> CornerOrientation {
        let mut total = CornerOrientation::Normal;

        for o in orientations.iter().copied() {
            total = total + o;
        }

        total
    }
}

impl std::ops::Add for CornerOrientation {
    type Output = CornerOrientation;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        match self {
            CornerOrientation::Normal => rhs,
            CornerOrientation::CW => match rhs {
                CornerOrientation::Normal => CornerOrientation::CW,
                CornerOrientation::CW => CornerOrientation::CCW,
                CornerOrientation::CCW => CornerOrientation::Normal,
            },
            CornerOrientation::CCW => match rhs {
                CornerOrientation::Normal => CornerOrientation::CCW,
                CornerOrientation::CW => CornerOrientation::Normal,
                CornerOrientation::CCW => CornerOrientation::CW,
            },
        }
    }
}

impl Default for CornerOrientation {
    fn default() -> Self {
        Self::Normal
    }
}

impl Distribution<CornerOrientation> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CornerOrientation {
        let val = rng.gen_range(0..3);

        match val {
            0 => CornerOrientation::Normal,
            1 => CornerOrientation::CW,
            2 => CornerOrientation::CCW,
            other => unreachable!("Should get a value from 0 to 2, but got {other}"),
        }
    }
}

impl CornerOrientation {
    #[inline(always)]
    pub fn pack_two_bits_u64(self, source: &mut u64) {
        *source = (*source << 2) + (self.as_u8_two_bits() as u64);
    }

    // TODO: unite this and the above as a trait maybe?
    #[inline(always)]
    pub fn pack_two_bits_u32(self, source: &mut u32) {
        *source = (*source << 2) + (self.as_u8_two_bits() as u32);
    }

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
    pub fn cw_mut(&mut self) {
        *self = self.cw()
    }

    #[inline(always)]
    pub fn ccw(self) -> Self {
        match self {
            CornerOrientation::Normal => CornerOrientation::CCW,
            CornerOrientation::CCW => CornerOrientation::CW,
            CornerOrientation::CW => CornerOrientation::Normal,
        }
    }

    #[inline(always)]
    pub fn ccw_mut(&mut self) {
        *self = self.ccw()
    }

    // needed for some macros; just don't do any changes
    #[inline(always)]
    pub fn no_swap(self) -> Self {
        self
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
    /// using at most one bit.
    #[inline(always)]
    pub fn as_u8_one_bit(self) -> u8 {
        match self {
            EdgeOrientation::Normal => 0,
            EdgeOrientation::Flipped => 1,
        }
    }

    #[inline(always)]
    pub fn pack(self, bits: &mut u64) {
        *bits = (*bits << 1) + (self.as_u8_one_bit() as u64)
    }
}

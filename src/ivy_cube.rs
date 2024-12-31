use std::fmt::Formatter;

use derive_more::Display;
use rand::Rng;

use crate::cubesearch::SimpleStartState;
use crate::idasearch::heuristic_helpers::bounded_cache;
use crate::idasearch::{Heuristic, Solvable};
use crate::moves::{CanReverse, CornerTwistAmt};
use crate::orientations::CornerOrientation;
use crate::random_helpers::{shuffle_with_parity, TwoParity};
use crate::scrambles::RandomInit;

// 2 bits per corner * 4 corners plus 3 bits per center * 6 centers is 24 bits
type PackedBits = u32;

#[repr(u8)]
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
enum CenterCubelet {
    F,
    R,
    L,
    U,
    D,
    B,
}

impl CenterCubelet {
    #[inline(always)]
    fn pack(self, bits: &mut PackedBits) {
        *bits = (*bits << 3) | (self as PackedBits);
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct CenterState {
    f: CenterCubelet,
    u: CenterCubelet,
    r: CenterCubelet,
    l: CenterCubelet,
    d: CenterCubelet,
    b: CenterCubelet,
}

impl CenterState {
    #[inline(always)]
    fn pack(&self, bits: &mut PackedBits) {
        self.f.pack(bits);
        self.u.pack(bits);
        self.l.pack(bits);
        self.r.pack(bits);
        self.b.pack(bits);
        self.d.pack(bits);
    }

    #[inline(always)]
    fn ufl(self) -> Self {
        Self {
            u: self.l,
            l: self.f,
            f: self.u,
            ..self
        }
    }

    #[inline(always)]
    fn dfr(self) -> Self {
        Self {
            f: self.d,
            d: self.r,
            r: self.f,
            ..self
        }
    }

    #[inline(always)]
    fn ubr(self) -> Self {
        Self {
            u: self.r,
            r: self.b,
            b: self.u,
            ..self
        }
    }

    #[inline(always)]
    fn dbl(self) -> Self {
        Self {
            l: self.b,
            b: self.d,
            d: self.l,
            ..self
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct CornerState {
    ufl: CornerOrientation,
    dfr: CornerOrientation,
    ubr: CornerOrientation,
    dbl: CornerOrientation,
}

impl CornerState {
    fn pack(self, bits: &mut PackedBits) {
        self.ufl.pack_two_bits_u32(bits);
        self.dfr.pack_two_bits_u32(bits);
        self.ubr.pack_two_bits_u32(bits);
        self.dbl.pack_two_bits_u32(bits);
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct IvyCube {
    centers: CenterState,
    corners: CornerState,
}

impl IvyCube {
    pub fn solved_state() -> Self {
        Self {
            corners: CornerState {
                ufl: CornerOrientation::Normal,
                dfr: CornerOrientation::Normal,
                ubr: CornerOrientation::Normal,
                dbl: CornerOrientation::Normal,
            },
            centers: CenterState {
                u: CenterCubelet::U,
                d: CenterCubelet::D,
                f: CenterCubelet::D,
                b: CenterCubelet::B,
                r: CenterCubelet::R,
                l: CenterCubelet::L,
            },
        }
    }

    #[inline(always)]
    fn ufl(&self) -> Self {
        Self {
            centers: self.centers.ufl(),
            corners: CornerState {
                ufl: self.corners.ufl.cw(),
                ..self.corners
            },
        }
    }

    #[inline(always)]
    fn dfr(&self) -> Self {
        Self {
            centers: self.centers.dfr(),
            corners: CornerState {
                dfr: self.corners.dfr.cw(),
                ..self.corners
            },
        }
    }

    #[inline(always)]
    fn ubr(&self) -> Self {
        Self {
            centers: self.centers.ubr(),
            corners: CornerState {
                ubr: self.corners.ubr.cw(),
                ..self.corners
            },
        }
    }

    #[inline(always)]
    fn dbl(&self) -> Self {
        Self {
            centers: self.centers.dbl(),
            corners: CornerState {
                dbl: self.corners.dbl.cw(),
                ..self.corners
            },
        }
    }
}

impl RandomInit for IvyCube {
    fn random_state<R: Rng>(r: &mut R) -> Self {
        let centers = [
            CenterCubelet::U,
            CenterCubelet::D,
            CenterCubelet::R,
            CenterCubelet::L,
            CenterCubelet::F,
            CenterCubelet::B,
        ];
        let centers = shuffle_with_parity(r, &centers, TwoParity::Even);

        Self {
            centers: CenterState {
                u: centers[0],
                d: centers[1],
                r: centers[2],
                l: centers[3],
                f: centers[4],
                b: centers[5],
            },
            corners: CornerState {
                ufl: r.gen(),
                ubr: r.gen(),
                dbl: r.gen(),
                dfr: r.gen(),
            },
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Display)]
enum Dir {
    UFL,
    UBR,
    DBL,
    DFR,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Move(Dir, CornerTwistAmt);

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.0, self.1)
    }
}

impl CanReverse for Move {
    fn reverse(&self) -> Self {
        Move(self.0, self.1.reverse())
    }
}

impl Solvable for IvyCube {
    type Move = Move;

    fn is_solved(&self) -> bool {
        self == &Self::solved_state()
    }

    fn available_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        [
            // top layer
            Move(Dir::UBR, CornerTwistAmt::Cw),
            Move(Dir::UBR, CornerTwistAmt::Ccw),
            Move(Dir::UFL, CornerTwistAmt::Cw),
            Move(Dir::UFL, CornerTwistAmt::Ccw),
            // bottom front
            Move(Dir::DFR, CornerTwistAmt::Cw),
            Move(Dir::DFR, CornerTwistAmt::Ccw),
            // bottom back
            Move(Dir::DBL, CornerTwistAmt::Cw),
            Move(Dir::DBL, CornerTwistAmt::Ccw),
        ]
    }

    fn is_redundant(last_move: Self::Move, next_move: Self::Move) -> bool {
        // none of the twists commute with each other so there's nothing going on here
        last_move.0 == next_move.0
    }

    fn apply(&self, m: Self::Move) -> Self {
        match (m.0, m.1) {
            (Dir::UBR, CornerTwistAmt::Cw) => self.ubr(),
            (Dir::UBR, CornerTwistAmt::Ccw) => self.ubr().ubr(),
            (Dir::UFL, CornerTwistAmt::Cw) => self.ufl(),
            (Dir::UFL, CornerTwistAmt::Ccw) => self.ufl().ufl(),
            (Dir::DFR, CornerTwistAmt::Cw) => self.dfr(),
            (Dir::DFR, CornerTwistAmt::Ccw) => self.dfr().dfr(),
            (Dir::DBL, CornerTwistAmt::Cw) => self.dbl(),
            (Dir::DBL, CornerTwistAmt::Ccw) => self.dbl().dbl(),
        }
    }

    fn max_fuel() -> usize {
        11
    }
}

impl SimpleStartState for IvyCube {
    type UniqueKey = PackedBits;

    fn start() -> Self {
        Self::solved_state()
    }

    fn uniq_key(&self) -> Self::UniqueKey {
        let mut out: PackedBits = 0;

        self.corners.pack(&mut out);
        self.centers.pack(&mut out);

        out
    }
}

pub fn make_heuristic() -> impl Heuristic<IvyCube> {
    // max depth is picked to keep the compute time low
    bounded_cache::<IvyCube>(6)
}

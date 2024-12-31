use derive_more::Display;
use enum_iterator::{all, Sequence};
use rand::Rng;

use crate::cubesearch::SimpleStartState;
use crate::idasearch::heuristic_helpers::bounded_cache;
use crate::idasearch::{Heuristic, Solvable};
use crate::moves::{CanReverse, CubeMoveAmt};
use crate::random_helpers;
use crate::scrambles::RandomInit;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Sequence)]
enum CornerCubelet {
    UFL,
    UFR,
    UBL,
    UBR,
    DFL,
    DFR,
    DBL,
    DBR,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct SquareZero {
    // eight corners
    ufl: CornerCubelet,
    ufr: CornerCubelet,
    ubl: CornerCubelet,
    ubr: CornerCubelet,
    dfl: CornerCubelet,
    dfr: CornerCubelet,
    dbl: CornerCubelet,
    dbr: CornerCubelet,

    // three movable centers
    middle_flipped: bool, // true is flipped, false is normal
}

impl SquareZero {
    #[inline(always)]
    fn solved() -> Self {
        Self {
            ufl: CornerCubelet::UFL,
            ufr: CornerCubelet::UFR,
            ubl: CornerCubelet::UBL,
            ubr: CornerCubelet::UBR,
            dfl: CornerCubelet::DFL,
            dfr: CornerCubelet::DFR,
            dbl: CornerCubelet::DBL,
            dbr: CornerCubelet::DBR,
            middle_flipped: false,
        }
    }

    #[inline(always)]
    fn u(&self) -> Self {
        Self {
            ufl: self.ufr,
            ufr: self.ubr,
            ubr: self.ubl,
            ubl: self.ufl,
            ..*self
        }
    }

    #[inline(always)]
    fn d(&self) -> Self {
        Self {
            dfl: self.dbl,
            dbl: self.dbr,
            dbr: self.dfr,
            dfr: self.dfl,
            ..*self
        }
    }

    #[inline(always)]
    fn r2(&self) -> Self {
        Self {
            // cycle right corners
            dfr: self.ubr,
            ubr: self.dfr,
            dbr: self.ufr,
            ufr: self.dbr,
            // flip the middle
            middle_flipped: !self.middle_flipped,
            // left side unchanged
            ..*self
        }
    }
}

impl SimpleStartState for SquareZero {
    type UniqueKey = Self;

    fn start() -> Self {
        Self::solved()
    }

    fn uniq_key(&self) -> Self::UniqueKey {
        *self
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Hash, Sequence)]
pub enum Move {
    R2,
    // But U and D can do any of the usual four amounts
    #[display(fmt = "U{}", _0)]
    U(CubeMoveAmt),
    #[display(fmt = "D{}", _0)]
    D(CubeMoveAmt),
}

impl CanReverse for Move {
    fn reverse(&self) -> Self {
        match self {
            Move::R2 => Move::R2,
            Move::U(amt) => Move::U(amt.reverse()),
            Move::D(amt) => Move::D(amt.reverse()),
        }
    }
}

impl Solvable for SquareZero {
    type Move = Move;

    fn is_solved(&self) -> bool {
        self == &Self::solved()
    }

    fn available_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        [
            Move::R2,
            Move::U(CubeMoveAmt::One),
            Move::U(CubeMoveAmt::Two),
            Move::U(CubeMoveAmt::Rev),
            Move::D(CubeMoveAmt::One),
            Move::D(CubeMoveAmt::Two),
            Move::D(CubeMoveAmt::Rev),
        ]
    }

    fn is_redundant(last_move: Self::Move, next_move: Self::Move) -> bool {
        match last_move {
            Move::R2 => next_move == Move::R2,
            Move::U(_) => matches!(next_move, Move::U(_)),
            Move::D(_) => matches!(next_move, Move::D(_)),
        }
    }

    fn apply(&self, m: Self::Move) -> Self {
        match m {
            Move::R2 => self.r2(),
            Move::U(amt) => match amt {
                CubeMoveAmt::One => self.u(),
                CubeMoveAmt::Two => self.u().u(),
                CubeMoveAmt::Rev => self.u().u().u(),
            },
            Move::D(amt) => match amt {
                CubeMoveAmt::One => self.d(),
                CubeMoveAmt::Two => self.d().d(),
                CubeMoveAmt::Rev => self.d().d().d(),
            },
        }
    }

    fn max_fuel() -> usize {
        14
    }
}

impl RandomInit for SquareZero {
    fn random_state<R: Rng>(r: &mut R) -> Self {
        // any permutation is fine
        let (corners, _) = random_helpers::shuffle_any(r, all::<CornerCubelet>());

        Self {
            ufl: corners[0],
            ufr: corners[1],
            ubl: corners[2],
            ubr: corners[3],
            dfl: corners[4],
            dfr: corners[5],
            dbl: corners[6],
            dbr: corners[7],
            middle_flipped: r.gen(),
        }
    }
}

pub fn make_heuristic() -> impl Heuristic<SquareZero> {
    bounded_cache::<SquareZero>(8)
}

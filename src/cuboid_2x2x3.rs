use derive_more::Display;
use enum_iterator::{all, Sequence};
use rand::Rng;

use crate::cubesearch::SimpleStartState;
use crate::idasearch::Solvable;
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

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Sequence)]
enum CenterCubelet {
    // we fix the BL center cubelet, so we don't need it here
    FL,
    FR,
    BR,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Cuboid2x2x3 {
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
    flc: CenterCubelet,
    frc: CenterCubelet,
    brc: CenterCubelet,
}

impl Cuboid2x2x3 {
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
            flc: CenterCubelet::FL,
            frc: CenterCubelet::FR,
            brc: CenterCubelet::BR,
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
    fn f2(&self) -> Self {
        Self {
            // cycle front corners
            dfl: self.ufr,
            ufr: self.dfl,
            dfr: self.ufl,
            ufl: self.dfr,
            // mess with front two centers
            flc: self.frc,
            frc: self.flc,
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
            // mess with right two centers
            frc: self.brc,
            brc: self.frc,
            ..*self
        }
    }
}

impl SimpleStartState for Cuboid2x2x3 {
    fn start() -> Self {
        Self::solved()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Hash, Sequence)]
pub enum Move {
    // R and F can only go 2
    R2,
    F2,
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
            Move::F2 => Move::F2,
            Move::U(amt) => Move::U(amt.reverse()),
            Move::D(amt) => Move::D(amt.reverse()),
        }
    }
}

impl Solvable for Cuboid2x2x3 {
    type Move = Move;

    fn is_solved(&self) -> bool {
        self == &Self::solved()
    }

    fn available_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        [
            Move::R2,
            Move::F2,
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
            Move::F2 => next_move == Move::F2,
            Move::U(_) => matches!(next_move, Move::U(_)),
            Move::D(_) => matches!(next_move, Move::D(_)),
        }
    }

    fn apply(&self, m: Self::Move) -> Self {
        match m {
            Move::R2 => self.r2(),
            Move::F2 => self.f2(),
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

impl RandomInit for Cuboid2x2x3 {
    fn random_state<R: Rng>(r: &mut R) -> Self {
        // any permutation is fine
        let (corners, _) = random_helpers::shuffle_any(r, all::<CornerCubelet>());
        let (centers, _) = random_helpers::shuffle_any(r, all::<CenterCubelet>());

        Self {
            ufl: corners[0],
            ufr: corners[1],
            ubl: corners[2],
            ubr: corners[3],
            dfl: corners[4],
            dfr: corners[5],
            dbl: corners[6],
            dbr: corners[7],
            flc: centers[0],
            frc: centers[1],
            brc: centers[2],
        }
    }
}

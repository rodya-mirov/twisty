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
    // DBL is fixed; everything else can move
    UFL,
    UFR,
    UBL,
    UBR,
    DFL,
    DFR,
    DBR,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Sequence)]
enum EdgeCubelet {
    UF,
    UL,
    UB,
    UR,
    DF,
    DL,
    DB,
    DR,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Sequence)]
enum CenterCubelet {
    U,
    D,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Cuboid2x3x3 {
    // seven corners (dbl fixed)
    ufl: CornerCubelet,
    ufr: CornerCubelet,
    ubl: CornerCubelet,
    ubr: CornerCubelet,
    dfl: CornerCubelet,
    dfr: CornerCubelet,
    dbr: CornerCubelet,

    // eight edge pieces
    uf: EdgeCubelet,
    ur: EdgeCubelet,
    ub: EdgeCubelet,
    ul: EdgeCubelet,
    df: EdgeCubelet,
    dr: EdgeCubelet,
    db: EdgeCubelet,
    dl: EdgeCubelet,

    // two movable centers
    uc: CenterCubelet,
    dc: CenterCubelet,
}

impl Cuboid2x3x3 {
    #[inline(always)]
    fn solved() -> Self {
        Self {
            // corners
            ufl: CornerCubelet::UFL,
            ufr: CornerCubelet::UFR,
            ubl: CornerCubelet::UBL,
            ubr: CornerCubelet::UBR,
            dfl: CornerCubelet::DFL,
            dfr: CornerCubelet::DFR,
            dbr: CornerCubelet::DBR,
            // edges
            uf: EdgeCubelet::UF,
            ul: EdgeCubelet::UL,
            ub: EdgeCubelet::UB,
            ur: EdgeCubelet::UR,
            df: EdgeCubelet::DF,
            dl: EdgeCubelet::DL,
            db: EdgeCubelet::DB,
            dr: EdgeCubelet::DR,
            // centers
            uc: CenterCubelet::U,
            dc: CenterCubelet::D,
        }
    }

    #[inline(always)]
    fn u(&self) -> Self {
        Self {
            // cycle edges
            uf: self.ur,
            ur: self.ub,
            ub: self.ul,
            ul: self.uf,

            // cycle corners
            ufl: self.ufr,
            ufr: self.ubr,
            ubr: self.ubl,
            ubl: self.ufl,

            // else same
            ..*self
        }
    }

    #[inline(always)]
    fn r2(&self) -> Self {
        Self {
            // swap corners, a little bit
            ufr: self.dbr,
            dbr: self.ufr,
            ubr: self.dfr,
            dfr: self.ubr,
            // swap edges too
            ur: self.dr,
            dr: self.ur,
            // else same
            ..*self
        }
    }

    #[inline(always)]
    fn rw2(&self) -> Self {
        Self {
            // first, everything from r2
            // swap corners, a little bit
            ufr: self.dbr,
            dbr: self.ufr,
            ubr: self.dfr,
            dfr: self.ubr,
            // swap edges too
            ur: self.dr,
            dr: self.ur,
            // then the same ideas, but in the M column
            uf: self.db,
            db: self.uf,
            ub: self.df,
            df: self.ub,
            uc: self.dc,
            dc: self.uc,
            // some things stay the same, but not many
            ..*self
        }
    }

    #[inline(always)]
    fn f2(&self) -> Self {
        Self {
            // swap some corners
            ufl: self.dfr,
            dfr: self.ufl,
            ufr: self.dfl,
            dfl: self.ufr,
            // edges, too
            uf: self.df,
            df: self.uf,
            // some stuff doesn't move
            ..*self
        }
    }

    #[inline(always)]
    fn fw2(&self) -> Self {
        Self {
            // everything from f2
            // swap some corners
            ufl: self.dfr,
            dfr: self.ufl,
            ufr: self.dfl,
            dfl: self.ufr,
            // edges, too
            uf: self.df,
            df: self.uf,
            // then the center row
            ul: self.dr,
            dr: self.ul,
            dl: self.ur,
            ur: self.dl,
            uc: self.dc,
            dc: self.uc,
            // a few things stay still, but not many
            ..*self
        }
    }
}

impl SimpleStartState for Cuboid2x3x3 {
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
    // R and F can only go 2
    Rw2,
    R2,
    Fw2,
    F2,
    // But U can do any of the usual four amounts
    #[display(fmt = "U{}", _0)]
    U(CubeMoveAmt),
}

impl CanReverse for Move {
    fn reverse(&self) -> Self {
        match self {
            Move::Rw2 => Move::Rw2,
            Move::R2 => Move::R2,
            Move::Fw2 => Move::Fw2,
            Move::F2 => Move::F2,
            Move::U(amt) => Move::U(amt.reverse()),
        }
    }
}

impl Solvable for Cuboid2x3x3 {
    type Move = Move;

    fn is_solved(&self) -> bool {
        self == &Self::solved()
    }

    fn available_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        [
            Move::Rw2,
            Move::R2,
            Move::Fw2,
            Move::F2,
            Move::U(CubeMoveAmt::One),
            Move::U(CubeMoveAmt::Two),
            Move::U(CubeMoveAmt::Rev),
        ]
    }

    fn is_redundant(last_move: Self::Move, next_move: Self::Move) -> bool {
        match last_move {
            Move::Rw2 => next_move == Move::R2 || next_move == Move::Rw2,
            Move::R2 => next_move == Move::R2,
            Move::Fw2 => next_move == Move::F2 || next_move == Move::Fw2,
            Move::F2 => next_move == Move::F2,
            Move::U(_) => matches!(next_move, Move::U(_)),
        }
    }

    fn apply(&self, m: Self::Move) -> Self {
        match m {
            Move::R2 => self.r2(),
            Move::Rw2 => self.rw2(),
            Move::F2 => self.f2(),
            Move::Fw2 => self.fw2(),
            Move::U(amt) => match amt {
                CubeMoveAmt::One => self.u(),
                CubeMoveAmt::Two => self.u().u(),
                CubeMoveAmt::Rev => self.u().u().u(),
            },
        }
    }

    fn max_fuel() -> usize {
        30
    }
}

impl RandomInit for Cuboid2x3x3 {
    fn random_state<R: Rng>(r: &mut R) -> Self {
        // any permutation is fine
        let (corners, _) = random_helpers::shuffle_any(r, all::<CornerCubelet>());
        let (edges, _) = random_helpers::shuffle_any(r, all::<EdgeCubelet>());
        let (centers, _) = random_helpers::shuffle_any(r, all::<CenterCubelet>());

        Self {
            ufl: corners[0],
            ufr: corners[1],
            ubl: corners[2],
            ubr: corners[3],
            dfl: corners[4],
            dfr: corners[5],
            dbr: corners[6],
            uf: edges[0],
            ur: edges[1],
            ub: edges[2],
            ul: edges[3],
            df: edges[4],
            dr: edges[5],
            db: edges[6],
            dl: edges[7],
            uc: centers[0],
            dc: centers[1],
        }
    }
}

pub fn make_heuristic() -> impl Heuristic<Cuboid2x3x3> {
    bounded_cache::<Cuboid2x3x3>(8)
}

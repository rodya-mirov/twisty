use derive_more::Display;
use enum_iterator::{all, Sequence};
use rand::Rng;

use crate::cubesearch::SimpleStartState;
use crate::idasearch::heuristic_helpers::bounded_cache;
use crate::idasearch::{Heuristic, Solvable};
use crate::moves::{CanReverse, CubeMoveAmt};
use crate::random_helpers::shuffle_any;
use crate::scrambles::RandomInit;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Sequence)]
#[repr(u8)]
enum OuterCornerCubelet {
    // DBL is fixed; everything else can move
    UFL,
    UFR,
    UBL,
    UBR,
    DFL,
    DFR,
    DBR,
}

impl OuterCornerCubelet {
    fn pack(self, source: &mut u64) {
        *source = (*source << 3) + (self as u64);
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Sequence)]
#[repr(u8)]
enum InnerCornerCubelet {
    UFL,
    UFR,
    UBL,
    UBR,
    DFL,
    DFR,
    DBL,
    DBR,
}

impl InnerCornerCubelet {
    fn pack(self, source: &mut u64) {
        *source = (*source << 3) + (self as u64);
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Sequence)]
#[repr(u8)]
enum OuterEdgeCubelet {
    UF,
    UL,
    UB,
    UR,
    DF,
    DL,
    DB,
    DR,
}

impl OuterEdgeCubelet {
    fn pack(self, source: &mut u64) {
        *source = (*source << 3) + (self as u64);
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Sequence)]
#[repr(u8)]
enum OuterCenterCubelet {
    U,
    D,
}

impl OuterCenterCubelet {
    fn pack(self, source: &mut u64) {
        *source = (*source << 1) + (self as u64);
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Sequence)]
#[repr(u8)]
enum InnerCenterCubelet {
    F,
    L,
    R,
    B,
}

impl InnerCenterCubelet {
    fn pack(self, source: &mut u64) {
        *source = (*source << 2) + (self as u64);
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct OuterCorners {
    // seven extreme corners (dbl fixed)
    ufl: OuterCornerCubelet,
    ufr: OuterCornerCubelet,
    ubl: OuterCornerCubelet,
    ubr: OuterCornerCubelet,
    dfl: OuterCornerCubelet,
    dfr: OuterCornerCubelet,
    dbr: OuterCornerCubelet,

    // two movable centers, toss em in here I guess
    uc: OuterCenterCubelet,
    dc: OuterCenterCubelet,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct OuterEdges {
    // eight extreme edge pieces
    uf: OuterEdgeCubelet,
    ur: OuterEdgeCubelet,
    ub: OuterEdgeCubelet,
    ul: OuterEdgeCubelet,
    df: OuterEdgeCubelet,
    dr: OuterEdgeCubelet,
    db: OuterEdgeCubelet,
    dl: OuterEdgeCubelet,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct OuterCuboid3x3x4 {
    corners: OuterCorners,
    edges: OuterEdges,
}

impl OuterCorners {
    fn solved() -> Self {
        Self {
            // 7 outer corners (dbl fixed)
            ufl: OuterCornerCubelet::UFL,
            ufr: OuterCornerCubelet::UFR,
            ubl: OuterCornerCubelet::UBL,
            ubr: OuterCornerCubelet::UBR,
            dfl: OuterCornerCubelet::DFL,
            dfr: OuterCornerCubelet::DFR,
            dbr: OuterCornerCubelet::DBR,
            // centers
            uc: OuterCenterCubelet::U,
            dc: OuterCenterCubelet::D,
        }
    }

    #[inline(always)]
    fn pack(&self, out: &mut u64) {
        // pack 6 corners; last is determined
        self.ufl.pack(out);
        self.ufr.pack(out);
        self.ubl.pack(out);
        self.ubr.pack(out);
        self.dfl.pack(out);
        self.dfr.pack(out);

        // pack 1 center; other is determined
        self.uc.pack(out);

        // note 6 * 3 + 1 * 1 == 19
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
    fn uw(&self) -> Self {
        // uw and uww are same as u, for outer
        self.u()
    }

    #[inline(always)]
    fn uww(&self) -> Self {
        // uw and uww are same as u, for outer
        self.u()
    }

    #[inline(always)]
    fn r2(&self) -> Self {
        Self {
            // corner, two pairs of swaps
            ufr: self.dbr,
            dbr: self.ufr,
            ubr: self.dfr,
            dfr: self.ubr,

            // else same
            ..*self
        }
    }

    #[inline(always)]
    fn rw2(&self) -> Self {
        Self {
            // everything from r2
            // corner, two pairs of swaps
            ufr: self.dbr,
            dbr: self.ufr,
            ubr: self.dfr,
            dfr: self.ubr,

            // then middle slice, which is similar
            // center swap
            uc: self.dc,
            dc: self.uc,

            // else same
            ..*self
        }
    }

    #[inline(always)]
    fn f2(&self) -> Self {
        Self {
            // front slice
            ufl: self.dfr,
            dfr: self.ufl,
            ufr: self.dfl,
            dfl: self.ufr,
            ..*self
        }
    }

    #[inline(always)]
    fn fw2(&self) -> Self {
        Self {
            // front slice
            ufl: self.dfr,
            dfr: self.ufl,
            ufr: self.dfl,
            dfl: self.ufr,
            // mid slice
            uc: self.dc,
            dc: self.uc,
            ..*self
        }
    }
}

impl OuterEdges {
    fn solved() -> Self {
        Self {
            // 8 outer edges
            uf: OuterEdgeCubelet::UF,
            ur: OuterEdgeCubelet::UR,
            ub: OuterEdgeCubelet::UB,
            ul: OuterEdgeCubelet::UL,
            df: OuterEdgeCubelet::DF,
            dr: OuterEdgeCubelet::DR,
            db: OuterEdgeCubelet::DB,
            dl: OuterEdgeCubelet::DL,
        }
    }

    #[inline(always)]
    fn pack(&self, out: &mut u64) {
        // pack 7 edges; last is determined
        self.uf.pack(out);
        self.ul.pack(out);
        self.ur.pack(out);
        self.ub.pack(out);
        self.df.pack(out);
        self.dl.pack(out);
        self.dr.pack(out);
    }

    #[inline(always)]
    fn u(&self) -> Self {
        Self {
            uf: self.ur,
            ur: self.ub,
            ub: self.ul,
            ul: self.uf,
            ..*self
        }
    }

    #[inline(always)]
    fn uw(&self) -> Self {
        // uw and uww are same as u, for outer
        self.u()
    }

    #[inline(always)]
    fn uww(&self) -> Self {
        // uw and uww are same as u, for outer
        self.u()
    }

    #[inline(always)]
    fn r2(&self) -> Self {
        Self {
            // u/d r center swap
            ur: self.dr,
            dr: self.ur,

            // else same
            ..*self
        }
    }

    #[inline(always)]
    fn rw2(&self) -> Self {
        Self {
            // everything from r2
            // u/d r center swap
            ur: self.dr,
            dr: self.ur,

            // then middle slice, which is similar

            // f/b edge swap
            uf: self.db,
            db: self.uf,
            ub: self.df,
            df: self.ub,

            // else same
            ..*self
        }
    }

    #[inline(always)]
    fn f2(&self) -> Self {
        Self {
            // front slice
            uf: self.df,
            df: self.uf,
            ..*self
        }
    }

    #[inline(always)]
    fn fw2(&self) -> Self {
        Self {
            // front slice
            uf: self.df,
            df: self.uf,
            // mid slice
            ul: self.dr,
            dr: self.ul,
            ur: self.dl,
            dl: self.ur,
            ..*self
        }
    }
}

impl OuterCuboid3x3x4 {
    fn solved() -> Self {
        Self {
            corners: OuterCorners::solved(),
            edges: OuterEdges::solved(),
        }
    }

    #[inline(always)]
    fn uniq_key(&self) -> u64 {
        let mut out: u64 = 0;

        self.corners.pack(&mut out);
        self.edges.pack(&mut out);

        out
    }

    #[inline(always)]
    fn u(&self) -> Self {
        Self {
            corners: self.corners.u(),
            edges: self.edges.u(),
        }
    }

    #[inline(always)]
    fn uw(&self) -> Self {
        // uw and uww are same as u, for outer
        self.u()
    }

    #[inline(always)]
    fn uww(&self) -> Self {
        // uw and uww are same as u, for outer
        self.u()
    }

    #[inline(always)]
    fn r2(&self) -> Self {
        Self {
            corners: self.corners.r2(),
            edges: self.edges.r2(),
        }
    }

    #[inline(always)]
    fn rw2(&self) -> Self {
        Self {
            corners: self.corners.rw2(),
            edges: self.edges.rw2(),
        }
    }

    #[inline(always)]
    fn f2(&self) -> Self {
        Self {
            corners: self.corners.f2(),
            edges: self.edges.f2(),
        }
    }

    #[inline(always)]
    fn fw2(&self) -> Self {
        Self {
            corners: self.corners.fw2(),
            edges: self.edges.fw2(),
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct InnerCuboid3x3x4 {
    // eight inner corners
    ufl: InnerCornerCubelet,
    ufr: InnerCornerCubelet,
    ubl: InnerCornerCubelet,
    ubr: InnerCornerCubelet,
    dfl: InnerCornerCubelet,
    dfr: InnerCornerCubelet,
    dbl: InnerCornerCubelet,
    dbr: InnerCornerCubelet,

    // eight inner center pieces
    uf: InnerCenterCubelet,
    ur: InnerCenterCubelet,
    ub: InnerCenterCubelet,
    ul: InnerCenterCubelet,
    df: InnerCenterCubelet,
    dr: InnerCenterCubelet,
    db: InnerCenterCubelet,
    dl: InnerCenterCubelet,
}

impl InnerCuboid3x3x4 {
    fn solved() -> Self {
        Self {
            ufl: InnerCornerCubelet::UFL,
            ufr: InnerCornerCubelet::UFR,
            ubl: InnerCornerCubelet::UBL,
            ubr: InnerCornerCubelet::UBR,
            dfl: InnerCornerCubelet::DFL,
            dfr: InnerCornerCubelet::DFR,
            dbl: InnerCornerCubelet::DBL,
            dbr: InnerCornerCubelet::DBR,
            uf: InnerCenterCubelet::F,
            ur: InnerCenterCubelet::R,
            ub: InnerCenterCubelet::B,
            ul: InnerCenterCubelet::L,
            df: InnerCenterCubelet::F,
            dr: InnerCenterCubelet::R,
            db: InnerCenterCubelet::B,
            dl: InnerCenterCubelet::L,
        }
    }

    #[inline(always)]
    fn uniq_key(&self) -> u64 {
        let mut out: u64 = 0;

        // pack 7 corners; last is determined
        self.ufl.pack(&mut out);
        self.ufr.pack(&mut out);
        self.ubl.pack(&mut out);
        self.ubr.pack(&mut out);
        self.dfl.pack(&mut out);
        self.dfr.pack(&mut out);
        self.dbl.pack(&mut out);

        // pack 7 edges; last is determined
        self.uf.pack(&mut out);
        self.ul.pack(&mut out);
        self.ur.pack(&mut out);
        self.ub.pack(&mut out);
        self.df.pack(&mut out);
        self.dl.pack(&mut out);
        self.dr.pack(&mut out);

        // note 7 * 3 + 7 * 3 == 42, which fits in 64 bits

        out
    }

    #[inline(always)]
    fn u(&self) -> Self {
        // u is a no-op for the inners
        *self
    }

    #[inline(always)]
    fn uw(&self) -> Self {
        Self {
            // u centers
            uf: self.ur,
            ur: self.ub,
            ub: self.ul,
            ul: self.uf,
            // u corners
            ufr: self.ubr,
            ubr: self.ubl,
            ubl: self.ufl,
            ufl: self.ufr,
            // D is fixed
            ..*self
        }
    }

    #[inline(always)]
    fn uww(&self) -> Self {
        Self {
            // u centers
            uf: self.ur,
            ur: self.ub,
            ub: self.ul,
            ul: self.uf,
            // u corners
            ufr: self.ubr,
            ubr: self.ubl,
            ubl: self.ufl,
            ufl: self.ufr,
            // d centers
            df: self.dr,
            dr: self.db,
            db: self.dl,
            dl: self.df,
            // d corners
            dfr: self.dbr,
            dbr: self.dbl,
            dbl: self.dfl,
            dfl: self.dfr,
        }
    }

    #[inline(always)]
    fn r2(&self) -> Self {
        Self {
            // right slice; three swaps
            ur: self.dr,
            dr: self.ur,
            ufr: self.dbr,
            dbr: self.ufr,
            ubr: self.dfr,
            dfr: self.ubr,
            ..*self
        }
    }

    #[inline(always)]
    fn rw2(&self) -> Self {
        Self {
            // same as r2
            ur: self.dr,
            dr: self.ur,
            ufr: self.dbr,
            dbr: self.ufr,
            ubr: self.dfr,
            dfr: self.ubr,
            // then inner slice
            uf: self.db,
            db: self.uf,
            df: self.ub,
            ub: self.df,
            // rest same
            ..*self
        }
    }

    #[inline(always)]
    fn f2(&self) -> Self {
        Self {
            uf: self.df,
            df: self.uf,
            ufl: self.dfr,
            dfr: self.ufl,
            ufr: self.dfl,
            dfl: self.ufr,
            ..*self
        }
    }

    #[inline(always)]
    fn fw2(&self) -> Self {
        Self {
            // front slice
            uf: self.df,
            df: self.uf,
            ufl: self.dfr,
            dfr: self.ufl,
            ufr: self.dfl,
            dfl: self.ufr,
            // mid slice; no core pieces so just two swaps
            ul: self.dr,
            dr: self.ul,
            ur: self.dl,
            dl: self.ur,
            // rest same
            ..*self
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Cuboid3x3x4 {
    outer: OuterCuboid3x3x4,
    inner: InnerCuboid3x3x4,
}

impl Cuboid3x3x4 {
    #[inline(always)]
    fn solved() -> Self {
        Self {
            inner: InnerCuboid3x3x4::solved(),
            outer: OuterCuboid3x3x4::solved(),
        }
    }
}

impl SimpleStartState for Cuboid3x3x4 {
    type UniqueKey = (u64, u64);

    fn start() -> Self {
        Self::solved()
    }

    fn uniq_key(&self) -> Self::UniqueKey {
        (self.inner.uniq_key(), self.outer.uniq_key())
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
    // It can also go deeper down (avoiding D or Dw)
    #[display(fmt = "U{}", _0)]
    U(CubeMoveAmt),
    #[display(fmt = "Uw{}", _0)]
    Uw(CubeMoveAmt),
    #[display(fmt = "Uww{}", _0)]
    Uww(CubeMoveAmt),
}

impl CanReverse for Move {
    fn reverse(&self) -> Self {
        match self {
            Move::Rw2 => Move::Rw2,
            Move::R2 => Move::R2,
            Move::Fw2 => Move::Fw2,
            Move::F2 => Move::F2,
            Move::U(amt) => Move::U(amt.reverse()),
            Move::Uw(amt) => Move::Uw(amt.reverse()),
            Move::Uww(amt) => Move::Uww(amt.reverse()),
        }
    }
}

impl Solvable for InnerCuboid3x3x4 {
    type Move = Move;

    fn is_solved(&self) -> bool {
        self == &InnerCuboid3x3x4::solved()
    }

    fn available_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        [
            Move::R2,
            Move::Rw2,
            Move::F2,
            Move::Fw2,
            // don't bother emitting U, it's a no-op
            Move::Uw(CubeMoveAmt::One),
            Move::Uw(CubeMoveAmt::Two),
            Move::Uw(CubeMoveAmt::Rev),
            Move::Uww(CubeMoveAmt::One),
            Move::Uww(CubeMoveAmt::Two),
            Move::Uww(CubeMoveAmt::Rev),
        ]
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
            Move::Uw(amt) => match amt {
                CubeMoveAmt::One => self.uw(),
                CubeMoveAmt::Two => self.uw().uw(),
                CubeMoveAmt::Rev => self.uw().uw().uw(),
            },
            Move::Uww(amt) => match amt {
                CubeMoveAmt::One => self.uww(),
                CubeMoveAmt::Two => self.uww().uww(),
                CubeMoveAmt::Rev => self.uww().uww().uww(),
            },
        }
    }

    fn max_fuel() -> usize {
        13 // ???
    }
}

impl Solvable for OuterCuboid3x3x4 {
    type Move = Move;

    fn is_solved(&self) -> bool {
        self == &OuterCuboid3x3x4::solved()
    }

    fn available_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        [
            Move::R2,
            Move::Rw2,
            Move::F2,
            Move::Fw2,
            // don't bother emitting Uw or Uww, they're the same as U
            Move::U(CubeMoveAmt::One),
            Move::U(CubeMoveAmt::Two),
            Move::U(CubeMoveAmt::Rev),
        ]
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
            Move::Uw(amt) => match amt {
                CubeMoveAmt::One => self.uw(),
                CubeMoveAmt::Two => self.uw().uw(),
                CubeMoveAmt::Rev => self.uw().uw().uw(),
            },
            Move::Uww(amt) => match amt {
                CubeMoveAmt::One => self.uww(),
                CubeMoveAmt::Two => self.uww().uww(),
                CubeMoveAmt::Rev => self.uww().uww().uww(),
            },
        }
    }

    fn max_fuel() -> usize {
        13 // ???
    }
}

impl Solvable for OuterCorners {
    type Move = Move;

    fn is_solved(&self) -> bool {
        self == &Self::solved()
    }

    fn available_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        [
            Move::R2,
            Move::Rw2,
            Move::F2,
            Move::Fw2,
            // don't bother emitting Uw or Uww, they're the same as U
            Move::U(CubeMoveAmt::One),
            Move::U(CubeMoveAmt::Two),
            Move::U(CubeMoveAmt::Rev),
        ]
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
            Move::Uw(amt) => match amt {
                CubeMoveAmt::One => self.uw(),
                CubeMoveAmt::Two => self.uw().uw(),
                CubeMoveAmt::Rev => self.uw().uw().uw(),
            },
            Move::Uww(amt) => match amt {
                CubeMoveAmt::One => self.uww(),
                CubeMoveAmt::Two => self.uww().uww(),
                CubeMoveAmt::Rev => self.uww().uww().uww(),
            },
        }
    }

    fn max_fuel() -> usize {
        14 // ???
    }
}

impl Solvable for OuterEdges {
    type Move = Move;

    fn is_solved(&self) -> bool {
        self == &Self::solved()
    }

    fn available_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        [
            Move::R2,
            Move::Rw2,
            Move::F2,
            Move::Fw2,
            // don't bother emitting Uw or Uww, they're the same as U
            Move::U(CubeMoveAmt::One),
            Move::U(CubeMoveAmt::Two),
            Move::U(CubeMoveAmt::Rev),
        ]
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
            Move::Uw(amt) => match amt {
                CubeMoveAmt::One => self.uw(),
                CubeMoveAmt::Two => self.uw().uw(),
                CubeMoveAmt::Rev => self.uw().uw().uw(),
            },
            Move::Uww(amt) => match amt {
                CubeMoveAmt::One => self.uww(),
                CubeMoveAmt::Two => self.uww().uww(),
                CubeMoveAmt::Rev => self.uww().uww().uww(),
            },
        }
    }

    fn max_fuel() -> usize {
        14 // ???
    }
}

impl SimpleStartState for InnerCuboid3x3x4 {
    type UniqueKey = u64;

    fn start() -> Self {
        InnerCuboid3x3x4::solved()
    }

    fn uniq_key(&self) -> Self::UniqueKey {
        InnerCuboid3x3x4::uniq_key(self)
    }
}

impl SimpleStartState for OuterCuboid3x3x4 {
    type UniqueKey = u64;

    fn start() -> Self {
        OuterCuboid3x3x4::solved()
    }

    fn uniq_key(&self) -> Self::UniqueKey {
        OuterCuboid3x3x4::uniq_key(self)
    }
}

impl SimpleStartState for OuterCorners {
    type UniqueKey = u64;

    fn start() -> Self {
        Self::solved()
    }

    fn uniq_key(&self) -> Self::UniqueKey {
        let mut out: u64 = 0;
        self.pack(&mut out);
        out
    }
}

impl SimpleStartState for OuterEdges {
    type UniqueKey = u64;

    fn start() -> Self {
        Self::solved()
    }

    fn uniq_key(&self) -> Self::UniqueKey {
        let mut out: u64 = 0;
        self.pack(&mut out);
        out
    }
}

impl Solvable for Cuboid3x3x4 {
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
            Move::Uw(CubeMoveAmt::One),
            Move::Uw(CubeMoveAmt::Two),
            Move::Uw(CubeMoveAmt::Rev),
            Move::Uww(CubeMoveAmt::One),
            Move::Uww(CubeMoveAmt::Two),
            Move::Uww(CubeMoveAmt::Rev),
        ]
    }

    fn is_redundant(last_move: Self::Move, next_move: Self::Move) -> bool {
        match last_move {
            Move::Rw2 => next_move == Move::R2 || next_move == Move::Rw2,
            Move::R2 => next_move == Move::R2,
            Move::Fw2 => next_move == Move::F2 || next_move == Move::Fw2,
            Move::F2 => next_move == Move::F2,
            Move::U(_) => matches!(next_move, Move::U(_)),
            Move::Uw(_) => matches!(next_move, Move::Uw(_) | Move::U(_)),
            Move::Uww(_) => matches!(next_move, Move::Uww(_) | Move::Uw(_) | Move::U(_)),
        }
    }

    fn apply(&self, m: Self::Move) -> Self {
        Self {
            inner: self.inner.apply(m),
            outer: self.outer.apply(m),
        }
    }

    fn max_fuel() -> usize {
        19 // ???
    }
}

impl RandomInit for InnerCuboid3x3x4 {
    fn random_state<R: Rng>(r: &mut R) -> Self {
        // literally any permutation is fine; the indistinguishable centers eat the parity problems
        let (corners, _) = shuffle_any(r, all::<InnerCornerCubelet>());
        let (centers, _) = shuffle_any(
            r,
            [
                InnerCenterCubelet::F,
                InnerCenterCubelet::F,
                InnerCenterCubelet::R,
                InnerCenterCubelet::R,
                InnerCenterCubelet::L,
                InnerCenterCubelet::L,
                InnerCenterCubelet::B,
                InnerCenterCubelet::B,
            ],
        );

        Self {
            ufl: corners[0],
            ufr: corners[1],
            ubl: corners[2],
            ubr: corners[3],
            dfl: corners[4],
            dfr: corners[5],
            dbl: corners[6],
            dbr: corners[7],
            uf: centers[0],
            ur: centers[1],
            ub: centers[2],
            ul: centers[3],
            df: centers[4],
            dr: centers[5],
            db: centers[6],
            dl: centers[7],
        }
    }
}

impl RandomInit for OuterCuboid3x3x4 {
    fn random_state<R: Rng>(r: &mut R) -> Self {
        let (corners, _) = shuffle_any(r, all::<OuterCornerCubelet>());
        let (edges, _) = shuffle_any(r, all::<OuterEdgeCubelet>());
        let (centers, _) = shuffle_any(r, all::<OuterCenterCubelet>());

        let corners = OuterCorners {
            ufl: corners[0],
            ufr: corners[1],
            ubl: corners[2],
            ubr: corners[3],
            dfl: corners[4],
            dfr: corners[5],
            dbr: corners[6],
            uc: centers[0],
            dc: centers[1],
        };

        let edges = OuterEdges {
            uf: edges[0],
            ur: edges[1],
            ub: edges[2],
            ul: edges[3],
            df: edges[4],
            dr: edges[5],
            db: edges[6],
            dl: edges[7],
        };

        Self { edges, corners }
    }
}

impl RandomInit for Cuboid3x3x4 {
    fn random_state<R: Rng>(r: &mut R) -> Self {
        Self {
            inner: InnerCuboid3x3x4::random_state(r),
            outer: OuterCuboid3x3x4::random_state(r),
        }
    }
}

pub fn make_heuristic() -> impl Heuristic<Cuboid3x3x4> {
    // the outer and inner layers have 406M states, which is more than i want to put in a cache
    // may want to ... make the corners? or something?
    // might need a thistlethwaite-type algorithm to reasonably solve this (reduce to a subgroup)
    let outer = bounded_cache::<OuterCuboid3x3x4>(11);
    let inner = bounded_cache::<InnerCuboid3x3x4>(11);

    // interestingly, increasing the depth here actually slows down the solve, even if you
    // ignore the extra time making the heuristic
    let total = bounded_cache::<Cuboid3x3x4>(7);

    // experimentally: we can add a perfect cache for the corners / edges individually, but
    // evaluating the cache takes more time than the additional information saves

    move |state: &Cuboid3x3x4| {
        let o = outer.estimated_remaining_cost(&state.outer);
        let i = inner.estimated_remaining_cost(&state.inner);
        let t = total.estimated_remaining_cost(state);

        o.max(i).max(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_corner_cubelets_fit_in_space() {
        for c in all::<OuterCornerCubelet>() {
            assert!((c as u8) < 8);
        }
        for c in all::<InnerCornerCubelet>() {
            assert!((c as u8) < 8);
        }
    }

    #[test]
    fn ensure_edge_cubelets_fit_in_space() {
        for c in all::<OuterEdgeCubelet>() {
            assert!((c as u8) < 8);
        }
    }

    #[test]
    fn ensure_center_cubelets_fit_in_space() {
        for c in all::<OuterCenterCubelet>() {
            assert!((c as u8) < 2);
        }
        for c in all::<InnerCenterCubelet>() {
            assert!((c as u8) < 4);
        }
    }

    #[test]
    fn ensure_parity_alg_works() {
        let mut cube = Cuboid3x3x4::solved();

        // standard parity problem -- swap two inner edges (front-right and back-right)
        std::mem::swap(&mut cube.inner.ufr, &mut cube.inner.dbr);
        std::mem::swap(&mut cube.inner.ubr, &mut cube.inner.dfr);

        // try Uw2 F2 R2 Uw2 U2 R2 F2 Uw2 which swaps right two edges, should make us solved

        let moves: Vec<Move> = vec![
            Move::Uw(CubeMoveAmt::Two),
            Move::F2,
            Move::R2,
            Move::Uw(CubeMoveAmt::Two),
            Move::U(CubeMoveAmt::Two),
            Move::R2,
            Move::F2,
            Move::Uw(CubeMoveAmt::Two),
        ];

        for m in moves.iter().copied() {
            cube = cube.apply(m);
        }

        assert!(cube.is_solved());
    }
}

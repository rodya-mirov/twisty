use derive_more::Display;
use enum_iterator::Sequence;
use rand::Rng;

use crate::cubesearch::SimpleStartState;
use crate::idasearch::heuristic_helpers::bounded_cache;
use crate::idasearch::{Heuristic, Solvable};
use crate::moves::{CanReverse, CubeMoveAmt};
use crate::orientations::{CornerOrientation, EdgeOrientation};
use crate::scrambles::RandomInit;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Sequence)]
#[repr(u8)]
enum CornerCubelet {
    // DFL, DBL is fixed; everything else can move
    UFL,
    UFR,
    UBL,
    UBR,
    DFR,
    DBR,
}

impl CornerCubelet {
    fn pack(self, source: &mut u64) {
        *source = (*source << 3) + (self as u64);
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Sequence)]
#[repr(u8)]
enum EdgeCubelet {
    // DL is fixed; everything else can move
    UF,
    UL,
    UB,
    UR,
    FR,
    BR,
    DF,
    DB,
    DR,
}

impl EdgeCubelet {
    fn pack(self, source: &mut u64) {
        *source = (*source << 3) + (self as u64);
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Sequence)]
#[repr(u8)]
enum CenterCubelet {
    // we need little-r moves, so the U/D/F/B centers can get messed up
    U,
    D,
    F,
    B,
}

impl CenterCubelet {
    fn pack(self, source: &mut u64) {
        *source = (*source << 2) + (self as u64);
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct PositionState {
    // six corners (dbl, dfl fixed)
    ufl: CornerCubelet,
    ufr: CornerCubelet,
    ubl: CornerCubelet,
    ubr: CornerCubelet,
    dfr: CornerCubelet,
    dbr: CornerCubelet,

    // nine edge pieces (fl, br, dl fixed)
    uf: EdgeCubelet,
    ur: EdgeCubelet,
    ub: EdgeCubelet,
    ul: EdgeCubelet,
    fr: EdgeCubelet,
    br: EdgeCubelet,
    df: EdgeCubelet,
    dr: EdgeCubelet,
    db: EdgeCubelet,

    // four movable centers
    uc: CenterCubelet,
    dc: CenterCubelet,
    fc: CenterCubelet,
    bc: CenterCubelet,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Default)]
struct OrientationState {
    // six corners (dbl, dfl fixed)
    ufl: CornerOrientation,
    ufr: CornerOrientation,
    ubl: CornerOrientation,
    ubr: CornerOrientation,
    dfr: CornerOrientation,
    dbr: CornerOrientation,

    // nine edge pieces (fl, bl, dl fixed)
    uf: EdgeOrientation,
    ur: EdgeOrientation,
    ub: EdgeOrientation,
    ul: EdgeOrientation,
    fr: EdgeOrientation,
    br: EdgeOrientation,
    df: EdgeOrientation,
    dr: EdgeOrientation,
    db: EdgeOrientation,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Bandaged3x3x3with1x2x3 {
    pos: PositionState,
    orr: OrientationState,
}

impl PositionState {
    #[inline(always)]
    fn solved() -> Self {
        Self {
            // corners
            ufl: CornerCubelet::UFL,
            ufr: CornerCubelet::UFR,
            ubl: CornerCubelet::UBL,
            ubr: CornerCubelet::UBR,
            dfr: CornerCubelet::DFR,
            dbr: CornerCubelet::DBR,
            // edges
            uf: EdgeCubelet::UF,
            ul: EdgeCubelet::UL,
            ub: EdgeCubelet::UB,
            ur: EdgeCubelet::UR,
            fr: EdgeCubelet::FR,
            br: EdgeCubelet::BR,
            df: EdgeCubelet::DF,
            db: EdgeCubelet::DB,
            dr: EdgeCubelet::DR,
            // centers
            uc: CenterCubelet::U,
            dc: CenterCubelet::D,
            fc: CenterCubelet::F,
            bc: CenterCubelet::B,
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

    // basically Rw R'
    #[inline(always)]
    fn m(&self) -> Self {
        Self {
            // M layer edges permute
            uf: self.df,
            df: self.db,
            db: self.ub,
            ub: self.uf,

            // centers permute
            uc: self.fc,
            fc: self.dc,
            dc: self.bc,
            bc: self.uc,

            // everything else same
            ..*self
        }
    }

    #[inline(always)]
    fn r(&self) -> Self {
        Self {
            // R face corners permute
            ufr: self.dfr,
            dfr: self.dbr,
            dbr: self.ubr,
            ubr: self.ufr,
            // R face edges permute
            ur: self.fr,
            fr: self.dr,
            dr: self.br,
            br: self.ur,
            // everything else same
            ..*self
        }
    }

    #[inline(always)]
    fn rw(&self) -> Self {
        self.r().m()
    }
}

impl OrientationState {
    #[inline(always)]
    fn solved() -> Self {
        Self::default()
    }

    #[inline(always)]
    fn u(&self) -> Self {
        // U moves don't mess with the orientation of anything, which is nice; just permutations
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
    fn m(&self) -> Self {
        Self {
            // M layer edges permute and flip
            uf: self.df.flipped(),
            df: self.db.flipped(),
            db: self.ub.flipped(),
            ub: self.uf.flipped(),

            // centers permute, but have no orientation, so we can ignore that here

            // everything else same
            ..*self
        }
    }

    #[inline(always)]
    fn r(&self) -> Self {
        Self {
            // R face corners permute and reorient
            ufr: self.dfr.ccw(),
            dfr: self.dbr.cw(),
            dbr: self.ubr.ccw(),
            ubr: self.ufr.cw(),
            // R face edges permute but orientation is preserved
            ur: self.fr,
            fr: self.dr,
            dr: self.br,
            br: self.ur,
            // everything else same
            ..*self
        }
    }

    #[inline(always)]
    fn rw(&self) -> Self {
        self.m().r()
    }
}

impl Bandaged3x3x3with1x2x3 {
    #[inline(always)]
    fn solved() -> Self {
        Self {
            pos: PositionState::solved(),
            orr: OrientationState::solved(),
        }
    }

    #[inline(always)]
    fn r(&self) -> Self {
        Self {
            pos: self.pos.r(),
            orr: self.orr.r(),
        }
    }

    #[inline(always)]
    fn rw(&self) -> Self {
        Self {
            pos: self.pos.rw(),
            orr: self.orr.rw(),
        }
    }

    #[inline(always)]
    fn u(&self) -> Self {
        Self {
            pos: self.pos.u(),
            orr: self.orr.u(),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Hash, Sequence)]
pub enum Move {
    // But U can do any of the usual four amounts
    #[display(fmt = "R{}", _0)]
    R(CubeMoveAmt),
    #[display(fmt = "Rw{}", _0)]
    Rw(CubeMoveAmt),
    #[display(fmt = "U{}", _0)]
    U(CubeMoveAmt),
}

impl CanReverse for Move {
    fn reverse(&self) -> Self {
        match self {
            Move::Rw(amt) => Move::Rw(amt.reverse()),
            Move::R(amt) => Move::R(amt.reverse()),
            Move::U(amt) => Move::U(amt.reverse()),
        }
    }
}

impl Solvable for Bandaged3x3x3with1x2x3 {
    type Move = Move;

    fn is_solved(&self) -> bool {
        self == &Bandaged3x3x3with1x2x3::solved()
    }

    fn available_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        [
            Move::R(CubeMoveAmt::One),
            Move::R(CubeMoveAmt::Two),
            Move::R(CubeMoveAmt::Rev),
            Move::Rw(CubeMoveAmt::One),
            Move::Rw(CubeMoveAmt::Two),
            Move::Rw(CubeMoveAmt::Rev),
            Move::U(CubeMoveAmt::One),
            Move::U(CubeMoveAmt::Two),
            Move::U(CubeMoveAmt::Rev),
        ]
    }

    fn is_redundant(last_move: Self::Move, next_move: Self::Move) -> bool {
        match last_move {
            Move::U(_) => matches!(next_move, Move::U(_)),
            Move::R(_) => matches!(next_move, Move::R(_)),
            Move::Rw(_) => matches!(next_move, Move::Rw(_)) || matches!(next_move, Move::R(_)),
        }
    }

    fn apply(&self, m: Self::Move) -> Self {
        match m {
            Move::R(amt) => match amt {
                CubeMoveAmt::One => self.r(),
                CubeMoveAmt::Two => self.r().r(),
                CubeMoveAmt::Rev => self.r().r().r(),
            },
            Move::Rw(amt) => match amt {
                CubeMoveAmt::One => self.rw(),
                CubeMoveAmt::Two => self.rw().rw(),
                CubeMoveAmt::Rev => self.rw().rw().rw(),
            },
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

pub fn make_heuristic() -> impl Heuristic<Bandaged3x3x3with1x2x3> {
    bounded_cache::<Bandaged3x3x3with1x2x3>(8)
}

impl SimpleStartState for Bandaged3x3x3with1x2x3 {
    type UniqueKey = (u64, u64);

    fn start() -> Self {
        Self::solved()
    }

    fn uniq_key(&self) -> Self::UniqueKey {
        // 6 corners: 3 bits for pos, 2 bits for orr        30 bits
        // 9 edges: 3 bits for pos, 1 bit for orr           36 bits
        // 4 centers: 2 bits for pos, orr is fixed          8 bits

        // so we'll put the edges on one side, and the corners and center into the other

        // pack corners & centers
        let mut out = 0;
        self.pos.ufl.pack(&mut out);
        self.pos.ufr.pack(&mut out);
        self.pos.ubl.pack(&mut out);
        self.pos.ubr.pack(&mut out);
        self.pos.dfr.pack(&mut out);
        self.pos.dbr.pack(&mut out);

        out = (out << 2) | self.orr.ufl.as_u8_two_bits() as u64;
        out = (out << 2) | self.orr.ufr.as_u8_two_bits() as u64;
        out = (out << 2) | self.orr.ubl.as_u8_two_bits() as u64;
        out = (out << 2) | self.orr.ubr.as_u8_two_bits() as u64;
        out = (out << 2) | self.orr.dfr.as_u8_two_bits() as u64;
        out = (out << 2) | self.orr.dbr.as_u8_two_bits() as u64;

        self.pos.uc.pack(&mut out);
        self.pos.dc.pack(&mut out);
        self.pos.fc.pack(&mut out);
        self.pos.bc.pack(&mut out);

        let out_corners = out;

        // pack edges
        let mut out = 0;
        self.pos.uf.pack(&mut out);
        self.pos.uf.pack(&mut out);
        self.pos.uf.pack(&mut out);
        self.pos.uf.pack(&mut out);
        self.pos.uf.pack(&mut out);
        self.pos.uf.pack(&mut out);
        self.pos.uf.pack(&mut out);
        self.pos.uf.pack(&mut out);
        self.pos.uf.pack(&mut out);

        out = (out << 3) | self.orr.uf.as_u8_one_bit() as u64;
        out = (out << 3) | self.orr.ur.as_u8_one_bit() as u64;
        out = (out << 3) | self.orr.ul.as_u8_one_bit() as u64;
        out = (out << 3) | self.orr.ub.as_u8_one_bit() as u64;
        out = (out << 3) | self.orr.fr.as_u8_one_bit() as u64;
        out = (out << 3) | self.orr.br.as_u8_one_bit() as u64;
        out = (out << 3) | self.orr.df.as_u8_one_bit() as u64;
        out = (out << 3) | self.orr.db.as_u8_one_bit() as u64;
        out = (out << 3) | self.orr.dr.as_u8_one_bit() as u64;
        let out_edges = out;

        (out_corners, out_edges)
    }
}

impl RandomInit for Bandaged3x3x3with1x2x3 {
    fn random_state<R: Rng>(_r: &mut R) -> Self {
        todo!("not sure about this yet, still working")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use ahash::HashSet;

    use super::CornerCubelet;

    #[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
    struct CornerPositionState {
        // six corners (dbl, dfl fixed)
        ufl: CornerCubelet,
        ufr: CornerCubelet,
        ubl: CornerCubelet,
        ubr: CornerCubelet,
        dfr: CornerCubelet,
        dbr: CornerCubelet,
    }

    impl CornerPositionState {
        fn r(&self) -> Self {
            Self {
                ufl: self.ufl,
                ubl: self.ubl,

                ufr: self.dfr,
                dfr: self.dbr,
                dbr: self.ubr,
                ubr: self.ufr,
            }
        }

        fn u(&self) -> Self {
            Self {
                dbr: self.dbr,
                dfr: self.dfr,

                ufl: self.ufr,
                ufr: self.ubr,
                ubr: self.ubl,
                ubl: self.ufl,
            }
        }
    }

    #[test]
    fn count_permutations() {
        let start = CornerPositionState {
            ufl: CornerCubelet::UFL,
            ufr: CornerCubelet::UFR,
            ubl: CornerCubelet::UBL,
            ubr: CornerCubelet::UBR,
            dbr: CornerCubelet::DBR,
            dfr: CornerCubelet::DFR,
        };

        let mut seen = HashSet::default();

        let mut to_process = VecDeque::new();

        to_process.push_back(start);

        while let Some(p) = to_process.pop_front() {
            let r = p.r();
            let u = p.u();

            if seen.insert(r) {
                to_process.push_back(r);
            }
            if seen.insert(u) {
                to_process.push_back(u);
            }
        }

        // basically we think any even permutation is fine, except when you get to the top,
        // everything is correct, possibly with a rotation? which is weird? can't prove it
        // analytically but we can demonstrate it empirically since the numbers are fairly small.
        // the total should be 6 * 5 (for the positions of your choice of DFR and DBR)
        // times 4 (for the position of UFL); the rest should be fixed. Total is 120

        assert_eq!(seen.len(), 120);
    }
}

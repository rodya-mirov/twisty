use std::fmt::Formatter;

use crate::cubesearch::SimpleStartState;
use derive_more::Display;
use enum_iterator::{all, Sequence};
use rand::Rng;

use crate::idasearch::heuristic_helpers::bounded_cache;
use crate::idasearch::{Heuristic, Solvable};
use crate::moves::{CanReverse, CornerTwistAmt};
use crate::random_helpers::{shuffle_with_parity, TwoParity};
use crate::scrambles::RandomInit;

#[repr(u8)]
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Sequence)]
enum EdgeCubelet {
    // we leave the UF fixed; everything else is represented
    // u layer
    UL,
    UB,
    UR,

    // mid layer
    FL,
    FR,
    BL,
    BR,

    // d layer
    DL,
    DB,
    DR,
    DF,
}

impl EdgeCubelet {
    #[inline(always)]
    fn pack(self, bits: &mut u64) {
        *bits = (*bits << 4) | (self as u64);
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct DinoCube {
    // u layer
    ul: EdgeCubelet,
    ub: EdgeCubelet,
    ur: EdgeCubelet,

    // mid layer
    fl: EdgeCubelet,
    fr: EdgeCubelet,
    bl: EdgeCubelet,
    br: EdgeCubelet,

    // d layer
    dl: EdgeCubelet,
    db: EdgeCubelet,
    dr: EdgeCubelet,
    df: EdgeCubelet,
}

impl DinoCube {
    pub fn solved_state() -> Self {
        Self {
            // U layer
            ul: EdgeCubelet::UL,
            ub: EdgeCubelet::UB,
            ur: EdgeCubelet::UR,
            // mid layer
            fl: EdgeCubelet::FL,
            fr: EdgeCubelet::FR,
            bl: EdgeCubelet::BL,
            br: EdgeCubelet::BR,
            // D layer
            dl: EdgeCubelet::DL,
            db: EdgeCubelet::DB,
            dr: EdgeCubelet::DR,
            df: EdgeCubelet::DF,
        }
    }

    pub fn solved_mirrored() -> Self {
        // L and R are mirrored; everything else is the same
        Self {
            // U layer
            ul: EdgeCubelet::UR,
            ub: EdgeCubelet::UB,
            ur: EdgeCubelet::UL,
            // mid layer
            fl: EdgeCubelet::FR,
            fr: EdgeCubelet::FL,
            bl: EdgeCubelet::BR,
            br: EdgeCubelet::BL,
            // D layer
            dl: EdgeCubelet::DR,
            db: EdgeCubelet::DB,
            dr: EdgeCubelet::DL,
            df: EdgeCubelet::DF,
        }
    }

    #[inline(always)]
    fn ubl(&self) -> Self {
        Self {
            ul: self.ub,
            ub: self.bl,
            bl: self.ul,
            ..*self
        }
    }

    #[inline(always)]
    fn ubr(&self) -> Self {
        Self {
            ur: self.br,
            br: self.ub,
            ub: self.ur,
            ..*self
        }
    }

    #[inline(always)]
    fn dfl(&self) -> Self {
        Self {
            fl: self.dl,
            dl: self.df,
            df: self.fl,
            ..*self
        }
    }

    #[inline(always)]
    fn dfr(&self) -> Self {
        Self {
            fr: self.df,
            df: self.dr,
            dr: self.fr,
            ..*self
        }
    }

    #[inline(always)]
    fn dbl(&self) -> Self {
        Self {
            bl: self.db,
            db: self.dl,
            dl: self.bl,
            ..*self
        }
    }

    #[inline(always)]
    fn dbr(&self) -> Self {
        Self {
            dr: self.db,
            db: self.br,
            br: self.dr,
            ..*self
        }
    }

    #[inline(always)]
    fn dblw(&self) -> Self {
        Self {
            // everything from dbl
            bl: self.db,
            db: self.dl,
            dl: self.bl,
            // then another layer, which is a lot of stuff unfortunately; two 3-cycles
            // cycle one
            fl: self.ub,
            ub: self.dr,
            dr: self.fl,
            // cycle two
            ul: self.br,
            br: self.df,
            df: self.ul,
            // rest is same
            ..*self
        }
    }

    #[inline(always)]
    fn dbrw(&self) -> Self {
        Self {
            // everything from dbr
            dr: self.db,
            db: self.br,
            br: self.dr,
            // then another layer, which is a lot of stuff unfortunately; two 3-cycles
            // cycle one
            fr: self.dl,
            dl: self.ub,
            ub: self.fr,
            // cycle two
            df: self.bl,
            bl: self.ur,
            ur: self.df,
            // rest is same
            ..*self
        }
    }
}

impl RandomInit for DinoCube {
    fn random_state<R: Rng>(r: &mut R) -> Self {
        let edges: Vec<EdgeCubelet> = all::<EdgeCubelet>().collect();
        let edges = shuffle_with_parity(r, &edges, TwoParity::Even);

        Self {
            ul: edges[0],
            ub: edges[1],
            ur: edges[2],
            fl: edges[3],
            fr: edges[4],
            bl: edges[5],
            br: edges[6],
            dl: edges[7],
            db: edges[8],
            dr: edges[9],
            df: edges[10],
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Sequence, Display)]
enum Dir {
    // top layer; note no UF twists
    UBL,
    UBR,
    // bottom layer
    DFL,
    DFR,
    DBL,
    DBR,
    // two-layer twists of the UF-antipodal corners
    DBLw,
    DBRw,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Sequence)]
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

impl Solvable for DinoCube {
    type Move = Move;

    fn is_solved(&self) -> bool {
        self == &Self::solved_state() || self == &Self::solved_mirrored()
    }

    fn available_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        [
            // wow i hated typing this all out
            // top layer
            Move(Dir::UBL, CornerTwistAmt::Cw),
            Move(Dir::UBL, CornerTwistAmt::Ccw),
            Move(Dir::UBR, CornerTwistAmt::Cw),
            Move(Dir::UBR, CornerTwistAmt::Ccw),
            // bottom front
            Move(Dir::DFL, CornerTwistAmt::Cw),
            Move(Dir::DFL, CornerTwistAmt::Ccw),
            Move(Dir::DFR, CornerTwistAmt::Cw),
            Move(Dir::DFR, CornerTwistAmt::Ccw),
            // bottom back
            Move(Dir::DBL, CornerTwistAmt::Cw),
            Move(Dir::DBL, CornerTwistAmt::Ccw),
            Move(Dir::DBL, CornerTwistAmt::Cw),
            Move(Dir::DBL, CornerTwistAmt::Ccw),
            // bottom back double slices
            Move(Dir::DBLw, CornerTwistAmt::Cw),
            Move(Dir::DBLw, CornerTwistAmt::Ccw),
            Move(Dir::DBRw, CornerTwistAmt::Cw),
            Move(Dir::DBRw, CornerTwistAmt::Ccw),
        ]
    }

    fn is_redundant(last_move: Self::Move, next_move: Self::Move) -> bool {
        let last_dir = last_move.0;
        let next_dir = next_move.0;

        // note: we could speed this up dramatically, since lots of directions do commute with
        // each other

        match last_dir {
            // for the deep slices, we want deep slice to go after regular slice (no reason, just
            // need to pick something)
            Dir::DBLw => next_dir == Dir::DBLw || next_dir == Dir::DBL,
            Dir::DBRw => next_dir == Dir::DBRw || next_dir == Dir::DBR,
            // in other cases redundancy is just being the same direction
            _ => last_dir == next_dir,
        }
    }

    fn apply(&self, m: Self::Move) -> Self {
        match (m.0, m.1) {
            (Dir::UBL, CornerTwistAmt::Cw) => self.ubl(),
            (Dir::UBL, CornerTwistAmt::Ccw) => self.ubl().ubl(),
            (Dir::UBR, CornerTwistAmt::Cw) => self.ubr(),
            (Dir::UBR, CornerTwistAmt::Ccw) => self.ubr().ubr(),
            (Dir::DFL, CornerTwistAmt::Cw) => self.dfl(),
            (Dir::DFL, CornerTwistAmt::Ccw) => self.dfl().dfl(),
            (Dir::DFR, CornerTwistAmt::Cw) => self.dfr(),
            (Dir::DFR, CornerTwistAmt::Ccw) => self.dfr().dfr(),
            (Dir::DBL, CornerTwistAmt::Cw) => self.dbl(),
            (Dir::DBL, CornerTwistAmt::Ccw) => self.dbl().dbl(),
            (Dir::DBR, CornerTwistAmt::Cw) => self.dbr(),
            (Dir::DBR, CornerTwistAmt::Ccw) => self.dbr().dbr(),
            (Dir::DBLw, CornerTwistAmt::Cw) => self.dblw(),
            (Dir::DBLw, CornerTwistAmt::Ccw) => self.dblw().dblw(),
            (Dir::DBRw, CornerTwistAmt::Cw) => self.dbrw(),
            (Dir::DBRw, CornerTwistAmt::Ccw) => self.dbrw().dbrw(),
        }
    }

    fn max_fuel() -> usize {
        11
    }
}

impl SimpleStartState for DinoCube {
    type UniqueKey = u64;

    fn start() -> Self {
        Self::solved_state()
    }

    fn uniq_key(&self) -> Self::UniqueKey {
        let mut out: u64 = 0;

        // we can pack any 10 edges; the last is determined by the previous 10

        // arbitrarily we'll do 2 from u
        self.ul.pack(&mut out);
        self.ur.pack(&mut out);

        // all of mid
        self.fl.pack(&mut out);
        self.fr.pack(&mut out);
        self.bl.pack(&mut out);
        self.br.pack(&mut out);

        // all of d
        self.df.pack(&mut out);
        self.dl.pack(&mut out);
        self.db.pack(&mut out);
        self.dr.pack(&mut out);

        out
    }
}

pub fn make_heuristic() -> impl Heuristic<DinoCube> {
    // max depth is picked to keep the compute time low
    bounded_cache::<DinoCube>(6)
}

#[cfg(test)]
mod bits_test {
    use enum_iterator::all;

    use super::*;

    #[test]
    fn should_fit_in_four_bits() {
        for c in all::<EdgeCubelet>() {
            assert!((c as u8) < 16);
        }
    }
}

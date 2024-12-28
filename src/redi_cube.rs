use derive_more::Display;
use enum_iterator::Sequence;
use rand::Rng;

use crate::cubesearch::SimpleStartState;
use crate::idasearch::heuristic_helpers::bounded_cache;
use crate::idasearch::{Heuristic, Solvable};
use crate::moves::{CanReverse, CornerTwistAmt};
use crate::orientations::CornerOrientation;
use crate::permutation_helpers::cycle_cw;
use crate::random_helpers::TwoParity;
use crate::scrambles::RandomInit;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Ord, PartialOrd, Sequence)]
#[repr(u8)]
pub enum EdgeCubelet {
    // 12 values, so fits in 4 bits
    UF,
    UL,
    UR,
    UB,
    DF,
    DL,
    DR,
    DB,
    FL,
    FR,
    BL,
    BR,
}

impl EdgeCubelet {
    #[inline(always)]
    fn pack(self, source: &mut u64) {
        *source = (*source << 4) + (self as u64);
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Ord, PartialOrd, Sequence)]
struct EdgeState {
    uf: EdgeCubelet,
    ur: EdgeCubelet,
    ul: EdgeCubelet,
    ub: EdgeCubelet,

    df: EdgeCubelet,
    dr: EdgeCubelet,
    dl: EdgeCubelet,
    db: EdgeCubelet,

    fl: EdgeCubelet,
    fr: EdgeCubelet,
    bl: EdgeCubelet,
    br: EdgeCubelet,
}

impl EdgeState {
    #[inline(always)]
    fn pack(self, source: &mut u64) {
        self.uf.pack(source);
        self.ur.pack(source);
        self.ul.pack(source);
        self.ub.pack(source);

        self.df.pack(source);
        self.dr.pack(source);
        self.dl.pack(source);
        self.db.pack(source);

        self.fl.pack(source);
        self.fr.pack(source);
        self.bl.pack(source);
        self.br.pack(source);
    }

    #[inline(always)]
    fn solved() -> Self {
        Self {
            uf: EdgeCubelet::UF,
            ur: EdgeCubelet::UR,
            ul: EdgeCubelet::UL,
            ub: EdgeCubelet::UB,

            df: EdgeCubelet::DF,
            dr: EdgeCubelet::DR,
            dl: EdgeCubelet::DL,
            db: EdgeCubelet::DB,

            fl: EdgeCubelet::FL,
            fr: EdgeCubelet::FR,
            bl: EdgeCubelet::BL,
            br: EdgeCubelet::BR,
        }
    }

    #[inline(always)]
    fn ufl(&mut self) {
        cycle_cw(&mut self.uf, &mut self.fl, &mut self.ul);
    }

    #[inline(always)]
    fn ufr(&mut self) {
        cycle_cw(&mut self.uf, &mut self.ur, &mut self.fr);
    }

    #[inline(always)]
    fn ubl(&mut self) {
        cycle_cw(&mut self.ub, &mut self.ul, &mut self.bl);
    }

    #[inline(always)]
    fn ubr(&mut self) {
        cycle_cw(&mut self.ub, &mut self.br, &mut self.ur);
    }

    #[inline(always)]
    fn dfl(&mut self) {
        cycle_cw(&mut self.df, &mut self.dl, &mut self.fl);
    }

    #[inline(always)]
    fn dfr(&mut self) {
        cycle_cw(&mut self.df, &mut self.fr, &mut self.dr);
    }

    #[inline(always)]
    fn dbl(&mut self) {
        cycle_cw(&mut self.db, &mut self.bl, &mut self.dl);
    }

    #[inline(always)]
    fn dbr(&mut self) {
        cycle_cw(&mut self.db, &mut self.dr, &mut self.br);
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct CornerState {
    ufl: CornerOrientation,
    ufr: CornerOrientation,
    ubl: CornerOrientation,
    ubr: CornerOrientation,

    dfl: CornerOrientation,
    dfr: CornerOrientation,
    dbl: CornerOrientation,
    dbr: CornerOrientation,
}

impl CornerState {
    #[inline(always)]
    fn pack(self, source: &mut u64) {
        self.ufl.pack_two_bits(source);
        self.ufr.pack_two_bits(source);
        self.ubl.pack_two_bits(source);
        self.ubr.pack_two_bits(source);

        self.dfl.pack_two_bits(source);
        self.dfr.pack_two_bits(source);
        self.dbl.pack_two_bits(source);
        self.dbr.pack_two_bits(source);
    }

    #[inline(always)]
    fn solved() -> Self {
        Self {
            ufl: CornerOrientation::Normal,
            ufr: CornerOrientation::Normal,
            ubl: CornerOrientation::Normal,
            ubr: CornerOrientation::Normal,
            dfl: CornerOrientation::Normal,
            dfr: CornerOrientation::Normal,
            dbl: CornerOrientation::Normal,
            dbr: CornerOrientation::Normal,
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct RediCube {
    edges: EdgeState,
    corners: CornerState,
}

impl RediCube {
    #[inline(always)]
    fn solved() -> Self {
        Self {
            edges: EdgeState::solved(),
            corners: CornerState::solved(),
        }
    }

    #[inline(always)]
    fn ufr(&mut self) {
        self.edges.ufr();
        self.corners.ufr.cw_mut();
    }

    #[inline(always)]
    fn ufl(&mut self) {
        self.edges.ufl();
        self.corners.ufl.cw_mut();
    }

    #[inline(always)]
    fn ubl(&mut self) {
        self.edges.ubl();
        self.corners.ubl.cw_mut();
    }

    #[inline(always)]
    fn ubr(&mut self) {
        self.edges.ubr();
        self.corners.ubr.cw_mut();
    }

    #[inline(always)]
    fn dfr(&mut self) {
        self.edges.dfr();
        self.corners.dfr.cw_mut();
    }

    #[inline(always)]
    fn dfl(&mut self) {
        self.edges.dfl();
        self.corners.dfl.cw_mut();
    }

    #[inline(always)]
    fn dbl(&mut self) {
        self.edges.dbl();
        self.corners.dbl.cw_mut();
    }

    #[inline(always)]
    fn dbr(&mut self) {
        self.edges.dbr();
        self.corners.dbr.cw_mut();
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Hash, Sequence)]
pub enum Move {
    #[display(fmt = "UFR{}", _0)]
    UFR(CornerTwistAmt),
    #[display(fmt = "UFL{}", _0)]
    UFL(CornerTwistAmt),
    #[display(fmt = "UBR{}", _0)]
    UBR(CornerTwistAmt),
    #[display(fmt = "UBL{}", _0)]
    UBL(CornerTwistAmt),

    #[display(fmt = "DFR{}", _0)]
    DFR(CornerTwistAmt),
    #[display(fmt = "DFL{}", _0)]
    DFL(CornerTwistAmt),
    #[display(fmt = "DBR{}", _0)]
    DBR(CornerTwistAmt),
    #[display(fmt = "DBL{}", _0)]
    DBL(CornerTwistAmt),
}

impl CanReverse for Move {
    fn reverse(&self) -> Self {
        match self {
            Move::UFR(a) => Move::UFR(a.reverse()),
            Move::UFL(a) => Move::UFL(a.reverse()),
            Move::UBR(a) => Move::UBR(a.reverse()),
            Move::UBL(a) => Move::UBL(a.reverse()),

            Move::DFR(a) => Move::DFR(a.reverse()),
            Move::DFL(a) => Move::DFL(a.reverse()),
            Move::DBR(a) => Move::DBR(a.reverse()),
            Move::DBL(a) => Move::DBL(a.reverse()),
        }
    }
}

impl Solvable for RediCube {
    type Move = Move;

    fn is_solved(&self) -> bool {
        self == &Self::solved()
    }

    fn available_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        [
            Move::UFL(CornerTwistAmt::Cw),
            Move::UFL(CornerTwistAmt::Ccw),
            Move::UFR(CornerTwistAmt::Cw),
            Move::UFR(CornerTwistAmt::Ccw),
            Move::UBL(CornerTwistAmt::Cw),
            Move::UBL(CornerTwistAmt::Ccw),
            Move::UBR(CornerTwistAmt::Cw),
            Move::UBR(CornerTwistAmt::Ccw),
            Move::DFL(CornerTwistAmt::Cw),
            Move::DFL(CornerTwistAmt::Ccw),
            Move::DFR(CornerTwistAmt::Cw),
            Move::DFR(CornerTwistAmt::Ccw),
            Move::DBL(CornerTwistAmt::Cw),
            Move::DBL(CornerTwistAmt::Ccw),
            Move::DBR(CornerTwistAmt::Cw),
            Move::DBR(CornerTwistAmt::Ccw),
        ]
    }

    fn is_redundant(last_move: Self::Move, next_move: Self::Move) -> bool {
        // basically we'll say that moves are in reverse order -- so
        //      UFR > UFL > UBR > UBL > DFR > DFL > DFR > DBR > DBL
        // and that if two moves COMMUTE then the larger one should come SECOND; so you could do
        //  (e.g.) UFR DFL, but not DFL UFR
        // two moves COMMUTE if they don't touch the same edges (which in our notation means they
        // have at most one letter in common) or if they're the same (no sense doing two UFR moves
        // in a row)
        match last_move {
            Move::UFR(_) => matches!(next_move, Move::UFR(_)),
            Move::UFL(_) => matches!(next_move, Move::UFL(_)),
            Move::UBR(_) => matches!(next_move, Move::UFL(_) | Move::UBR(_)),
            Move::UBL(_) => matches!(next_move, Move::UFR(_) | Move::UBL(_)),
            Move::DFR(_) => matches!(next_move, Move::UFL(_) | Move::UBR(_) | Move::UBL(_) | Move::DFR(_)),
            Move::DFL(_) => matches!(next_move, Move::UFR(_) | Move::UBR(_) | Move::UBL(_) | Move::DFL(_)),
            Move::DBR(_) => matches!(
                next_move,
                Move::UFR(_) | Move::UFL(_) | Move::UBL(_) | Move::DFL(_) | Move::DBR(_)
            ),
            Move::DBL(_) => matches!(
                next_move,
                Move::UFR(_) | Move::UFL(_) | Move::UBR(_) | Move::DFR(_) | Move::DBL(_)
            ),
        }
    }

    fn apply(&self, m: Self::Move) -> Self {
        let mut out = self.clone();
        match m {
            Move::UFR(amt) => match amt {
                CornerTwistAmt::Cw => out.ufr(),
                // inlining should eliminate the repetition here
                CornerTwistAmt::Ccw => {
                    out.ufr();
                    out.ufr();
                }
            },
            Move::UFL(amt) => match amt {
                CornerTwistAmt::Cw => out.ufl(),
                // inlining should eliminate the repetition here
                CornerTwistAmt::Ccw => {
                    out.ufl();
                    out.ufl();
                }
            },
            Move::UBL(amt) => match amt {
                CornerTwistAmt::Cw => out.ubl(),
                // inlining should eliminate the repetition here
                CornerTwistAmt::Ccw => {
                    out.ubl();
                    out.ubl();
                }
            },
            Move::UBR(amt) => match amt {
                CornerTwistAmt::Cw => out.ubr(),
                // inlining should eliminate the repetition here
                CornerTwistAmt::Ccw => {
                    out.ubr();
                    out.ubr();
                }
            },

            Move::DFR(amt) => match amt {
                CornerTwistAmt::Cw => out.dfr(),
                // inlining should eliminate the repetition here
                CornerTwistAmt::Ccw => {
                    out.dfr();
                    out.dfr();
                }
            },
            Move::DFL(amt) => match amt {
                CornerTwistAmt::Cw => out.dfl(),
                // inlining should eliminate the repetition here
                CornerTwistAmt::Ccw => {
                    out.dfl();
                    out.dfl();
                }
            },
            Move::DBL(amt) => match amt {
                CornerTwistAmt::Cw => out.dbl(),
                // inlining should eliminate the repetition here
                CornerTwistAmt::Ccw => {
                    out.dbl();
                    out.dbl();
                }
            },
            Move::DBR(amt) => match amt {
                CornerTwistAmt::Cw => out.dbr(),
                // inlining should eliminate the repetition here
                CornerTwistAmt::Ccw => {
                    out.dbr();
                    out.dbr();
                }
            },
        }
        out
    }

    fn max_fuel() -> usize {
        30
    }
}

pub fn make_heuristic() -> impl Heuristic<RediCube> {
    bounded_cache::<RediCube>(8)
}

impl SimpleStartState for RediCube {
    type UniqueKey = u64;

    fn start() -> Self {
        Self::solved()
    }

    fn uniq_key(&self) -> Self::UniqueKey {
        // 8 corners: 2 bits each -- 16 bits here
        // 12 edges: 4 bits each -- 48 bits here
        // that's EXACTLY 64 bits (whew)

        // pack corners & centers
        let mut out = 0;
        self.edges.pack(&mut out);
        self.corners.pack(&mut out);

        out
    }
}

impl RandomInit for RediCube {
    fn random_state<R: Rng>(r: &mut R) -> Self {
        let permutation = crate::random_helpers::shuffle_with_parity(
            r,
            &[
                EdgeCubelet::UF,
                EdgeCubelet::UR,
                EdgeCubelet::UL,
                EdgeCubelet::UB,
                EdgeCubelet::DF,
                EdgeCubelet::DR,
                EdgeCubelet::DL,
                EdgeCubelet::DB,
                EdgeCubelet::FL,
                EdgeCubelet::FR,
                EdgeCubelet::BL,
                EdgeCubelet::BR,
            ],
            TwoParity::Even,
        );

        assert_eq!(permutation.len(), 12);

        let edges = EdgeState {
            uf: permutation[0],
            ur: permutation[1],
            ul: permutation[2],
            ub: permutation[3],
            df: permutation[4],
            dr: permutation[5],
            dl: permutation[6],
            db: permutation[7],
            fl: permutation[8],
            fr: permutation[9],
            bl: permutation[10],
            br: permutation[11],
        };

        let corners = CornerState {
            ufl: r.gen(),
            ufr: r.gen(),
            ubl: r.gen(),
            ubr: r.gen(),
            dfl: r.gen(),
            dfr: r.gen(),
            dbl: r.gen(),
            dbr: r.gen(),
        };

        RediCube { edges, corners }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moves::CornerTwistAmt::{Ccw, Cw};

    #[test]
    fn total_perm_test() {
        let mut state = RediCube::solved();

        let moves: Vec<Move> = vec![
            Move::UFL(Cw),
            Move::DBL(Ccw),
            Move::DBR(Cw),
            Move::UFR(Cw),
            Move::UBL(Cw),
            Move::UBR(Ccw),
            Move::DFL(Cw),
            Move::DFR(Ccw),
        ];

        for m in moves {
            state = state.apply(m);
        }

        assert_eq!(
            state,
            RediCube {
                edges: EdgeState {
                    uf: EdgeCubelet::FR,
                    ur: EdgeCubelet::DL,
                    ul: EdgeCubelet::UB,
                    ub: EdgeCubelet::DR,

                    df: EdgeCubelet::UR,
                    dr: EdgeCubelet::UF,
                    dl: EdgeCubelet::DF,
                    db: EdgeCubelet::BR,

                    fl: EdgeCubelet::DB,
                    fr: EdgeCubelet::BL,
                    bl: EdgeCubelet::FL,
                    br: EdgeCubelet::UL,
                },
                corners: CornerState {
                    ufl: CornerOrientation::CW,
                    ufr: CornerOrientation::CW,
                    ubl: CornerOrientation::CW,
                    ubr: CornerOrientation::CCW,

                    dfl: CornerOrientation::CW,
                    dfr: CornerOrientation::CCW,
                    dbl: CornerOrientation::CCW,
                    dbr: CornerOrientation::CW,
                }
            }
        )
    }
}

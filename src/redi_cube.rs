#[cfg(feature = "hit_rate")]
use std::sync::atomic::{AtomicUsize, Ordering};

use derive_more::Display;
use enum_iterator::Sequence;
use rand::Rng;

use crate::cubesearch::SimpleStartState;
use crate::idasearch::heuristic_helpers::{bounded_cache, BoundedStateCache};
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
        self.ufl.pack_two_bits_u64(source);
        self.ufr.pack_two_bits_u64(source);
        self.ubl.pack_two_bits_u64(source);
        self.ubr.pack_two_bits_u64(source);

        self.dfl.pack_two_bits_u64(source);
        self.dfr.pack_two_bits_u64(source);
        self.dbl.pack_two_bits_u64(source);
        self.dbr.pack_two_bits_u64(source);
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

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
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

macro_rules! do_twist {
    ($corner_name:ident, $amt_name:ident, $out:ident) => {
        match $amt_name {
            CornerTwistAmt::Cw => $out.$corner_name(),
            // inlining should eliminate the repetition here
            CornerTwistAmt::Ccw => {
                $out.$corner_name();
                $out.$corner_name();
            }
        }
    };
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
            Move::UFR(amt) => do_twist!(ufr, amt, out),
            Move::UFL(amt) => do_twist!(ufl, amt, out),
            Move::UBL(amt) => do_twist!(ubl, amt, out),
            Move::UBR(amt) => do_twist!(ubr, amt, out),

            Move::DFR(amt) => do_twist!(dfr, amt, out),
            Move::DFL(amt) => do_twist!(dfl, amt, out),
            Move::DBL(amt) => do_twist!(dbl, amt, out),
            Move::DBR(amt) => do_twist!(dbr, amt, out),
        }
        out
    }

    fn max_fuel() -> usize {
        30
    }
}

// TODO: generify this somehow so I can use it in other places, if it works
struct RediHeuristic {
    bounded_cache: BoundedStateCache<u64>,
    #[cfg(feature = "hit_rate")]
    heuristic_hits: AtomicUsize,
    #[cfg(feature = "hit_rate")]
    heuristic_misses: AtomicUsize,
}

#[cfg(feature = "hit_rate")]
impl Drop for RediHeuristic {
    fn drop(&mut self) {
        let hits = self.heuristic_hits.load(Ordering::Relaxed);
        let misses = self.heuristic_misses.load(Ordering::Relaxed);
        let pct = (hits as f32) / ((hits + misses) as f32) * 100.0;
        println!(
            "Final stats for redi heuristic: {hits} heuristic hits and {misses} heuristic misses ({pct:.2}% hit rate)"
        )
    }
}

impl Heuristic<RediCube> for RediHeuristic {
    fn estimated_remaining_cost(&self, t: &RediCube) -> usize {
        // turns out this hashmap lookup is still the pain point; more efficient packing or lookups
        // may help performance further
        if let Some(known_cost) = self.bounded_cache.remaining_cost_if_known(t) {
            return known_cost;
        }

        let fb = self.bounded_cache.fallback_depth();
        let heuristic = dist_heuristic(t);

        #[cfg(feature = "hit_rate")]
        {
            if heuristic > fb {
                self.heuristic_hits.fetch_add(1, Ordering::Relaxed);
            } else {
                self.heuristic_misses.fetch_add(1, Ordering::Relaxed);
            }
        }

        fb.max(heuristic)
    }
}

// compute the minimum cost (number of times this edge cubelet must move) based on the
// orientations of its associated corners. This is assuming the cubelet is in its home position.
#[inline(always)]
fn in_place_cost(corner_a_orr: CornerOrientation, corner_b_orr: CornerOrientation) -> usize {
    if corner_a_orr == CornerOrientation::Normal {
        if corner_b_orr == CornerOrientation::Normal {
            0
        } else {
            2
        }
    } else {
        if corner_b_orr == CornerOrientation::Normal {
            2
        } else {
            4
        }
    }
}

// compute the minimum cost (number of times this edge cubelet must move) based on the
// orientation of its attached corner. This is assuming the edge cubelet is one away from its
// home; that is, it's attached to one of its associated corners, but not the other.
// The arguments pertain to the actual orientation of that corner as well as the expected one
// (expected meaning "the edge and corner agree, so you can send it home in one move")
#[inline(always)]
fn one_off_cost(expected_corner_orr: CornerOrientation, actual_corner_orr: CornerOrientation) -> usize {
    if actual_corner_orr == expected_corner_orr {
        1
    } else {
        3
    }
}

// good candidate for inlining because very often one of the arguments is constant, so LLVM
// can unroll it to a very simple comparison. Too bad const generics doesn't yet allow ADTs as
// generics to optimize out
//
// :param: source_position -- this is the slot the cubelet came from (essentially the field name)
// :param: goal_position -- slot the cubelet wants to go to; this "is" the thing of interest
#[inline(always)]
fn dist(source_position: EdgeCubelet, goal_position: EdgeCubelet, cube: &RediCube) -> usize {
    match goal_position {
        EdgeCubelet::UF => {
            // using this as a baseline, computed very carefully by hand
            match source_position {
                EdgeCubelet::UF => in_place_cost(cube.corners.ufl, cube.corners.ufr),
                EdgeCubelet::DB => 3,
                EdgeCubelet::UL => one_off_cost(cube.corners.ufl, CornerOrientation::CCW),
                EdgeCubelet::FL => one_off_cost(cube.corners.ufl, CornerOrientation::CW),
                EdgeCubelet::UR => one_off_cost(cube.corners.ufr, CornerOrientation::CW),
                EdgeCubelet::FR => one_off_cost(cube.corners.ufr, CornerOrientation::CCW),
                _ => 2,
            }
        }
        EdgeCubelet::UL => {
            // rotation is y (or y', i can never remember the notation) from UF
            match source_position {
                EdgeCubelet::UL => in_place_cost(cube.corners.ubl, cube.corners.ufl),
                EdgeCubelet::DR => 3,
                EdgeCubelet::UB => one_off_cost(cube.corners.ubl, CornerOrientation::CCW),
                EdgeCubelet::BL => one_off_cost(cube.corners.ubl, CornerOrientation::CW),
                EdgeCubelet::UF => one_off_cost(cube.corners.ufl, CornerOrientation::CW),
                EdgeCubelet::FL => one_off_cost(cube.corners.ufl, CornerOrientation::CCW),
                _ => 2,
            }
        }
        EdgeCubelet::UR => {
            // rotation is y (or y', i can never remember the notation) from UF
            match source_position {
                EdgeCubelet::UR => in_place_cost(cube.corners.ufr, cube.corners.ubr),
                EdgeCubelet::DL => 3,
                EdgeCubelet::UF => one_off_cost(cube.corners.ufr, CornerOrientation::CCW),
                EdgeCubelet::FR => one_off_cost(cube.corners.ufr, CornerOrientation::CW),
                EdgeCubelet::UB => one_off_cost(cube.corners.ubr, CornerOrientation::CW),
                EdgeCubelet::BR => one_off_cost(cube.corners.ubr, CornerOrientation::CCW),
                _ => 2,
            }
        }
        EdgeCubelet::UB => {
            // rotation is basically y2 from UF
            match source_position {
                EdgeCubelet::UB => in_place_cost(cube.corners.ubr, cube.corners.ubl),
                EdgeCubelet::DF => 3,
                EdgeCubelet::UR => one_off_cost(cube.corners.ubr, CornerOrientation::CCW),
                EdgeCubelet::BR => one_off_cost(cube.corners.ubr, CornerOrientation::CW),
                EdgeCubelet::UL => one_off_cost(cube.corners.ubl, CornerOrientation::CW),
                EdgeCubelet::BL => one_off_cost(cube.corners.ubl, CornerOrientation::CCW),
                _ => 2,
            }
        }
        EdgeCubelet::DF => {
            // rotation is basically z2 from UF
            match source_position {
                EdgeCubelet::DF => in_place_cost(cube.corners.dfl, cube.corners.dfr),
                EdgeCubelet::UB => 3,
                EdgeCubelet::DR => one_off_cost(cube.corners.dfl, CornerOrientation::CCW),
                EdgeCubelet::FR => one_off_cost(cube.corners.dfl, CornerOrientation::CW),
                EdgeCubelet::DL => one_off_cost(cube.corners.dfr, CornerOrientation::CW),
                EdgeCubelet::FL => one_off_cost(cube.corners.dfr, CornerOrientation::CCW),
                _ => 2,
            }
        }
        EdgeCubelet::DL => {
            // rotation is basically z2 from UL
            match source_position {
                EdgeCubelet::DL => in_place_cost(cube.corners.dfl, cube.corners.dbl),
                EdgeCubelet::UR => 3,
                EdgeCubelet::DF => one_off_cost(cube.corners.dfl, CornerOrientation::CCW),
                EdgeCubelet::FL => one_off_cost(cube.corners.dfl, CornerOrientation::CW),
                EdgeCubelet::DB => one_off_cost(cube.corners.dbl, CornerOrientation::CW),
                EdgeCubelet::BL => one_off_cost(cube.corners.dbl, CornerOrientation::CCW),
                _ => 2,
            }
        }
        EdgeCubelet::DR => {
            // rotation is basically z2 from UR
            match source_position {
                EdgeCubelet::DR => in_place_cost(cube.corners.dbr, cube.corners.dfr),
                EdgeCubelet::UL => 3,
                EdgeCubelet::DB => one_off_cost(cube.corners.dbr, CornerOrientation::CCW),
                EdgeCubelet::BR => one_off_cost(cube.corners.dbr, CornerOrientation::CW),
                EdgeCubelet::DF => one_off_cost(cube.corners.dfr, CornerOrientation::CW),
                EdgeCubelet::FR => one_off_cost(cube.corners.dfr, CornerOrientation::CCW),
                _ => 2,
            }
        }
        EdgeCubelet::DB => {
            // rotation is basically z2 from UB, or x2 from UF
            match source_position {
                EdgeCubelet::DB => in_place_cost(cube.corners.dbl, cube.corners.dbr),
                EdgeCubelet::UF => 3,
                // from UL, FL, UR, FR
                EdgeCubelet::DL => one_off_cost(cube.corners.dbl, CornerOrientation::CCW),
                EdgeCubelet::BL => one_off_cost(cube.corners.dbl, CornerOrientation::CW),
                EdgeCubelet::DR => one_off_cost(cube.corners.dbr, CornerOrientation::CW),
                EdgeCubelet::BR => one_off_cost(cube.corners.dbr, CornerOrientation::CCW),
                _ => 2,
            }
        }
        // for mid layer, when choosing which face gets rotated to top, prefer F/B over L/R
        EdgeCubelet::FL => match source_position {
            EdgeCubelet::FL => in_place_cost(cube.corners.dbl, cube.corners.dbr),
            EdgeCubelet::BR => 3,
            EdgeCubelet::UF => one_off_cost(cube.corners.ufl, CornerOrientation::CCW),
            EdgeCubelet::UL => one_off_cost(cube.corners.ufl, CornerOrientation::CW),
            EdgeCubelet::DF => one_off_cost(cube.corners.dfl, CornerOrientation::CW),
            EdgeCubelet::DL => one_off_cost(cube.corners.dfl, CornerOrientation::CCW),
            _ => 2,
        },
        EdgeCubelet::FR => match source_position {
            EdgeCubelet::FR => in_place_cost(cube.corners.dfr, cube.corners.ufr),
            EdgeCubelet::BL => 3,
            EdgeCubelet::DF => one_off_cost(cube.corners.dfr, CornerOrientation::CCW),
            EdgeCubelet::DR => one_off_cost(cube.corners.dfr, CornerOrientation::CW),
            EdgeCubelet::UF => one_off_cost(cube.corners.ufr, CornerOrientation::CW),
            EdgeCubelet::UR => one_off_cost(cube.corners.ufr, CornerOrientation::CCW),
            _ => 2,
        },
        EdgeCubelet::BL => match source_position {
            EdgeCubelet::BL => in_place_cost(cube.corners.dbl, cube.corners.ubl),
            EdgeCubelet::FR => 3,
            EdgeCubelet::DB => one_off_cost(cube.corners.dbl, CornerOrientation::CCW),
            EdgeCubelet::DL => one_off_cost(cube.corners.dbl, CornerOrientation::CW),
            EdgeCubelet::UB => one_off_cost(cube.corners.ubl, CornerOrientation::CW),
            EdgeCubelet::UL => one_off_cost(cube.corners.ubl, CornerOrientation::CCW),
            _ => 2,
        },
        EdgeCubelet::BR => match source_position {
            EdgeCubelet::BR => in_place_cost(cube.corners.ubr, cube.corners.dbr),
            EdgeCubelet::FL => 3,
            EdgeCubelet::UB => one_off_cost(cube.corners.ubr, CornerOrientation::CCW),
            EdgeCubelet::UR => one_off_cost(cube.corners.ubr, CornerOrientation::CW),
            EdgeCubelet::DB => one_off_cost(cube.corners.dbr, CornerOrientation::CW),
            EdgeCubelet::DR => one_off_cost(cube.corners.dbr, CornerOrientation::CCW),
            _ => 2,
        },
    }
}

fn dist_heuristic(cube: &RediCube) -> usize {
    // some ideas:
    // 1. given an edge and a goal position, there is a minimum number of moves required to get
    //      it to that position; one position is correct, four are one away, six are two away,
    //      and one is three away
    // 2. if an edge is in the right place but the corner is wrong, it takes at least two moves
    //      to fix it (4 if both corners are wrong); this is not totally obvious but it is true
    // 3. if an edge is one away, and the relevant corner is at the incorrect orientation, it takes
    //      at least three moves to fix it; this is true for basically the same reason
    // each actual move can move at most three edges at a time, so (super sloppy) can add up
    // all those numbers, divide by three (round up) and that's the minimum number of twists to
    // solve the whole puzzle.
    //
    // Tradeoff is correctly upping the cost helps discard bad paths, but if the heuristic is
    // expensive to compute, it's better not to bother; exploring more paths very quickly can be
    // better than exploring fewer paths more slowly (operative word "can be")
    //
    // For a "random" puzzle these rules give costs as follows:
    //  - Rule 1 only:  19 edge moves (cost rounds up to 7)
    //                      1*0 + 4*1 + 6*2 + 1*3
    //  - Rules 1-2:    21 + 2/3 edge moves (cost rounds up to 8)
    //                      1*(0 + 2*4/9 + 4*4/9) + 4*1 + 6*2 + 1*3
    //  - Rules 1-3:    27 (cost of 9)
    //                      1*(0 + 2*4/9 + 4*4/9) + 4*(1*1/3 + 3*2/3) + 6*2 + 1*3
    //
    // experimentally; define a "hit rate" as the percentage of states where this gives a higher
    // cost than the bounded cache's fallback depth.
    //
    // against cache of depth 7 (so heuristic cost must be at least 9 to matter):
    // - using just rule 1, we get about 1% hit rate
    // - using rules 1 and 2, we get about 9-10% hit rate
    //
    // against cache of depth 8 (so heuristic cost must be at least 10 to matter):
    // - using just rule 1, we get less than 0.1% hit rate (essentially nothing)
    // - using rules 1 and 2, we get about 2% hit rate
    let mut total_cost = 0;

    // upper layer
    total_cost += dist(EdgeCubelet::UF, cube.edges.uf, cube);
    total_cost += dist(EdgeCubelet::UL, cube.edges.ul, cube);
    total_cost += dist(EdgeCubelet::UR, cube.edges.ur, cube);
    total_cost += dist(EdgeCubelet::UB, cube.edges.ub, cube);

    // lower layer
    total_cost += dist(EdgeCubelet::DF, cube.edges.df, cube);
    total_cost += dist(EdgeCubelet::DL, cube.edges.dl, cube);
    total_cost += dist(EdgeCubelet::DR, cube.edges.dr, cube);
    total_cost += dist(EdgeCubelet::DB, cube.edges.db, cube);

    // mid layer
    total_cost += dist(EdgeCubelet::FL, cube.edges.fl, cube);
    total_cost += dist(EdgeCubelet::FR, cube.edges.fr, cube);
    total_cost += dist(EdgeCubelet::BL, cube.edges.bl, cube);
    total_cost += dist(EdgeCubelet::BR, cube.edges.br, cube);

    // divide by three, rounded up
    (total_cost + 2) / 3
}

pub fn make_heuristic(max_depth: usize) -> impl Heuristic<RediCube> {
    let cache = bounded_cache::<RediCube>(max_depth);
    RediHeuristic {
        bounded_cache: cache,
        #[cfg(feature = "hit_rate")]
        heuristic_hits: Default::default(),
        #[cfg(feature = "hit_rate")]
        heuristic_misses: Default::default(),
    }
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
    use crate::moves::CornerTwistAmt::{Ccw, Cw};

    use super::*;

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

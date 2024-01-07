use derive_more::Display;
use enum_iterator::{all, Sequence};
use rand::Rng;

use crate::cubesearch::State;
use crate::idasearch::Solvable;
use crate::moves::CanReverse;
use crate::orientations::EdgeOrientation;
use crate::random_helpers;
use crate::scrambles::RandomInit;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Sequence)]
enum CornerCubelet {
    UL,
    UR,
    DL,
    DR,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Floppy1x3x3 {
    ul: CornerCubelet,
    ur: CornerCubelet,
    dl: CornerCubelet,
    dr: CornerCubelet,
    rc_solved: EdgeOrientation,
    uc_solved: EdgeOrientation,
    dc_solved: EdgeOrientation,
    lc_solved: EdgeOrientation,
}

impl Floppy1x3x3 {
    fn solved() -> Self {
        Self {
            ul: CornerCubelet::UL,
            ur: CornerCubelet::UR,
            dl: CornerCubelet::DL,
            dr: CornerCubelet::DR,
            rc_solved: EdgeOrientation::Normal,
            lc_solved: EdgeOrientation::Normal,
            uc_solved: EdgeOrientation::Normal,
            dc_solved: EdgeOrientation::Normal,
        }
    }

    fn u2(&self) -> Self {
        Self {
            ul: self.ur,
            ur: self.ul,
            uc_solved: self.uc_solved.flipped(),
            ..*self
        }
    }

    fn d2(&self) -> Self {
        Self {
            dl: self.dr,
            dr: self.dl,
            dc_solved: self.dc_solved.flipped(),
            ..*self
        }
    }

    fn r2(&self) -> Self {
        Self {
            ur: self.dr,
            dr: self.ur,
            rc_solved: self.rc_solved.flipped(),
            ..*self
        }
    }

    fn l2(&self) -> Self {
        Self {
            ul: self.dl,
            dl: self.ul,
            lc_solved: self.lc_solved.flipped(),
            ..*self
        }
    }
}

impl State for Floppy1x3x3 {
    type UniqueKey = Self;

    fn neighbors<Recv>(&self, to_add: &mut Recv)
    where
        Recv: FnMut(Self),
    {
        to_add(self.l2());
        to_add(self.r2());
        to_add(self.u2());
        to_add(self.d2());
    }

    fn start() -> Self {
        Self::solved()
    }

    fn uniq_key(&self) -> Self::UniqueKey {
        *self
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Hash)]
pub enum Move {
    R2,
    U2,
    D2,
    L2,
}

impl CanReverse for Move {
    fn reverse(&self) -> Self {
        // all moves are self-inverse, which is nice
        *self
    }
}

impl RandomInit for Floppy1x3x3 {
    fn random_state<R: Rng>(r: &mut R) -> Self {
        // the total parity of the position permutation ...
        let (cubelets, pos_parity) = random_helpers::shuffle_any(r, all::<CornerCubelet>());
        // ... must match the total parity of the center orientations
        let orientations = random_helpers::flips_with_parity(r, 4, pos_parity);

        Self {
            ul: cubelets[0],
            ur: cubelets[1],
            dl: cubelets[2],
            dr: cubelets[3],

            lc_solved: orientations[0],
            rc_solved: orientations[1],
            uc_solved: orientations[2],
            dc_solved: orientations[3],
        }
    }
}

impl Solvable for Floppy1x3x3 {
    type Move = Move;

    fn is_solved(&self) -> bool {
        *self == Self::solved()
    }

    fn available_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        [Move::D2, Move::U2, Move::L2, Move::R2]
    }

    fn is_redundant(last_move: Self::Move, next_move: Self::Move) -> bool {
        last_move == next_move
    }

    fn apply(&self, m: Self::Move) -> Self {
        match m {
            Move::R2 => self.r2(),
            Move::U2 => self.u2(),
            Move::D2 => self.d2(),
            Move::L2 => self.l2(),
        }
    }

    fn max_fuel() -> usize {
        8
    }
}

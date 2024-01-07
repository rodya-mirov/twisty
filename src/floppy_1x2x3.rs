use derive_more::Display;
use rand::prelude::SliceRandom;
use rand::Rng;

use crate::cubesearch::SimpleStartState;
use crate::idasearch::Solvable;
use crate::moves::CanReverse;
use crate::orientations::EdgeOrientation;
use crate::scrambles::RandomInit;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
enum CornerCubelet {
    UL,
    UR,
    DL,
    DR,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Floppy1x2x3 {
    ul: CornerCubelet,
    ur: CornerCubelet,
    dl: CornerCubelet,
    dr: CornerCubelet,
    rc_solved: EdgeOrientation,
}

impl Floppy1x2x3 {
    fn solved() -> Self {
        Self {
            ul: CornerCubelet::UL,
            ur: CornerCubelet::UR,
            dl: CornerCubelet::DL,
            dr: CornerCubelet::DR,
            rc_solved: EdgeOrientation::Normal,
        }
    }

    fn u2(&self) -> Self {
        Self {
            ul: self.ur,
            ur: self.ul,
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

    fn d2(&self) -> Self {
        Self {
            dl: self.dr,
            dr: self.dl,
            ..*self
        }
    }
}

impl SimpleStartState for Floppy1x2x3 {
    type UniqueKey = Self;

    fn start() -> Self {
        Self::solved()
    }

    fn uniq_key(&self) -> Self::UniqueKey {
        *self
    }
}

impl RandomInit for Floppy1x2x3 {
    fn random_state<R: Rng>(r: &mut R) -> Self {
        // any permutation of the cubelets is possible, so we can just shuffle the possibilities
        let mut cubelets = [
            CornerCubelet::UL,
            CornerCubelet::UR,
            CornerCubelet::DL,
            CornerCubelet::DR,
        ];
        cubelets.shuffle(r);

        Floppy1x2x3 {
            ul: cubelets[0],
            ur: cubelets[1],
            dl: cubelets[2],
            dr: cubelets[3],
            rc_solved: EdgeOrientation::random(r),
        }
    }
}

/// The moves for a Floppy 1x2x3 are just R/U/D, as half turns
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Display)]
pub enum Move {
    R2,
    U2,
    D2,
}

impl CanReverse for Move {
    fn reverse(&self) -> Self {
        // all moves are self-inverse, which is neat
        *self
    }
}

impl Solvable for Floppy1x2x3 {
    type Move = Move;

    fn is_solved(&self) -> bool {
        self == &Floppy1x2x3::solved()
    }

    fn is_redundant(last_move: Self::Move, next_move: Self::Move) -> bool {
        last_move == next_move
    }

    fn available_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        [Move::R2, Move::U2, Move::D2]
    }

    fn apply(&self, m: Self::Move) -> Self {
        match m {
            Move::U2 => self.u2(),
            Move::R2 => self.r2(),
            Move::D2 => self.d2(),
        }
    }

    fn max_fuel() -> usize {
        6
    }
}

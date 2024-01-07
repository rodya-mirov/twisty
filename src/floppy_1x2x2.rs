use derive_more::Display;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::cubesearch::SimpleStartState;
use crate::idasearch::Solvable;
use crate::moves::CanReverse;
use crate::scrambles::RandomInit;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
enum CornerCubelet {
    UL,
    UR,
    DR,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Floppy1x2x2 {
    ul: CornerCubelet,
    ur: CornerCubelet,
    dr: CornerCubelet,
}

impl Floppy1x2x2 {
    fn solved() -> Self {
        Self {
            ul: CornerCubelet::UL,
            ur: CornerCubelet::UR,
            dr: CornerCubelet::DR,
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
            ..*self
        }
    }
}

impl SimpleStartState for Floppy1x2x2 {
    type UniqueKey = Self;

    fn start() -> Self {
        Self::solved()
    }

    fn uniq_key(&self) -> Self::UniqueKey {
        *self
    }
}

impl RandomInit for Floppy1x2x2 {
    fn random_state<R: Rng>(r: &mut R) -> Self {
        // any permutation of the cubelets is possible, so we can just shuffle the possibilities
        let mut cubelets = [CornerCubelet::UL, CornerCubelet::UR, CornerCubelet::DR];
        cubelets.shuffle(r);

        Floppy1x2x2 {
            ul: cubelets[0],
            ur: cubelets[1],
            dr: cubelets[2],
        }
    }
}

/// The moves for a Floppy 1x2x2 are just R/U, as half turns
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Display)]
pub enum Move {
    R2,
    U2,
}

impl CanReverse for Move {
    fn reverse(&self) -> Self {
        match self {
            Move::R2 => Move::R2,
            Move::U2 => Move::U2,
        }
    }
}

impl Solvable for Floppy1x2x2 {
    type Move = Move;

    fn is_solved(&self) -> bool {
        self == &Floppy1x2x2::solved()
    }

    fn is_redundant(last_move: Self::Move, next_move: Self::Move) -> bool {
        last_move == next_move
    }

    fn available_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        [Move::R2, Move::U2]
    }

    fn apply(&self, m: Self::Move) -> Self {
        match m {
            Move::U2 => self.u2(),
            Move::R2 => self.r2(),
        }
    }

    fn max_fuel() -> usize {
        3
    }
}

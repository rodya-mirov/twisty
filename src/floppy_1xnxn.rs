use std::mem::swap;

use derive_more::Display;
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

/// Strictly speaking, this describes a 1 x M+2 x N+2 floppy -- MxN is the dimension of the center.
/// So M=N=0 gives you the Z-cube, M=N=1 gives you the super floppy, M=0 N=1 gives you the 1x2x3
/// floppy, and so on. You cannot get ideal cubes (1x1xZ dimensional) this way.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Floppy1xMxN<const H: usize, const W: usize> {
    // corners can never be disoriented in a floppy, only out of place
    ul: CornerCubelet,
    ur: CornerCubelet,
    dr: CornerCubelet,

    // centers can be the right or wrong color (that is, flipped or not), but that's all
    // "true" means correct
    // index as [x][y] where [0][0] is the upper-right corner of the center
    centers: [[bool; W]; H], // index as [y][x]

    // edges can be the right or wrong position (left-on-right or not) and they can also be
    // flipped (disoriented), so they have two bits of information each.
    // for all of this "true" means "correct" and "false" means "wrong"

    // index these as [y]; note that [0] is the top layer of the center, and so on, so that [H-1]
    // is the bottom layer of the center.
    left_edge_pos: [bool; H],
    left_edge_orr: [bool; H],

    right_edge_pos: [bool; H],
    right_edge_orr: [bool; H],

    // index these as [x]; note that [0] is the right layer of the center, and so on, so that [W-1]
    // is the left layer of the center.
    top_edge_pos: [bool; W],
    top_edge_orr: [bool; W],

    bot_edge_pos: [bool; W],
    bot_edge_orr: [bool; W],
}

impl<const H: usize, const W: usize> Floppy1xMxN<H, W> {
    fn solved() -> Self {
        Self {
            ul: CornerCubelet::UL,
            ur: CornerCubelet::UR,
            dr: CornerCubelet::DR,

            centers: [[true; W]; H],

            left_edge_orr: [true; H],
            left_edge_pos: [true; H],

            right_edge_orr: [true; H],
            right_edge_pos: [true; H],

            top_edge_orr: [true; W],
            top_edge_pos: [true; W],

            bot_edge_orr: [true; W],
            bot_edge_pos: [true; W],
        }
    }

    /// Do a U2(y) move. y=0 means just the top slice (no effect on centers). y=1 means the top
    /// and the top layer of centers, and so on; y=H means everything but the bottom.
    /// There is a requirement that y<=H.
    fn u2(&self, num_center_rows: usize) -> Self {
        debug_assert!(num_center_rows <= H);

        let mut out = *self;

        // top pieces -- note they have position but not orientation
        swap(&mut out.ul, &mut out.ur);

        // top edges -- note they have position (which is swapped) and orientation (which is
        // swapped and flipped)
        for x in 0..(W / 2) {
            let x_opp = W - 1 - x;

            out.top_edge_pos.swap(x, x_opp);
            out.top_edge_orr.swap(x, x_opp);
        }

        for x in 0..W {
            out.top_edge_orr[x] = !out.top_edge_orr[x];
        }

        for y in 0..num_center_rows {
            // left/right edge swaps
            swap(&mut out.left_edge_orr[y], &mut out.right_edge_orr[y]);
            swap(&mut out.left_edge_pos[y], &mut out.right_edge_pos[y]);

            out.left_edge_pos[y] = !out.left_edge_pos[y];
            out.right_edge_pos[y] = !out.right_edge_pos[y];

            out.left_edge_orr[y] = !out.left_edge_orr[y];
            out.right_edge_orr[y] = !out.right_edge_orr[y];

            // and center swaps
            for x in 0..(W / 2) {
                let x_opp = W - 1 - x;
                out.centers[y].swap(x, x_opp);
            }

            for x in 0..W {
                out.centers[y][x] = !out.centers[y][x];
            }
        }

        out
    }

    /// Do a R2(x) move. x=0 means just the right slice (no effect on centers). x=1 means the
    /// right edge and the right-most edge of the center, and so on. x=W means everything
    /// but the left edge. There is a requirement that x<=W.
    fn r2(&self, num_center_cols: usize) -> Self {
        debug_assert!(num_center_cols <= W);

        let mut out = *self;

        // right corners -- note they have position but not orientation
        swap(&mut out.ur, &mut out.dr);

        // right edges -- note they have position (which is swapped) and orientation (which is
        // swapped and flipped)
        for y in 0..(H / 2) {
            let y_opp = H - 1 - y;

            out.right_edge_orr.swap(y, y_opp);
            out.right_edge_pos.swap(y, y_opp);
        }

        for y in 0..H {
            out.right_edge_orr[y] = !out.right_edge_orr[y];
        }

        // then go from the right across, flipping inner columns
        for x in 0..num_center_cols {
            // swapping top and bottom edges; note we swap AND negate each flag here
            swap(&mut out.top_edge_orr[x], &mut out.bot_edge_orr[x]);
            swap(&mut out.top_edge_pos[x], &mut out.bot_edge_pos[x]);

            out.top_edge_pos[x] = !out.top_edge_pos[x];
            out.bot_edge_pos[x] = !out.bot_edge_pos[x];

            out.top_edge_orr[x] = !out.top_edge_orr[x];
            out.bot_edge_orr[x] = !out.bot_edge_orr[x];

            // and center pieces
            for y in 0..(H / 2) {
                let y_opp = H - 1 - y;

                let temp = out.centers[y][x];
                out.centers[y][x] = out.centers[y_opp][x];
                out.centers[y_opp][x] = temp;
            }

            for y in 0..H {
                out.centers[y][x] = !out.centers[y][x];
            }
        }

        out
    }
}

impl<const H: usize, const W: usize> SimpleStartState for Floppy1xMxN<H, W> {
    type UniqueKey = Self;

    fn start() -> Self {
        Self::solved()
    }

    fn uniq_key(&self) -> Self::UniqueKey {
        *self
    }
}

impl<const H: usize, const W: usize> RandomInit for Floppy1xMxN<H, W> {
    fn random_state<R: Rng>(_r: &mut R) -> Self {
        todo!("I actually know how to do this now, so we could do it")
    }
}

/// The moves for a Floppy 1x2x2 are just R/U, as half turns
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Display)]
pub enum Move {
    R2(usize),
    U2(usize),
}

impl CanReverse for Move {
    fn reverse(&self) -> Self {
        *self
    }
}

impl<const H: usize, const W: usize> Solvable for Floppy1xMxN<H, W> {
    type Move = Move;

    fn is_solved(&self) -> bool {
        self == &Floppy1xMxN::solved()
    }

    fn available_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        let u_moves = (0..=H).map(Move::U2);
        let r_moves = (0..=W).map(Move::R2);

        u_moves.chain(r_moves)
    }

    fn is_redundant(last_move: Self::Move, next_move: Self::Move) -> bool {
        last_move == next_move
    }

    fn apply(&self, m: Self::Move) -> Self {
        match m {
            Move::U2(y) => self.u2(y),
            Move::R2(x) => self.r2(x),
        }
    }

    fn max_fuel() -> usize {
        10
    }
}

#[cfg(test)]
mod tests_133;

#[cfg(test)]
mod tests_134;

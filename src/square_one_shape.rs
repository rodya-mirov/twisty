use std::mem::swap;

use crate::cubesearch::State;
use crate::idasearch::Solvable;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Ord, PartialOrd)]
enum Piecelet {
    // a self-contained piecelet (30 degrees of fill)
    Edge,
    // the start of a corner piecelet (first half of 60 degrees of fill), going in clockwise order
    StartCorner,
    // the end of a corner piecelet (second half of 60 degrees of fill), going in clockwise order
    EndCorner,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Ord, PartialOrd)]
pub struct SquareOneShape {
    // clockwise from the front slice point, as viewed from the top
    top: [Piecelet; 12],
    // counterclockwise from the front slice point, as viewed from the top
    bot: [Piecelet; 12],
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Ord, PartialOrd)]
pub enum Move {
    // actually, from 1 to 11
    U(u8),
    // actually, from 1 to 11
    D(u8),
    Slice,
}

impl SquareOneShape {
    fn u(&self, amt: usize) -> Self {
        debug_assert!(amt < 12);

        let mut out = *self;
        out.top.rotate_right(amt);
        out
    }

    fn d(&self, amt: usize) -> Self {
        debug_assert!(amt < 12);

        let mut out = *self;
        out.bot.rotate_right(amt);
        out
    }

    fn slice(&self) -> Self {
        let mut out = *self;

        swap(&mut out.top[6], &mut out.bot[0]);
        swap(&mut out.top[7], &mut out.bot[1]);
        swap(&mut out.top[8], &mut out.bot[2]);
        swap(&mut out.top[9], &mut out.bot[3]);
        swap(&mut out.top[10], &mut out.bot[4]);
        swap(&mut out.top[11], &mut out.bot[5]);

        out
    }

    fn can_slice(&self) -> bool {
        self.top[0] != Piecelet::EndCorner
            && self.top[6] != Piecelet::EndCorner
            && self.bot[0] != Piecelet::EndCorner
            && self.bot[6] != Piecelet::EndCorner
    }
}

impl Solvable for SquareOneShape {
    type Move = Move;

    fn is_solved(&self) -> bool {
        self == &Self::start()
    }

    fn apply(&self, m: Self::Move) -> Self {
        match m {
            Move::U(amt) => self.u(amt as usize),
            Move::D(amt) => self.d(amt as usize),
            // note we don't actually check here if this is permissible, needs to be checked
            // in advance
            Move::Slice => self.slice(),
        }
    }

    fn available_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        (1..=11)
            .map(Move::U)
            .chain((1..=11).map(Move::D))
            .chain(std::iter::once(Move::Slice).filter(|_| self.can_slice()))
    }

    fn is_redundant(last_move: Self::Move, next_move: Self::Move) -> bool {
        match last_move {
            Move::U(_) => matches!(next_move, Move::U(_)),
            Move::D(_) => matches!(next_move, Move::U(_) | Move::D(_)),
            Move::Slice => next_move == Move::Slice,
        }
    }

    fn max_fuel() -> usize {
        10
    }
}

impl State for SquareOneShape {
    // there are 16 pieces total and only their order matters (top, in clockwise order, then
    // bottom, in clockwise order), so we only need 16 bits (0 for edge, 1 for corner)
    type UniqueKey = u16;

    fn uniq_key(&self) -> u16 {
        let mut out: u16 = 0;

        for e in self.top {
            match e {
                Piecelet::Edge => {
                    out <<= 1;
                }
                Piecelet::StartCorner => {
                    out = (out << 1) + 1;
                }
                Piecelet::EndCorner => {}
            }
        }

        out
    }

    fn neighbors<Recv>(&self, to_add: &mut Recv)
    where
        Recv: FnMut(Self),
    {
        for amt in 1..=11 {
            to_add(self.u(amt));
            to_add(self.d(amt));
        }

        if self.can_slice() {
            to_add(self.slice());
        }
    }

    fn start() -> Self {
        Self {
            top: [
                Piecelet::StartCorner,
                Piecelet::EndCorner,
                Piecelet::Edge,
                Piecelet::StartCorner,
                Piecelet::EndCorner,
                Piecelet::Edge,
                Piecelet::StartCorner,
                Piecelet::EndCorner,
                Piecelet::Edge,
                Piecelet::StartCorner,
                Piecelet::EndCorner,
                Piecelet::Edge,
            ],
            bot: [
                Piecelet::Edge,
                Piecelet::StartCorner,
                Piecelet::EndCorner,
                Piecelet::Edge,
                Piecelet::StartCorner,
                Piecelet::EndCorner,
                Piecelet::Edge,
                Piecelet::StartCorner,
                Piecelet::EndCorner,
                Piecelet::Edge,
                Piecelet::StartCorner,
                Piecelet::EndCorner,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use Piecelet::*;

    use super::*;

    #[test]
    fn u_one_test() {
        let actual = SquareOneShape::start().u(1);
        let expected = SquareOneShape {
            // should now start with edge, leftwards from slice point
            top: [
                Edge,
                StartCorner,
                EndCorner,
                Edge,
                StartCorner,
                EndCorner,
                Edge,
                StartCorner,
                EndCorner,
                Edge,
                StartCorner,
                EndCorner,
            ],
            bot: SquareOneShape::start().bot,
        };

        assert_eq!(actual, expected)
    }

    #[test]
    fn u_two_test() {
        let actual = SquareOneShape::start().u(2);
        let expected = SquareOneShape {
            // should now start with end corner, leftwards from slice point
            top: [
                EndCorner,
                Edge,
                StartCorner,
                EndCorner,
                Edge,
                StartCorner,
                EndCorner,
                Edge,
                StartCorner,
                EndCorner,
                Edge,
                StartCorner,
            ],
            bot: SquareOneShape::start().bot,
        };

        assert_eq!(actual, expected)
    }

    #[test]
    fn d_one_test() {
        let actual = SquareOneShape::start().d(1);

        let expected = SquareOneShape {
            top: SquareOneShape::start().top,
            bot: [
                EndCorner,
                Edge,
                StartCorner,
                EndCorner,
                Edge,
                StartCorner,
                EndCorner,
                Edge,
                StartCorner,
                EndCorner,
                Edge,
                StartCorner,
            ],
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn d_two_test() {
        let actual = SquareOneShape::start().d(2);

        let expected = SquareOneShape {
            top: SquareOneShape::start().top,
            bot: [
                StartCorner,
                EndCorner,
                Edge,
                StartCorner,
                EndCorner,
                Edge,
                StartCorner,
                EndCorner,
                Edge,
                StartCorner,
                EndCorner,
                Edge,
            ],
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn slice_test() {
        let actual = SquareOneShape::start().slice();

        let expected = SquareOneShape {
            top: [
                // unaffected portion
                StartCorner,
                EndCorner,
                Edge,
                StartCorner,
                EndCorner,
                Edge,
                // flipped portion
                Edge,
                StartCorner,
                EndCorner,
                Edge,
                StartCorner,
                EndCorner,
            ],
            bot: [
                // flipped portion
                StartCorner,
                EndCorner,
                Edge,
                StartCorner,
                EndCorner,
                Edge,
                // unaffected portion
                Edge,
                StartCorner,
                EndCorner,
                Edge,
                StartCorner,
                EndCorner,
            ],
        };

        assert_eq!(expected, actual);
    }

    #[test]
    fn complex_move_test() {
        let actual = SquareOneShape::start().u(1).d(2).slice().u(3).d(6).slice();

        let expected = SquareOneShape {
            top: [
                StartCorner,
                EndCorner,
                Edge,
                Edge,
                StartCorner,
                EndCorner,
                StartCorner,
                EndCorner,
                Edge,
                StartCorner,
                EndCorner,
                Edge,
            ],
            bot: [
                Edge,
                StartCorner,
                EndCorner,
                StartCorner,
                EndCorner,
                Edge,
                Edge,
                StartCorner,
                EndCorner,
                Edge,
                StartCorner,
                EndCorner,
            ],
        };

        assert_eq!(actual, expected);
    }
}

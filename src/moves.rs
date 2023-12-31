//! Traits and reusable data structures for describing moves.
use derive_more::Display;
use enum_iterator::Sequence;

pub trait CanReverse: Sized {
    fn reverse(&self) -> Self;
}

/// Typical moves for a cube twist -- one step, two steps, rev (three steps)
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display, Sequence)]
pub enum CubeMoveAmt {
    #[display(fmt = "")]
    One,
    #[display(fmt = "2")]
    Two,
    #[display(fmt = "'")]
    Rev,
}

impl CanReverse for CubeMoveAmt {
    fn reverse(&self) -> Self {
        match self {
            CubeMoveAmt::One => CubeMoveAmt::Rev,
            CubeMoveAmt::Two => CubeMoveAmt::Two,
            CubeMoveAmt::Rev => CubeMoveAmt::One,
        }
    }
}

/// Typical moves for a cube twist -- one step, two steps, rev (three steps)
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display, Sequence)]
pub enum CornerTwistAmt {
    #[display(fmt = "")]
    Cw,
    #[display(fmt = "'")]
    Ccw,
}

impl CanReverse for CornerTwistAmt {
    fn reverse(&self) -> Self {
        match self {
            CornerTwistAmt::Cw => CornerTwistAmt::Ccw,
            CornerTwistAmt::Ccw => CornerTwistAmt::Cw,
        }
    }
}

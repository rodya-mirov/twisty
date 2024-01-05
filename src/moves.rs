//! Traits and reusable data structures for describing moves.
use derive_more::Display;

pub trait CanReverse: Sized {
    fn reverse(&self) -> Self;
}

/// Typical moves for a cube twist -- one step, two steps, rev (three steps)
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display)]
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

//! Traits and reusable data structures for describing moves.

pub trait CanReverse: Sized {
    fn reverse(&self) -> Self;
}

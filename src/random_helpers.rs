//! Reusable functionality for shuffles, scrambles, and other random initalization logic.

use crate::orientations::EdgeOrientation;
use derive_more::Display;
use rand::Rng;

#[derive(Ord, PartialEq, Eq, PartialOrd, Debug, Display)]
pub enum TwoParity {
    Even,
    Odd,
}

mod permutations;

/// Shuffles the given array arbitrarily, and returns the parity of the resulting permutation
pub fn shuffle_any<R: Rng, T: Copy>(rng: &mut R, arr: &[T]) -> (Vec<T>, TwoParity) {
    let permutation = permutations::any_permutation(rng, arr.len());

    let parity = permutation.parity();

    let shuffled: Vec<T> = (0..arr.len()).map(|i| arr[permutation.apply(i)]).collect();

    (shuffled, parity)
}

#[allow(unused)]
pub fn shuffle_with_parity<R: Rng, T: Copy>(rng: &mut R, arr: &[T], desired: TwoParity) -> Vec<T> {
    let permutation = permutations::with_parity(rng, arr.len(), desired);

    let shuffled: Vec<T> = (0..arr.len()).map(|i| arr[permutation.apply(i)]).collect();

    shuffled
}

pub fn flips_with_parity<R: Rng>(
    rng: &mut R,
    len: usize,
    desired: TwoParity,
) -> Vec<EdgeOrientation> {
    if len == 0 && desired == TwoParity::Odd {
        panic!("Can't flip nothing and make it odd")
    }

    let mut out: Vec<EdgeOrientation> =
        (0..len - 1).map(|_| EdgeOrientation::random(rng)).collect();

    let current_parity = out
        .iter()
        .filter(|e| **e == EdgeOrientation::Flipped)
        .count()
        % 2;
    let current_parity = match current_parity {
        0 => TwoParity::Even,
        1 => TwoParity::Odd,
        other => unreachable!("We modded out by 2, should get 0 or 1; got {other}"),
    };

    if current_parity == desired {
        out.push(EdgeOrientation::Normal);
    } else {
        out.push(EdgeOrientation::Flipped);
    }

    assert_eq!(out.len(), len);

    out
}

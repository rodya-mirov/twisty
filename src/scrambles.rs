use crate::idasearch;
use crate::idasearch::{Heuristic, Solvable, SolveError};
use crate::moves::CanReverse;
use rand::Rng;
use std::fmt::Display;

pub trait RandomInit: Sized {
    fn random_state<R: Rng>(r: &mut R) -> Self;
}

pub fn random_scramble<R: Rng, M: CanReverse, State: RandomInit + Solvable<Move = M>, H: Heuristic<State>>(
    rng: &mut R,
    h: &H,
) -> Result<Vec<M>, SolveError> {
    let s = State::random_state(rng);

    // solve the scramble
    let solution: Vec<M> = idasearch::solve(&s, h)?;

    // reverse the order and the moves themselves
    let out = solution.into_iter().rev().map(|m| m.reverse()).collect();
    Ok(out)
}

pub fn random_scramble_string<
    R: Rng,
    M: CanReverse + Display,
    State: RandomInit + Solvable<Move = M>,
    H: Heuristic<State>,
>(
    rng: &mut R,
    h: &H,
) -> Result<String, SolveError> {
    let moves = random_scramble(rng, h)?;

    let out = moves
        .into_iter()
        .map(|m| format!("{m}"))
        .reduce(|a, b| format!("{a} {b}"))
        .unwrap_or_else(|| "".to_string());

    Ok(out)
}

use crate::idasearch;
use crate::idasearch::{Heuristic, Solvable, SolveError};
use crate::moves::CanReverse;
use rand::Rng;
use rayon::prelude::*;
use std::fmt::Display;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

pub trait RandomInit: Sized {
    fn random_state<R: Rng>(r: &mut R) -> Self;
}

pub fn bulk_scramble<
    R: Rng,
    M: CanReverse,
    State: RandomInit + Solvable<Move = M> + Sized + Sync + Send + 'static,
    H: Heuristic<State> + Sized + Sync + Send + 'static,
>(
    rng: &mut R,
    h: &H,
    num_scrambles: usize,
) -> Result<Vec<usize>, SolveError> {
    let states: Vec<State> = (0..num_scrambles).map(|_| State::random_state(rng)).collect();

    let completed = AtomicUsize::new(0);
    let start = Instant::now();

    states
        .into_par_iter()
        .map(|s| {
            let solution: Vec<M> = idasearch::solve(&s, h)?;
            let out = solution.len();

            let c = completed.fetch_add(1, Ordering::SeqCst);
            let c = c + 1; // fetch_add gets the OLD value
            if c % 1000 == 0 {
                let elapsed = start.elapsed();
                let elapsed_ms = elapsed.as_secs_f32() * 1000.0;
                let rate = elapsed_ms / (c as f32);
                let pct_complete = (c as f32) / (num_scrambles as f32) * 100.0;
                let rem_time = ((num_scrambles - c) as f32) * rate / 1000.0;
                println!("    Solved {c} states in {elapsed:?} -- {rate:.3} ms per iter ({pct_complete:.3}% complete; est {rem_time:.3} seconds remaining)");
            }
            Ok(out)
        })
        .collect()
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

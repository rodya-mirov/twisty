//! Helper functionality for IDA* search.

use crate::idasearch::SolveError::OutOfGas;

/// Estimator of the remaining cost. This must never OVER estimate (that is, if it says 10,
/// there really needs to not be a solution of size 9, or the algorithm will generate wrong answers)
/// but it is fine to UNDER estimate. In particular the function |_| 0 is a valid heuristic,
/// although the performance benefits are minimal.
pub mod heuristic_helpers;

pub trait Heuristic<T> {
    fn estimated_remaining_cost(&self, t: &T) -> usize;
}

// A closure or function of the appropriate signature can be used as a heuristic
impl<T, F: Fn(&T) -> usize> Heuristic<T> for F {
    fn estimated_remaining_cost(&self, t: &T) -> usize {
        self(t)
    }
}

/// Default implementation of a cost heuristic which is just ... no heuristic
/// This is useful for very small puzzles (where making a heuristic is an unnecessary hassle)
/// or for prototyping, or a baseline for comparison (to see what A* is giving you)
pub fn no_heuristic<T>(_t: &T) -> usize {
    0
}

/// Basic functionality required to feed into the IDA* search function.
pub trait Solvable: Sized + Clone {
    type Move: Copy + Clone + Eq + PartialEq;

    /// Determine if the current configuration is solved
    fn is_solved(&self) -> bool;

    /// List the available moves from here. All yielded moves must be applicable to the current
    /// position.
    // Note about IDE warning -- RPITIT is supported in Rust 1.75 but hasn't gotten into the IDE
    // plugin yet
    fn available_moves(&self) -> impl IntoIterator<Item = Self::Move>;

    /// Performance optimization; the implementor may describe which moves are redundant with
    /// each other; that is, if a particular move could follow another move in an optimal solution.
    /// If the moves are redundant but not filtered out by this function, the algorithm
    /// will still give correct results, but the branching factor may be higher than desired,
    /// causing slowness. On the other hand, if this rejects moves which are not actually redundant,
    /// the algorithm may give wrong results or fail to terminate.
    ///
    /// Thus the default rejects nothing, since this is never wrong.
    #[inline(always)]
    // parameters are present for trait implementors, not for the default implementation
    #[allow(unused_variables)]
    fn is_redundant(last_move: Self::Move, next_move: Self::Move) -> bool {
        false
    }

    /// Get the configuration brought about by applying the given move to the current position.
    /// This move is guaranteed to be given by the "available_moves" function for this configuration
    /// and it is fine to panic on invalid input.
    fn apply(&self, m: Self::Move) -> Self;

    /// A safe maximum for the search depth. IDA* will not search deeper than this. This is used
    /// as a stopgap, to prevent infinite searching, which should only occur in case of bugs.
    fn max_fuel() -> usize;
}

#[derive(Debug)]
pub enum SolveError {
    OutOfGas { max_fuel: usize },
}

pub fn solve<S: Solvable, H: Heuristic<S>>(state: &S, heuristic: &H) -> Result<Vec<<S as Solvable>::Move>, SolveError> {
    let max_fuel = S::max_fuel();

    #[derive(Eq, PartialEq, Copy, Clone, Debug)]
    enum SearchResult {
        Found,
        NotFound,
    }

    fn dfs<M: Copy, S: Solvable<Move = M>, H: Heuristic<S>>(
        state: &S,
        heuristic: &H,
        moves_so_far: &mut Vec<M>,
        rem_fuel: usize,
    ) -> SearchResult {
        if state.is_solved() {
            return SearchResult::Found;
        }

        let last_move = moves_so_far.last().copied();

        for m in state.available_moves() {
            // Note -- we don't need this in the config-depth algorithm because that
            // one has a HashSet that automatically deduplicates states.
            if last_move.is_some() && S::is_redundant(last_move.unwrap(), m) {
                continue;
            }

            let next = state.apply(m);

            let min_cost = heuristic.estimated_remaining_cost(&next) + 1;

            if min_cost > rem_fuel {
                continue;
            }

            moves_so_far.push(m);

            let sr_child = dfs(&next, heuristic, moves_so_far, rem_fuel - 1);
            if sr_child == SearchResult::Found {
                return sr_child;
            }

            moves_so_far.pop();
        }

        SearchResult::NotFound
    }

    for fuel in 0..=max_fuel {
        let mut solution = Vec::new();

        let sr = dfs(state, heuristic, &mut solution, fuel);

        if sr == SearchResult::Found {
            return Ok(solution);
        }
    }

    Err(OutOfGas { max_fuel })
}

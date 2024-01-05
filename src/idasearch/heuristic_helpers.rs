use std::hash::Hash;

use ahash::{HashMap, HashSet};

use crate::cubesearch::State;
use crate::idasearch::Heuristic;

pub struct BoundedStateCache<S: Hash + Eq + PartialEq> {
    stored: HashMap<S, usize>,
    // if not found, return this value
    fallback_depth: usize,
}

impl<S: Hash + Eq> Heuristic<S> for BoundedStateCache<S> {
    fn estimated_remaining_cost(&self, t: &S) -> usize {
        self.stored.get(t).copied().unwrap_or(self.fallback_depth)
    }
}

pub fn bounded_cache<S: Hash + Eq + Clone + State>(max_depth: usize) -> BoundedStateCache<S> {
    let mut out: HashMap<S, usize> = HashMap::default();

    // essentially just do a BFS until we hit the max depth
    let mut to_process: Vec<S> = vec![];
    let mut next_state: Vec<S> = vec![];
    let mut seen: HashSet<S> = HashSet::default();

    to_process.push(S::start());

    for depth in 0..=max_depth {
        for s in to_process.drain(..) {
            if !seen.insert(s.clone()) {
                continue;
            }

            out.insert(s.clone(), depth);

            let mut recv = |neighbor| {
                next_state.push(neighbor);
            };

            s.neighbors(&mut recv);
        }

        to_process.clear();
        std::mem::swap(&mut to_process, &mut next_state);
    }

    BoundedStateCache {
        stored: out,
        // we got everything of depth up to max_depth; so anything
        // else has more than that
        fallback_depth: max_depth + 1,
    }
}

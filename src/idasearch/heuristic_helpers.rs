use std::hash::Hash;

use ahash::{HashMap, HashSet};

use crate::cubesearch::State;
use crate::idasearch::Heuristic;

pub struct BoundedStateCache<H: Hash + Eq> {
    stored: HashMap<H, usize>,
    // if not found, return this value
    fallback_depth: usize,
}

impl<H: Hash + Eq> BoundedStateCache<H> {
    #[inline(always)]
    pub fn fallback_depth(&self) -> usize {
        self.fallback_depth
    }

    #[inline]
    pub fn remaining_cost_if_known<S: State<UniqueKey = H>>(&self, t: &S) -> Option<usize> {
        self.stored.get(&t.uniq_key()).copied()
    }
}

impl<H: Hash + Eq, S: State<UniqueKey = H>> Heuristic<S> for BoundedStateCache<H> {
    fn estimated_remaining_cost(&self, t: &S) -> usize {
        self.stored.get(&t.uniq_key()).copied().unwrap_or(self.fallback_depth)
    }
}

pub fn bounded_cache<S: Clone + State>(max_depth: usize) -> BoundedStateCache<<S as State>::UniqueKey> {
    let mut out: HashMap<<S as State>::UniqueKey, usize> = HashMap::default();

    // essentially just do a BFS until we hit the max depth
    let mut to_process: Vec<S> = vec![];
    let mut next_state: Vec<S> = vec![];
    let mut seen: HashSet<<S as State>::UniqueKey> = HashSet::default();

    to_process.push(S::start());

    for depth in 0..=max_depth {
        for s in to_process.drain(..) {
            if !seen.insert(s.uniq_key()) {
                continue;
            }

            out.insert(s.uniq_key(), depth);

            let mut recv = |neighbor| {
                next_state.push(neighbor);
            };

            s.neighbors(&mut recv);
        }

        assert!(to_process.is_empty());
        to_process.clear();
        std::mem::swap(&mut to_process, &mut next_state);

        if to_process.is_empty() {
            println!("Exited heuristic creation early; all solutions found in {depth} steps");
            break;
        }
    }

    BoundedStateCache {
        stored: out,
        // we got everything of depth up to max_depth; so anything
        // else has more than that
        fallback_depth: max_depth + 1,
    }
}

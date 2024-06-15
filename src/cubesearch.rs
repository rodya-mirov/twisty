use std::hash::Hash;
use std::time::{Duration, Instant};

use ahash::{HashMap, HashSet};
use itertools::Itertools;

use crate::idasearch::Solvable;

pub fn nice_print(puzzle_name: &str, counts: &HashMap<u128, u128>) {
    println!("Configuration depth summary for {puzzle_name}:");
    let total: u128 = counts.values().sum();

    let sorted_keys = counts.keys().copied().sorted();

    println!("\tThere are {total} total configurations.");

    for k in sorted_keys {
        let val = counts.get(&k).copied().unwrap();
        let pct = (val as f64) / (total as f64) * 100.;
        println!("\t{k} moves: {val} configurations ({pct:0.3} %)");
    }
}

pub trait State: Sized {
    type UniqueKey: 'static + Hash + Eq + PartialEq;

    fn neighbors<Recv>(&self, to_add: &mut Recv)
    where
        Recv: FnMut(Self);

    fn start() -> Self;

    /// Determine if the given configuration should count as "a" configuration
    /// This is used for deduplication; even if this returns false, it will still be processed
    /// in the BFS algorithm, but will not affect the counts per stage, typically because the
    /// "same" state has been or will be generated in another way, and we don't want to double
    /// count.
    ///
    /// Typically this is not needed; the default implication always returns true and is inlined,
    /// so should not cause a branch.
    #[inline(always)]
    fn should_count_as_config(&self) -> bool {
        true
    }

    /// A unique key identifying a puzzle state. In many cases this can just be the puzzle state
    /// itself, but it can be more performant to bitpack it manually here, so that the bitpacked
    /// version can be stored and compared against.
    ///
    /// This is primarily a performance optimization.
    fn uniq_key(&self) -> Self::UniqueKey;
}

/// Simple trait to implement if you have a solvable implementation already, and want a State
/// implementation for free
pub trait SimpleStartState: Sized {
    /// Key to store in a HashSet. If performance is not an issue, you can usually just use Self.
    type UniqueKey: 'static + Hash + Eq + PartialEq;

    fn start() -> Self;

    /// Determine if the given configuration should count as "a" configuration
    /// This is used for deduplication; even if this returns false, it will still be processed
    /// in the BFS algorithm, but will not affect the counts per stage, typically because the
    /// "same" state has been or will be generated in another way, and we don't want to double
    /// count.
    ///
    /// Typically this is not needed; the default implication always returns true and is inlined,
    /// so should not cause a branch.
    #[inline(always)]
    fn should_count_as_config(&self) -> bool {
        true
    }

    /// A unique key identifying a puzzle state. In many cases this can just be the puzzle state
    /// itself, but it can be more performant to bitpack it manually here, so that the bitpacked
    /// version can be stored and compared against.
    ///
    /// This is primarily a performance optimization.
    fn uniq_key(&self) -> Self::UniqueKey;
}

impl<K, T> State for T
where
    T: SimpleStartState<UniqueKey = K> + Solvable + Sized,
    K: Hash + Eq + PartialEq + Clone + 'static,
{
    type UniqueKey = K;

    fn neighbors<Recv>(&self, to_add: &mut Recv)
    where
        Recv: FnMut(Self),
    {
        let moves = self.available_moves();

        for m in moves {
            to_add(self.apply(m));
        }
    }

    fn start() -> Self {
        <Self as SimpleStartState>::start()
    }

    fn uniq_key(&self) -> Self::UniqueKey {
        SimpleStartState::uniq_key(self)
    }

    fn should_count_as_config(&self) -> bool {
        SimpleStartState::should_count_as_config(self)
    }
}

pub fn enumerate_state_space_started<T>(starts: Vec<T>) -> (Duration, HashMap<u128, u128>)
where
    T: State + Hash + Eq,
{
    let start_time = Instant::now();

    let mut counts: HashMap<_, _> = Default::default();

    let mut all_seen: HashSet<_> = Default::default();

    let mut next_distance = 0;
    let mut to_process: Vec<T> = starts;
    let mut next_stage: Vec<T> = Vec::default();

    loop {
        let mut this_stage_new_configs = 0;
        let mut recv = |neighbor| {
            next_stage.push(neighbor);
        };

        for state in to_process.iter() {
            if !all_seen.insert(state.uniq_key()) {
                continue;
            }

            if state.should_count_as_config() {
                this_stage_new_configs += 1;
            }

            state.neighbors(&mut recv);
        }

        if this_stage_new_configs == 0 {
            break;
        }

        counts.insert(next_distance, this_stage_new_configs);
        next_distance += 1;

        // TODO: find a nice way to enable/disable this with the CLI, without adding a ton of typing
        println!(
            "Many distance! Up to {next_distance} without stopping; up to {} unique states so far. Elapsed: {:?}",
            counts.values().sum::<u128>(),
            start_time.elapsed()
        );

        to_process.clear();
        std::mem::swap(&mut to_process, &mut next_stage);
    }

    let elapsed = start_time.elapsed();

    (elapsed, counts)
}

pub fn enumerate_state_space<T>() -> (Duration, HashMap<u128, u128>)
where
    T: State + Hash + Eq,
{
    enumerate_state_space_started(vec![T::start()])
}

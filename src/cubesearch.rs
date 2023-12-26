use std::hash::Hash;
use ahash::{HashMap, HashSet};
use itertools::Itertools;

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
    fn neighbors<Recv>(&self, to_add: &mut Recv) where Recv: FnMut(Self);

    fn start() -> Self;
}

pub fn enumerate_state_space<T>() -> HashMap<u128, u128> where T: State + Hash + Eq + Clone {
    let mut counts: HashMap<_, _> = Default::default();

    let mut all_seen: HashSet<T> = Default::default();

    let mut next_distance = 0;
    let mut to_process: HashSet<T> = HashSet::default();

    to_process.insert(T::start());

    loop {
        let mut next_stage = HashSet::default();

        let mut this_stage_new_configs = 0;
        let mut recv = |neighbor| {next_stage.insert(neighbor);};

        for state in to_process {
            if !all_seen.insert(state.clone()) {
                continue;
            }

            this_stage_new_configs += 1;
            state.neighbors(&mut recv);
        }

        if this_stage_new_configs == 0 {
            break;
        }

        counts.insert(next_distance, this_stage_new_configs);
        next_distance += 1;

        to_process = next_stage;
    }

    counts
}
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::assertions_on_constants)]

use std::time::Instant;

use crate::bandaged_3x3x3_1x2x3::Bandaged3x3x3with1x2x3;
use clap::{Parser, Subcommand};
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::coin_pyraminx::CoinPyraminx;
use crate::cubesearch::nice_print;
use crate::cubesearch::{enumerate_state_space, enumerate_state_space_started};
use crate::cuboid_2x2x3::Cuboid2x2x3;
use crate::cuboid_2x3x3::Cuboid2x3x3;
use crate::dino_cube::DinoCube;
use crate::floppy_1x2x2::Floppy1x2x2;
use crate::floppy_1x2x3::Floppy1x2x3;
use crate::floppy_1x3x3::Floppy1x3x3;
use crate::floppy_1xnxn::Floppy1xMxN;
use crate::idasearch::{no_heuristic, SolveError};
use crate::mirror_pocket_cube::MirrorPocketCube;
use crate::pocket_cube::PocketCube;
use crate::pyraminx::Pyraminx;

// reusable state modules
mod moves;
mod orientations;
mod random_helpers;
mod scrambles;

// reusable algorithm logic
mod cubesearch;
mod idasearch;

// actual puzzles
mod bandaged_3x3x3_1x2x3;
mod coin_pyraminx;
mod cuboid_2x2x3;
mod cuboid_2x3x3;
mod dino_cube;
mod floppy_1x2x2;
mod floppy_1x2x3;
mod floppy_1x3x3;
mod floppy_1xnxn;
mod mirror_pocket_cube;
mod pocket_cube;
mod pyraminx;
mod skewb;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(subcommand)]
    ConfigDepth(ConfigAlg),
    #[command(subcommand)]
    RandomScramble(ScrambleAlg),
}

#[derive(Subcommand, Copy, Clone, PartialEq, Eq)]
enum ConfigAlg {
    Floppy1x2x2,
    Floppy1x2x3,
    Floppy1x3x3,
    BigFloppy1x3x3,
    BigFloppy1x3x4,
    BigFloppy1x3x5,
    BigFloppy1x3x6,
    BigFloppy1x4x4,
    BigFloppy1x4x5,
    BigFloppy1x4x6,
    BigFloppy1x5x5,
    BigFloppy1x5x6,
    BigFloppy1x6x6,
    Cuboid2x2x3,
    Cuboid2x3x3,
    DinoCubeOneSolution,
    DinoCubeEitherSolution,
    Skewb,
    MirrorPocketCube,
    PocketCube,
    PyraminxNoTips,
    PyraminxWithTips,
    CoinPyraminx,
}

impl ConfigAlg {
    fn nice_name(&self) -> &'static str {
        match self {
            ConfigAlg::Floppy1x2x2 => "Floppy 1x2x2",
            ConfigAlg::Floppy1x2x3 => "Floppy 1x2x3",
            ConfigAlg::Floppy1x3x3 => "Floppy 1x3x3",
            ConfigAlg::BigFloppy1x3x3 => "Big Floppy 1x3x3",
            ConfigAlg::BigFloppy1x3x4 => "Big Floppy 1x3x4",
            ConfigAlg::BigFloppy1x3x5 => "Big Floppy 1x3x5",
            ConfigAlg::BigFloppy1x3x6 => "Big Floppy 1x3x6",
            ConfigAlg::BigFloppy1x4x4 => "Big Floppy 1x4x4",
            ConfigAlg::BigFloppy1x4x5 => "Big Floppy 1x4x5",
            ConfigAlg::BigFloppy1x4x6 => "Big Floppy 1x4x6",
            ConfigAlg::BigFloppy1x5x5 => "Big Floppy 1x5x5",
            ConfigAlg::BigFloppy1x5x6 => "Big Floppy 1x5x6",
            ConfigAlg::BigFloppy1x6x6 => "Big Floppy 1x6x6",
            ConfigAlg::Cuboid2x2x3 => "Cuboid 2x2x3",
            ConfigAlg::Cuboid2x3x3 => "Cuboid 2x3x3",
            ConfigAlg::DinoCubeOneSolution => "Dino Cube (To One Solution)",
            ConfigAlg::DinoCubeEitherSolution => "Dino Cube (To Either Solution)",
            ConfigAlg::Skewb => "Skewb",
            ConfigAlg::MirrorPocketCube => "Mirror Pocket Cube",
            ConfigAlg::PocketCube => "Pocket Cube",
            ConfigAlg::PyraminxNoTips => "Pyraminx (No Tips)",
            ConfigAlg::PyraminxWithTips => "Pyraminx (With Tips)",
            ConfigAlg::CoinPyraminx => "Coin Pyraminx",
        }
    }
}

#[derive(Subcommand, Copy, Clone, PartialEq, Eq)]
enum ScrambleAlg {
    Floppy1x2x2,
    Floppy1x2x3,
    Floppy1x3x3,
    Cuboid2x2x3,
    Cuboid2x3x3,
    DinoCube,
    Bandaged3x3x3With1x2x3,
}

impl ScrambleAlg {
    fn nice_name(&self) -> &'static str {
        match self {
            ScrambleAlg::Floppy1x2x2 => "Floppy 1x2x2",
            ScrambleAlg::Floppy1x2x3 => "Floppy 1x2x3",
            ScrambleAlg::Floppy1x3x3 => "Floppy 1x3x3",
            ScrambleAlg::Cuboid2x2x3 => "Cuboid 2x2x3",
            ScrambleAlg::Cuboid2x3x3 => "Cuboid 2x3x3",
            ScrambleAlg::DinoCube => "Dino Cube",
            ScrambleAlg::Bandaged3x3x3With1x2x3 => "Bandaged 3x3x3 with 1x2x3",
        }
    }
}

fn configuration_depth(alg: ConfigAlg) {
    println!("Computing configuration depth summary for {}", alg.nice_name());

    let (elapsed, gn_count) = match alg {
        ConfigAlg::Floppy1x2x2 => enumerate_state_space::<Floppy1x2x2>(),
        ConfigAlg::Floppy1x2x3 => enumerate_state_space::<Floppy1x2x3>(),
        ConfigAlg::Floppy1x3x3 => enumerate_state_space::<Floppy1x3x3>(),
        ConfigAlg::BigFloppy1x3x3 => enumerate_state_space::<Floppy1xMxN<1, 1>>(),
        ConfigAlg::BigFloppy1x3x4 => enumerate_state_space::<Floppy1xMxN<1, 2>>(),
        ConfigAlg::BigFloppy1x3x5 => enumerate_state_space::<Floppy1xMxN<1, 3>>(),
        ConfigAlg::BigFloppy1x3x6 => enumerate_state_space::<Floppy1xMxN<1, 4>>(),
        ConfigAlg::BigFloppy1x4x4 => enumerate_state_space::<Floppy1xMxN<2, 2>>(),
        ConfigAlg::BigFloppy1x4x5 => enumerate_state_space::<Floppy1xMxN<2, 3>>(),
        ConfigAlg::BigFloppy1x4x6 => enumerate_state_space::<Floppy1xMxN<2, 4>>(),
        ConfigAlg::BigFloppy1x5x5 => enumerate_state_space::<Floppy1xMxN<3, 3>>(),
        ConfigAlg::BigFloppy1x5x6 => enumerate_state_space::<Floppy1xMxN<3, 4>>(),
        ConfigAlg::BigFloppy1x6x6 => enumerate_state_space::<Floppy1xMxN<4, 4>>(),
        ConfigAlg::Cuboid2x2x3 => enumerate_state_space::<Cuboid2x2x3>(),
        ConfigAlg::Cuboid2x3x3 => enumerate_state_space::<Cuboid2x3x3>(),
        ConfigAlg::DinoCubeOneSolution => enumerate_state_space::<DinoCube>(),
        ConfigAlg::DinoCubeEitherSolution => {
            enumerate_state_space_started::<DinoCube>(vec![DinoCube::solved_state(), DinoCube::solved_mirrored()])
        }
        ConfigAlg::Skewb => enumerate_state_space::<skewb::Skewb>(),
        ConfigAlg::MirrorPocketCube => enumerate_state_space::<MirrorPocketCube>(),
        ConfigAlg::PocketCube => enumerate_state_space::<PocketCube>(),
        ConfigAlg::PyraminxNoTips => enumerate_state_space::<Pyraminx>(),
        ConfigAlg::PyraminxWithTips => {
            let start = Instant::now();
            let (_, gn_count) = enumerate_state_space::<Pyraminx>();
            let gn_count = pyraminx::gn_count_with_tips(gn_count);
            (start.elapsed(), gn_count)
        }
        ConfigAlg::CoinPyraminx => enumerate_state_space::<CoinPyraminx>(),
    };

    println!("Processing took {elapsed:?}");

    nice_print(alg.nice_name(), &gn_count);
}

fn random_scramble(alg: ScrambleAlg) {
    const NUM_SCRAMBLES: usize = 10;
    println!("Computing {NUM_SCRAMBLES} random scrambles for {}", alg.nice_name());

    // hard-coded seed for reproducibility
    // let mut rng = StdRng::from_seed([15; 32]);
    // random seed for actual scrambling
    let mut rng = StdRng::from_entropy();

    let setup_time = Instant::now();

    let mut scrambler: Box<dyn FnMut() -> Result<String, SolveError>> = match alg {
        ScrambleAlg::Floppy1x2x2 => {
            Box::new(|| scrambles::random_scramble_string::<_, _, Floppy1x2x2, _>(&mut rng, &no_heuristic))
        }
        ScrambleAlg::Floppy1x2x3 => {
            Box::new(|| scrambles::random_scramble_string::<_, _, Floppy1x2x3, _>(&mut rng, &no_heuristic))
        }
        ScrambleAlg::Floppy1x3x3 => {
            Box::new(|| scrambles::random_scramble_string::<_, _, Floppy1x3x3, _>(&mut rng, &no_heuristic))
        }
        ScrambleAlg::Cuboid2x2x3 => {
            let heuristic = cuboid_2x2x3::make_heuristic();
            Box::new(move || scrambles::random_scramble_string::<_, _, Cuboid2x2x3, _>(&mut rng, &heuristic))
        }
        ScrambleAlg::Cuboid2x3x3 => {
            let heuristic = cuboid_2x3x3::make_heuristic();
            Box::new(move || scrambles::random_scramble_string::<_, _, Cuboid2x3x3, _>(&mut rng, &heuristic))
        }
        ScrambleAlg::DinoCube => {
            let heuristic = dino_cube::make_heuristic();
            Box::new(move || scrambles::random_scramble_string::<_, _, DinoCube, _>(&mut rng, &heuristic))
        }
        ScrambleAlg::Bandaged3x3x3With1x2x3 => {
            let heuristic = bandaged_3x3x3_1x2x3::make_heuristic();
            Box::new(move || scrambles::random_scramble_string::<_, _, Bandaged3x3x3with1x2x3, _>(&mut rng, &heuristic))
        }
    };

    let elapsed = setup_time.elapsed();
    println!("Setting up heuristics took {elapsed:?}");

    for i in 0..NUM_SCRAMBLES {
        let start = Instant::now();
        let scramble_result = scrambler();
        let elapsed = start.elapsed();

        match scramble_result {
            Ok(scramble_str) => {
                let len = scramble_str.split_ascii_whitespace().count();
                println!("Random scramble {i}: {scramble_str}");
                println!("    (scramble of length {len} took {elapsed:?})");
            }
            Err(SolveError::OutOfGas { max_fuel }) => {
                println!("Could not find a solution to random state");
                println!("    (out of gas with max fuel of length {max_fuel} took {elapsed:?})");
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::ConfigDepth(alg) => configuration_depth(alg),
        Commands::RandomScramble(alg) => random_scramble(alg),
    }
}

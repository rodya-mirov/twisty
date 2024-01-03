#![allow(clippy::upper_case_acronyms)]

use std::time::Instant;

use clap::{Parser, Subcommand};
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::coin_pyraminx::CoinPyraminx;
use crate::cubesearch::enumerate_state_space;
use crate::cubesearch::nice_print;
use crate::cuboid_2x2x3::Cuboid2x2x3;
use crate::floppy_1x2x2::Floppy1x2x2;
use crate::floppy_1x2x3::Floppy1x2x3;
use crate::floppy_1x3x3::Floppy1x3x3;
use crate::idasearch::no_heuristic;
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
mod coin_pyraminx;
mod cuboid_2x2x3;
mod floppy_1x2x2;
mod floppy_1x2x3;
mod floppy_1x3x3;
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
    Cuboid2x2x3,
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
            ConfigAlg::Cuboid2x2x3 => "Cuboid 2x2x3",
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
}

impl ScrambleAlg {
    fn nice_name(&self) -> &'static str {
        match self {
            ScrambleAlg::Floppy1x2x2 => "Floppy 1x2x2",
            ScrambleAlg::Floppy1x2x3 => "Floppy 1x2x3",
            ScrambleAlg::Floppy1x3x3 => "Floppy 1x3x3",
        }
    }
}

fn configuration_depth(alg: ConfigAlg) {
    println!(
        "Computing configuration depth summary for {}",
        alg.nice_name()
    );

    let (elapsed, gn_count) = match alg {
        ConfigAlg::Floppy1x2x2 => enumerate_state_space::<Floppy1x2x2>(),
        ConfigAlg::Floppy1x2x3 => enumerate_state_space::<Floppy1x2x3>(),
        ConfigAlg::Floppy1x3x3 => enumerate_state_space::<Floppy1x3x3>(),
        ConfigAlg::Cuboid2x2x3 => enumerate_state_space::<Cuboid2x2x3>(),
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
    const NUM_SCRAMBLES: usize = 5;
    println!(
        "Computing {NUM_SCRAMBLES} random scrambles for {}",
        alg.nice_name()
    );

    let mut rng = StdRng::from_entropy();

    let mut scrambler: Box<dyn FnMut() -> String> = match alg {
        ScrambleAlg::Floppy1x2x2 => Box::new(|| {
            scrambles::random_scramble_string::<_, _, Floppy1x2x2, _>(&mut rng, &no_heuristic)
        }),
        ScrambleAlg::Floppy1x2x3 => Box::new(|| {
            scrambles::random_scramble_string::<_, _, Floppy1x2x3, _>(&mut rng, &no_heuristic)
        }),
        ScrambleAlg::Floppy1x3x3 => Box::new(|| {
            scrambles::random_scramble_string::<_, _, Floppy1x3x3, _>(&mut rng, &no_heuristic)
        }),
    };

    for i in 0..NUM_SCRAMBLES {
        let scramble_str = scrambler();
        println!("Random scramble {i}: {scramble_str}");
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::ConfigDepth(alg) => configuration_depth(alg),
        Commands::RandomScramble(alg) => random_scramble(alg),
    }
}

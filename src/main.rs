#![allow(clippy::upper_case_acronyms)]

use std::time::Instant;

use clap::Parser;

use crate::coin_pyraminx::CoinPyraminx;
use crate::cubesearch::enumerate_state_space;
use crate::cubesearch::nice_print;
use crate::cuboid_2x2x3::Cuboid2x2x3;
use crate::floppy_1x2x2::Floppy1x2x2;
use crate::floppy_1x2x3::Floppy1x2x3;
use crate::floppy_1x3x3::Floppy1x3x3;
use crate::mirror_pocket_cube::MirrorPocketCube;
use crate::pocket_cube::PocketCube;
use crate::pyraminx::Pyraminx;

// helper modules
mod cubesearch;
mod orientations;

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
enum Alg {
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

impl Alg {
    fn nice_name(&self) -> &'static str {
        match self {
            Alg::Floppy1x2x2 => "Floppy 1x2x2",
            Alg::Floppy1x2x3 => "Floppy 1x2x3",
            Alg::Floppy1x3x3 => "Floppy 1x3x3",
            Alg::Cuboid2x2x3 => "Cuboid 2x2x3",
            Alg::Skewb => "Skewb",
            Alg::MirrorPocketCube => "Mirror Pocket Cube",
            Alg::PocketCube => "Pocket Cube",
            Alg::PyraminxNoTips => "Pyraminx (No Tips)",
            Alg::PyraminxWithTips => "Pyraminx (With Tips)",
            Alg::CoinPyraminx => "Coin Pyraminx",
        }
    }
}

fn main() {
    let cli = Alg::parse();

    println!(
        "Computing configuration depth summary for {}",
        cli.nice_name()
    );

    let (elapsed, gn_count) = match cli {
        Alg::Floppy1x2x2 => enumerate_state_space::<Floppy1x2x2>(),
        Alg::Floppy1x2x3 => enumerate_state_space::<Floppy1x2x3>(),
        Alg::Floppy1x3x3 => enumerate_state_space::<Floppy1x3x3>(),
        Alg::Cuboid2x2x3 => enumerate_state_space::<Cuboid2x2x3>(),
        Alg::Skewb => enumerate_state_space::<skewb::Skewb>(),
        Alg::MirrorPocketCube => enumerate_state_space::<MirrorPocketCube>(),
        Alg::PocketCube => enumerate_state_space::<PocketCube>(),
        Alg::PyraminxNoTips => enumerate_state_space::<Pyraminx>(),
        Alg::PyraminxWithTips => {
            let start = Instant::now();
            let (_, gn_count) = enumerate_state_space::<Pyraminx>();
            let gn_count = pyraminx::gn_count_with_tips(gn_count);
            (start.elapsed(), gn_count)
        }
        Alg::CoinPyraminx => enumerate_state_space::<CoinPyraminx>(),
    };

    println!("Processing took {elapsed:?}");

    nice_print(cli.nice_name(), &gn_count);
}

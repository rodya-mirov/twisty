#![allow(clippy::upper_case_acronyms)]

use clap::Parser;

use cubesearch::enumerate_state_space;

use crate::cubesearch::nice_print;
use crate::mirror_pocket_cube::MirrorPocketCube;

// helper modules
mod cubesearch;

// actual puzzles
mod cuboid_2x3x3;
mod floppy_1x2x2;
mod floppy_1x2x3;
mod floppy_1x3x3;
mod mirror_pocket_cube;
mod skewb;

#[derive(Parser)]
enum Alg {
    Floppy1x2x2,
    Floppy1x2x3,
    Floppy1x3x3,
    Cuboid2x2x3,
    Skewb,
    MirrorPocketCube,
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
        }
    }
}

fn main() {
    let cli = Alg::parse();

    println!("Computing configuration depth summary for {}", cli.nice_name());

    let (elapsed, gn_count) = match cli {
        Alg::Floppy1x2x2 => enumerate_state_space::<floppy_1x2x2::Floppy1x2x2>(),
        Alg::Floppy1x2x3 => enumerate_state_space::<floppy_1x2x3::Floppy1x2x3>(),
        Alg::Floppy1x3x3 => enumerate_state_space::<floppy_1x3x3::Floppy1x3x3>(),
        Alg::Cuboid2x2x3 => enumerate_state_space::<cuboid_2x3x3::Cuboid2x3x3>(),
        Alg::Skewb => enumerate_state_space::<skewb::Skewb>(),
        Alg::MirrorPocketCube => enumerate_state_space::<MirrorPocketCube>(),
    };

    println!("Processing took {elapsed:?}");


    nice_print(cli.nice_name(), &gn_count);
}

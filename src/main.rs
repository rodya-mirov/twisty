#![allow(clippy::upper_case_acronyms)]

use crate::cubesearch::nice_print;
use cubesearch::enumerate_state_space;

// helper modules
mod cubesearch;

// actual puzzles
mod cuboid_2x3x3;
mod floppy_1x2x2;
mod floppy_1x2x3;
mod floppy_1x3x3;
mod mirror_pocket_cube;
mod skewb;

fn main() {
    // TODO: nicer CLI
    let (elapsed, gn_count) = enumerate_state_space::<floppy_1x2x3::Floppy1x2x3>();

    nice_print("Floppy 1x2x3", &elapsed, &gn_count);

    // TODO: nicer CLI
    let (elapsed, gn_count) = enumerate_state_space::<floppy_1x2x2::Floppy1x2x2>();

    nice_print("Floppy 1x2x2", &elapsed, &gn_count);

    // TODO: nicer CLI
    let (elapsed, gn_count) = enumerate_state_space::<floppy_1x3x3::Floppy1x3x3>();

    nice_print("Floppy 1x3x3", &elapsed, &gn_count);

    // TODO: nicer CLI
    let (elapsed, gn_count) = enumerate_state_space::<cuboid_2x3x3::Cuboid2x3x3>();

    nice_print("Cuboid 2x3x3", &elapsed, &gn_count);

    // TODO: nicer CLI
    let (elapsed, gn_count) = enumerate_state_space::<skewb::Skewb>();

    nice_print("Skewb", &elapsed, &gn_count);

    // TODO: nicer CLI
    let (elapsed, gn_count) = enumerate_state_space::<mirror_pocket_cube::MirrorPocketCube>();

    nice_print("Mirror Pocket Cube", &elapsed, &gn_count);
}

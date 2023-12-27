use crate::cubesearch::nice_print;
use cubesearch::enumerate_state_space;

// helper modules
mod cubesearch;

// actual puzzles
mod cuboid_2x3x3;
mod floppy_1x2x2;
mod floppy_1x2x3;
mod floppy_1x3x3;

fn main() {
    // TODO: nicer CLI
    let gn_count = enumerate_state_space::<floppy_1x2x3::Floppy1x2x3>();

    nice_print("Floppy 1x2x3", &gn_count);

    // TODO: nicer CLI
    let gn_count = enumerate_state_space::<floppy_1x2x2::Floppy1x2x2>();

    nice_print("Floppy 1x2x2", &gn_count);

    // TODO: nicer CLI
    let gn_count = enumerate_state_space::<floppy_1x3x3::Floppy1x3x3>();

    nice_print("Floppy 1x3x3", &gn_count);

    // TODO: nicer CLI
    let gn_count = enumerate_state_space::<cuboid_2x3x3::Cuboid2x3x3>();

    nice_print("Cuboid 2x3x3", &gn_count);
}

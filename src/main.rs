use cubesearch::enumerate_state_space;
use crate::cubesearch::nice_print;

// helper modules
mod cubesearch;

// actual puzzles
mod floppy_1x2x2;
mod floppy_1x2x3;

fn main() {
    // TODO: nicer CLI
    let gn_count = enumerate_state_space::<floppy_1x2x3::Floppy1x2x3>();

    nice_print("Floppy 1x2x3", &gn_count);

    // TODO: nicer CLI
    let gn_count = enumerate_state_space::<floppy_1x2x2::Floppy1x2x2>();

    nice_print("Floppy 1x2x2", &gn_count);
}

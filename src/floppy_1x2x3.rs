use crate::cubesearch::State;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
enum CornerCubelet {
    UL, UR, DL, DR
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Floppy1x2x3 {
    ul: CornerCubelet,
    ur: CornerCubelet,
    dl: CornerCubelet,
    dr: CornerCubelet,
    // true is "white is forward", false is "yellow is forward"
    // making a whole enum for this seemed dumb
    rc_solved: bool,
}

impl State for Floppy1x2x3 {
    fn neighbors<Recv>(&self, to_add: &mut Recv) where Recv: FnMut(Self) {
        // three neighbors: U2, R2, D2
        let Self { ul, ur, dl, dr, rc_solved } = *self;

        // U2
        to_add(Self {
            ul: ur,
            ur: ul,
            dl,
            dr,
            rc_solved
        });

        // D2
        to_add(Self {
            ul,
            ur,
            dl: dr,
            dr: dl,
            rc_solved,
        });

        // R2
        to_add(Self {
            ul,
            ur: dr,
            dl,
            dr: ur,
            rc_solved: !rc_solved,
        });
    }

    fn start() -> Self {
        Self {
            ul: CornerCubelet::UL,
            ur: CornerCubelet::UR,
            dl: CornerCubelet::DL,
            dr: CornerCubelet::DR,
            rc_solved: true,
        }
    }
}
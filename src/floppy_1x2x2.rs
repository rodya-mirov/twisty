use crate::cubesearch::State;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
enum CornerCubelet {
    UL, UR, DR
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Floppy1x2x2 {
    ul: CornerCubelet,
    ur: CornerCubelet,
    dr: CornerCubelet,
}

impl State for Floppy1x2x2 {
    fn neighbors<Recv>(&self, to_add: &mut Recv) where Recv: FnMut(Self) {
        // three neighbors: U2, R2, D2
        let Self { ul, ur, dr } = *self;

        // U2
        to_add(Self {
            ul: ur,
            ur: ul,
            dr,
        });

        // R2
        to_add(Self {
            ul,
            ur: dr,
            dr: ur,
        });
    }

    fn start() -> Self {
        Self {
            ul: CornerCubelet::UL,
            ur: CornerCubelet::UR,
            dr: CornerCubelet::DR,
        }
    }
}
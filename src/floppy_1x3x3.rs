use crate::cubesearch::State;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
enum CornerCubelet {
    UL,
    UR,
    DL,
    DR,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Floppy1x3x3 {
    ul: CornerCubelet,
    ur: CornerCubelet,
    dl: CornerCubelet,
    dr: CornerCubelet,
    // true is "white is forward", false is "yellow is forward"
    // making a whole enum for this seemed dumb
    rc_solved: bool,
    uc_solved: bool,
    dc_solved: bool,
    lc_solved: bool,
}

impl State for Floppy1x3x3 {
    fn neighbors<Recv>(&self, to_add: &mut Recv)
    where
        Recv: FnMut(Self),
    {
        // three neighbors: U2, R2, D2
        let Self {
            ul,
            ur,
            dl,
            dr,
            rc_solved,
            lc_solved,
            dc_solved,
            uc_solved,
        } = *self;

        // U2
        to_add(Self {
            ul: ur,
            ur: ul,
            dl,
            dr,
            uc_solved: !uc_solved,
            lc_solved,
            rc_solved,
            dc_solved,
        });

        // D2
        to_add(Self {
            ul,
            ur,
            dl: dr,
            dr: dl,
            uc_solved,
            lc_solved,
            rc_solved,
            dc_solved: !dc_solved,
        });

        // L2
        to_add(Self {
            ul: dl,
            dl: ul,
            ur,
            dr,

            uc_solved,
            lc_solved: !lc_solved,
            rc_solved,
            dc_solved,
        });

        // R2
        to_add(Self {
            ul,
            ur: dr,
            dl,
            dr: ur,
            uc_solved,
            lc_solved,
            rc_solved: !rc_solved,
            dc_solved,
        });
    }

    fn start() -> Self {
        Self {
            ul: CornerCubelet::UL,
            ur: CornerCubelet::UR,
            dl: CornerCubelet::DL,
            dr: CornerCubelet::DR,
            rc_solved: true,
            lc_solved: true,
            uc_solved: true,
            dc_solved: true,
        }
    }
}

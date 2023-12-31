use crate::cubesearch::State;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
enum Cubelet {
    // we leave one cube in the DBL position, and it never comes up again
    DBR,
    DFL,
    DFR,
    UBL,
    UBR,
    UFL,
    UFR,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
enum Orientation {
    Normal,
    CW,
    CCW,
}

impl Orientation {
    #[inline(always)]
    fn cw(self) -> Self {
        match self {
            Orientation::Normal => Orientation::CW,
            Orientation::CW => Orientation::CCW,
            Orientation::CCW => Orientation::Normal,
        }
    }

    #[inline(always)]
    fn ccw(self) -> Self {
        match self {
            Orientation::Normal => Orientation::CCW,
            Orientation::CCW => Orientation::CW,
            Orientation::CW => Orientation::Normal,
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
struct PosState {
    // DBL is fixed and has no need here
    dbr: Cubelet,
    dfl: Cubelet,
    dfr: Cubelet,
    ubl: Cubelet,
    ubr: Cubelet,
    ufl: Cubelet,
    ufr: Cubelet,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
struct OrientationState {
    // DBL is fixed and has no need here
    dbr: Orientation,
    dfl: Orientation,
    dfr: Orientation,
    ubl: Orientation,
    ubr: Orientation,
    ufl: Orientation,
    ufr: Orientation,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub struct PocketCube {
    pos: PosState,
    orr: OrientationState,
}

trait CubeState: Sized {
    fn start() -> Self;

    fn r(&self) -> Self;

    fn u(&self) -> Self;

    fn f(&self) -> Self;
}

impl CubeState for PosState {
    fn start() -> Self {
        Self {
            // DBL is fixed
            dbr: Cubelet::DBR,
            dfl: Cubelet::DFL,
            dfr: Cubelet::DFR,
            ubl: Cubelet::UBL,
            ubr: Cubelet::UBR,
            ufl: Cubelet::UFL,
            ufr: Cubelet::UFR,
        }
    }

    #[inline(always)]
    fn r(&self) -> Self {
        Self {
            ufr: self.dfr,
            dfr: self.dbr,
            dbr: self.ubr,
            ubr: self.ufr,
            ..*self
        }
    }

    #[inline(always)]
    fn f(&self) -> Self {
        Self {
            ufl: self.dfl,
            dfl: self.dfr,
            dfr: self.ufr,
            ufr: self.ufl,
            ..*self
        }
    }

    #[inline(always)]
    fn u(&self) -> Self {
        Self {
            ufl: self.ufr,
            ufr: self.ubr,
            ubr: self.ubl,
            ubl: self.ufl,
            ..*self
        }
    }
}

impl CubeState for OrientationState {
    fn start() -> Self {
        Self {
            // DBL is fixed
            dbr: Orientation::Normal,
            dfl: Orientation::Normal,
            dfr: Orientation::Normal,
            ubl: Orientation::Normal,
            ubr: Orientation::Normal,
            ufl: Orientation::Normal,
            ufr: Orientation::Normal,
        }
    }

    #[inline(always)]
    fn r(&self) -> Self {
        Self {
            ufr: self.dfr.ccw(),
            dfr: self.dbr.cw(),
            dbr: self.ubr.ccw(),
            ubr: self.ufr.cw(),
            ..*self
        }
    }

    #[inline(always)]
    fn f(&self) -> Self {
        Self {
            ufl: self.dfl.ccw(),
            dfl: self.dfr.cw(),
            dfr: self.ufr.ccw(),
            ufr: self.ufl.cw(),
            ..*self
        }
    }

    #[inline(always)]
    fn u(&self) -> Self {
        Self {
            // no orientation change for U turns
            ufl: self.ufr,
            ufr: self.ubr,
            ubr: self.ubl,
            ubl: self.ufl,
            ..*self
        }
    }
}

impl CubeState for PocketCube {
    fn start() -> Self {
        Self {
            pos: PosState::start(),
            orr: OrientationState::start(),
        }
    }

    #[inline(always)]
    fn u(&self) -> Self {
        Self {
            orr: self.orr.u(),
            pos: self.pos.u(),
        }
    }

    #[inline(always)]
    fn f(&self) -> Self {
        Self {
            orr: self.orr.f(),
            pos: self.pos.f(),
        }
    }

    #[inline(always)]
    fn r(&self) -> Self {
        Self {
            orr: self.orr.r(),
            pos: self.pos.r(),
        }
    }
}

impl State for PocketCube {
    fn neighbors<Recv>(&self, to_add: &mut Recv)
        where
            Recv: FnMut(Self),
    {
        // three moves -- R/F/D -- with three orientations each (1/2/rev)

        // R
        to_add(self.r());
        to_add(self.r().r());
        to_add(self.r().r().r());

        // F
        to_add(self.f());
        to_add(self.f().f());
        to_add(self.f().f().f());

        // U
        to_add(self.u());
        to_add(self.u().u());
        to_add(self.u().u().u());
    }

    fn start() -> Self {
        <PocketCube as CubeState>::start()
    }
}

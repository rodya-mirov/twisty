use crate::cubesearch::State;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
enum Cubelet {
    // we leave the small cube in the BUL position, and it never comes up again
    Narrow,
    Wide,
    BigCube,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
enum Orientation {
    Normal,
    CW,
    CCW,
    Fixed, // for things that don't change orientation
}

impl Orientation {
    #[inline(always)]
    fn cw(self) -> Self {
        match self {
            Orientation::Normal => Orientation::CW,
            Orientation::CW => Orientation::CCW,
            Orientation::CCW => Orientation::Normal,
            Orientation::Fixed => Orientation::Fixed,
        }
    }

    #[inline(always)]
    fn ccw(self) -> Self {
        match self {
            Orientation::Normal => Orientation::CCW,
            Orientation::CCW => Orientation::CW,
            Orientation::CW => Orientation::Normal,
            Orientation::Fixed => Orientation::Fixed,
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct PosState {
    ful: Cubelet,
    fur: Cubelet,
    bur: Cubelet,
    fdl: Cubelet,
    fdr: Cubelet,
    bdl: Cubelet,
    bdr: Cubelet,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct OrientationState {
    ful: Orientation,
    fur: Orientation,
    bur: Orientation,
    fdl: Orientation,
    fdr: Orientation,
    bdl: Orientation,
    bdr: Orientation,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct MirrorPocketCube {
    pos: PosState,
    orr: OrientationState,
}

trait CubeState: Sized {
    fn start() -> Self;

    fn r(&self) -> Self;

    fn d(&self) -> Self;

    fn f(&self) -> Self;
}

impl CubeState for PosState {
    fn start() -> Self {
        Self {
            ful: Cubelet::Narrow,
            fur: Cubelet::Wide,
            bur: Cubelet::Narrow,
            fdl: Cubelet::Wide,
            fdr: Cubelet::BigCube,
            bdl: Cubelet::Narrow,
            bdr: Cubelet::Wide,
        }
    }

    #[inline(always)]
    fn r(&self) -> Self {
        Self {
            fur: self.fdr,
            fdr: self.bdr,
            bdr: self.bur,
            bur: self.fur,
            ..*self
        }
    }

    #[inline(always)]
    fn f(&self) -> Self {
        Self {
            fur: self.ful,
            ful: self.fdl,
            fdl: self.fdr,
            fdr: self.fur,
            ..*self
        }
    }

    #[inline(always)]
    fn d(&self) -> Self {
        Self {
            fdr: self.fdl,
            fdl: self.bdl,
            bdl: self.bdr,
            bdr: self.fdr,
            ..*self
        }
    }
}

impl CubeState for OrientationState {
    fn start() -> Self {
        Self {
            // BUL is the small cube, so FDR is the big cube, which has a Fixed orientation
            ful: Orientation::Normal,
            fur: Orientation::Normal,
            bur: Orientation::Normal,
            fdl: Orientation::Normal,
            fdr: Orientation::Fixed,
            bdl: Orientation::Normal,
            bdr: Orientation::Normal,
        }
    }

    #[inline(always)]
    fn r(&self) -> Self {
        Self {
            fur: self.fdr.ccw(),
            fdr: self.bdr.cw(),
            bdr: self.bur.ccw(),
            bur: self.fur.cw(),
            ..*self
        }
    }

    #[inline(always)]
    fn f(&self) -> Self {
        Self {
            fur: self.ful.cw(),
            ful: self.fdl.ccw(),
            fdl: self.fdr.cw(),
            fdr: self.fur.ccw(),
            ..*self
        }
    }

    #[inline(always)]
    fn d(&self) -> Self {
        // no changes to orientation since it's defined by "facing up or down"
        Self {
            fdr: self.fdl,
            fdl: self.bdl,
            bdl: self.bdr,
            bdr: self.fdr,
            ..*self
        }
    }
}

impl CubeState for MirrorPocketCube {
    fn start() -> Self {
        Self {
            pos: PosState::start(),
            orr: OrientationState::start(),
        }
    }

    #[inline(always)]
    fn d(&self) -> Self {
        Self {
            orr: self.orr.d(),
            pos: self.pos.d(),
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

impl State for MirrorPocketCube {
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

        // D
        to_add(self.d());
        to_add(self.d().d());
        to_add(self.d().d().d());
    }

    fn start() -> Self {
        <MirrorPocketCube as CubeState>::start()
    }
}

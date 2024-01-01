use crate::cubesearch::State;
use crate::orientations::CornerOrientation;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
enum CornerCubelet {
    // we leave the BUL corner cubelet fixed, so we don't need to consider it
    FUL,
    FUR,
    BUR,
    FDL,
    FDR,
    BDL,
    BDR,
}

impl CornerCubelet {
    #[inline(always)]
    fn as_u8_three_bits(&self) -> u8 {
        match self {
            CornerCubelet::FUL => 0,
            CornerCubelet::FUR => 1,
            CornerCubelet::BUR => 2,
            CornerCubelet::FDL => 3,
            CornerCubelet::FDR => 4,
            CornerCubelet::BDL => 5,
            CornerCubelet::BDR => 6,
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
enum CenterCubelet {
    U,
    D,
    F,
    L,
    R,
    B,
}

impl CenterCubelet {
    fn as_u8_three_bits(&self) -> u8 {
        match self {
            CenterCubelet::U => 0,
            CenterCubelet::D => 1,
            CenterCubelet::F => 2,
            CenterCubelet::L => 3,
            CenterCubelet::R => 4,
            CenterCubelet::B => 5,
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct CornerPosState {
    ful: CornerCubelet,
    fur: CornerCubelet,
    bur: CornerCubelet,
    fdl: CornerCubelet,
    fdr: CornerCubelet,
    bdl: CornerCubelet,
    bdr: CornerCubelet,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct CornerOrientationState {
    ful: CornerOrientation,
    fur: CornerOrientation,
    bur: CornerOrientation,
    fdl: CornerOrientation,
    fdr: CornerOrientation,
    bdl: CornerOrientation,
    bdr: CornerOrientation,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct CenterState {
    u: CenterCubelet,
    d: CenterCubelet,
    r: CenterCubelet,
    l: CenterCubelet,
    f: CenterCubelet,
    b: CenterCubelet,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Skewb {
    corner_pos: CornerPosState,
    corner_orr: CornerOrientationState,
    centers: CenterState,
}

trait SkewbState: Sized {
    fn start() -> Self;

    fn dfl(&self) -> Self;

    fn dfr(&self) -> Self;

    fn dbr(&self) -> Self;

    fn ufr(&self) -> Self;
}

impl SkewbState for CornerPosState {
    fn start() -> Self {
        Self {
            ful: CornerCubelet::FUL,
            fur: CornerCubelet::FUR,
            bur: CornerCubelet::BUR,
            fdl: CornerCubelet::FDL,
            fdr: CornerCubelet::FDR,
            bdl: CornerCubelet::BDL,
            bdr: CornerCubelet::BDR,
        }
    }

    #[inline(always)]
    fn dfl(&self) -> Self {
        Self {
            ful: self.bdl,
            fdr: self.ful,
            bdl: self.fdr,
            ..*self
        }
    }

    #[inline(always)]
    fn dfr(&self) -> Self {
        Self {
            fur: self.fdl,
            bdr: self.fur,
            fdl: self.bdr,
            ..*self
        }
    }

    #[inline(always)]
    fn dbr(&self) -> Self {
        Self {
            fdr: self.bdl,
            bur: self.fdr,
            bdl: self.bur,
            ..*self
        }
    }

    #[inline(always)]
    fn ufr(&self) -> Self {
        Self {
            ful: self.fdr,
            fdr: self.bur,
            bur: self.ful,
            ..*self
        }
    }
}

impl SkewbState for CornerOrientationState {
    fn start() -> Self {
        Self {
            ful: CornerOrientation::Normal,
            fur: CornerOrientation::Normal,
            bur: CornerOrientation::Normal,
            fdl: CornerOrientation::Normal,
            fdr: CornerOrientation::Normal,
            bdl: CornerOrientation::Normal,
            bdr: CornerOrientation::Normal,
        }
    }

    #[inline(always)]
    fn dfl(&self) -> Self {
        Self {
            fdl: self.fdl.cw(),
            ful: self.bdl.ccw(),
            fdr: self.ful.ccw(),
            bdl: self.fdr.ccw(),
            ..*self
        }
    }

    #[inline(always)]
    fn dfr(&self) -> Self {
        Self {
            fdr: self.fdr.cw(),
            fur: self.fdl.ccw(),
            bdr: self.fur.ccw(),
            fdl: self.bdr.ccw(),
            ..*self
        }
    }

    #[inline(always)]
    fn dbr(&self) -> Self {
        Self {
            bdr: self.bdr.cw(),
            fdr: self.bdl.ccw(),
            bur: self.fdr.ccw(),
            bdl: self.bur.ccw(),
            ..*self
        }
    }

    #[inline(always)]
    fn ufr(&self) -> Self {
        Self {
            fur: self.fur.cw(),
            ful: self.fdr.ccw(),
            fdr: self.bur.ccw(),
            bur: self.ful.ccw(),
            ..*self
        }
    }
}

impl SkewbState for CenterState {
    fn start() -> Self {
        Self {
            u: CenterCubelet::U,
            d: CenterCubelet::D,
            r: CenterCubelet::R,
            l: CenterCubelet::L,
            f: CenterCubelet::F,
            b: CenterCubelet::B,
        }
    }

    #[inline(always)]
    fn dfl(&self) -> Self {
        Self {
            f: self.l,
            l: self.d,
            d: self.f,
            ..*self
        }
    }

    #[inline(always)]
    fn dfr(&self) -> Self {
        Self {
            r: self.f,
            f: self.d,
            d: self.r,
            ..*self
        }
    }

    #[inline(always)]
    fn dbr(&self) -> Self {
        Self {
            b: self.r,
            d: self.b,
            r: self.d,
            ..*self
        }
    }

    #[inline(always)]
    fn ufr(&self) -> Self {
        Self {
            u: self.f,
            f: self.r,
            r: self.u,
            ..*self
        }
    }
}

impl SkewbState for Skewb {
    fn start() -> Self {
        Self {
            centers: CenterState::start(),
            corner_pos: CornerPosState::start(),
            corner_orr: CornerOrientationState::start(),
        }
    }

    #[inline(always)]
    fn dfl(&self) -> Self {
        Self {
            centers: self.centers.dfl(),
            corner_pos: self.corner_pos.dfl(),
            corner_orr: self.corner_orr.dfl(),
        }
    }

    #[inline(always)]
    fn dfr(&self) -> Self {
        Self {
            centers: self.centers.dfr(),
            corner_pos: self.corner_pos.dfr(),
            corner_orr: self.corner_orr.dfr(),
        }
    }

    #[inline(always)]
    fn dbr(&self) -> Self {
        Self {
            centers: self.centers.dbr(),
            corner_pos: self.corner_pos.dbr(),
            corner_orr: self.corner_orr.dbr(),
        }
    }

    #[inline(always)]
    fn ufr(&self) -> Self {
        Self {
            centers: self.centers.ufr(),
            corner_pos: self.corner_pos.ufr(),
            corner_orr: self.corner_orr.ufr(),
        }
    }
}

impl State for Skewb {
    type UniqueKey = u64;

    fn neighbors<Recv>(&self, to_add: &mut Recv)
    where
        Recv: FnMut(Self),
    {
        // four corners -- DFL, DFR, DBR, UFR -- with two possible orientations

        // DFL
        to_add(self.dfl());
        to_add(self.dfl().dfl());

        // DFR
        to_add(self.dfr());
        to_add(self.dfr().dfr());

        // DBR
        to_add(self.dbr());
        to_add(self.dbr().dbr());

        // UFR
        to_add(self.ufr());
        to_add(self.ufr().ufr());
    }

    fn start() -> Self {
        <Skewb as SkewbState>::start()
    }

    fn uniq_key(&self) -> Self::UniqueKey {
        let mut out: u64 = 0;

        // can't fit it into 32 bits
        debug_assert!(7 * 5 + 6 * 3 < 64, "State should fit into 64 bits");

        out = (out << 3) | self.corner_pos.fdr.as_u8_three_bits() as u64;
        out = (out << 3) | self.corner_pos.fdl.as_u8_three_bits() as u64;
        out = (out << 3) | self.corner_pos.ful.as_u8_three_bits() as u64;
        out = (out << 3) | self.corner_pos.fur.as_u8_three_bits() as u64;
        out = (out << 3) | self.corner_pos.bdr.as_u8_three_bits() as u64;
        out = (out << 3) | self.corner_pos.bur.as_u8_three_bits() as u64;
        out = (out << 3) | self.corner_pos.bdl.as_u8_three_bits() as u64;

        out = (out << 2) | self.corner_orr.fdr.as_u8_two_bits() as u64;
        out = (out << 2) | self.corner_orr.fdl.as_u8_two_bits() as u64;
        out = (out << 2) | self.corner_orr.ful.as_u8_two_bits() as u64;
        out = (out << 2) | self.corner_orr.fur.as_u8_two_bits() as u64;
        out = (out << 2) | self.corner_orr.bdr.as_u8_two_bits() as u64;
        out = (out << 2) | self.corner_orr.bur.as_u8_two_bits() as u64;
        out = (out << 2) | self.corner_orr.bdl.as_u8_two_bits() as u64;

        out = (out << 3) | self.centers.u.as_u8_three_bits() as u64;
        out = (out << 3) | self.centers.d.as_u8_three_bits() as u64;
        out = (out << 3) | self.centers.f.as_u8_three_bits() as u64;
        out = (out << 3) | self.centers.b.as_u8_three_bits() as u64;
        out = (out << 3) | self.centers.r.as_u8_three_bits() as u64;
        out = (out << 3) | self.centers.l.as_u8_three_bits() as u64;

        out
    }
}

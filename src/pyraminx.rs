use crate::cubesearch::State;
use crate::orientations::{CornerOrientation, EdgeOrientation};
use ahash::HashMap;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
enum EdgeCubelet {
    UB,
    UL,
    UR,
    DL,
    DR,
    DF,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct AxialState {
    u: CornerOrientation,
    l: CornerOrientation,
    r: CornerOrientation,
    b: CornerOrientation,
}

impl PyraminxState for AxialState {
    fn start() -> Self {
        Self {
            u: CornerOrientation::Normal,
            l: CornerOrientation::Normal,
            r: CornerOrientation::Normal,
            b: CornerOrientation::Normal,
        }
    }

    fn u(&self) -> Self {
        Self {
            u: self.u.cw(),
            ..*self
        }
    }

    fn r(&self) -> Self {
        Self {
            r: self.r.cw(),
            ..*self
        }
    }

    fn l(&self) -> Self {
        Self {
            l: self.l.cw(),
            ..*self
        }
    }

    fn b(&self) -> Self {
        Self {
            b: self.b.cw(),
            ..*self
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct EdgePositions {
    ul: EdgeCubelet,
    ur: EdgeCubelet,
    ub: EdgeCubelet,
    dl: EdgeCubelet,
    dr: EdgeCubelet,
    df: EdgeCubelet,
}

impl PyraminxState for EdgePositions {
    fn start() -> Self {
        Self {
            ul: EdgeCubelet::UL,
            ur: EdgeCubelet::UR,
            ub: EdgeCubelet::UB,
            dl: EdgeCubelet::DL,
            dr: EdgeCubelet::DR,
            df: EdgeCubelet::DF,
        }
    }

    fn u(&self) -> Self {
        Self {
            ul: self.ur,
            ur: self.ub,
            ub: self.ul,
            ..*self
        }
    }

    fn r(&self) -> Self {
        Self {
            ur: self.df,
            df: self.dr,
            dr: self.ur,
            ..*self
        }
    }

    fn l(&self) -> Self {
        Self {
            df: self.ul,
            ul: self.dl,
            dl: self.df,
            ..*self
        }
    }

    fn b(&self) -> Self {
        Self {
            ub: self.dr,
            dr: self.dl,
            dl: self.ub,
            ..*self
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct EdgeOrientations {
    ul: EdgeOrientation,
    ur: EdgeOrientation,
    ub: EdgeOrientation,
    dl: EdgeOrientation,
    dr: EdgeOrientation,
    df: EdgeOrientation,
}

impl PyraminxState for EdgeOrientations {
    fn start() -> Self {
        Self {
            ul: EdgeOrientation::Normal,
            ur: EdgeOrientation::Normal,
            ub: EdgeOrientation::Normal,
            dl: EdgeOrientation::Normal,
            dr: EdgeOrientation::Normal,
            df: EdgeOrientation::Normal,
        }
    }

    fn u(&self) -> Self {
        Self {
            ul: self.ur,
            ur: self.ub,
            ub: self.ul,
            ..*self
        }
    }

    fn r(&self) -> Self {
        Self {
            ur: self.df,
            df: self.dr.flipped(),
            dr: self.ur.flipped(),
            ..*self
        }
    }

    fn l(&self) -> Self {
        Self {
            df: self.ul.flipped(),
            ul: self.dl,
            dl: self.df.flipped(),
            ..*self
        }
    }

    fn b(&self) -> Self {
        Self {
            ub: self.dr,
            dr: self.dl.flipped(),
            dl: self.ub.flipped(),
            ..*self
        }
    }
}

trait PyraminxState: Sized {
    fn start() -> Self;

    fn u(&self) -> Self;

    fn r(&self) -> Self;

    fn l(&self) -> Self;

    fn b(&self) -> Self;
}

/// State of a pyraminx puzzle with no tips
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Pyraminx {
    axials: AxialState,
    edge_pos: EdgePositions,
    edge_orr: EdgeOrientations,
}

impl PyraminxState for Pyraminx {
    fn start() -> Self {
        Pyraminx {
            axials: AxialState::start(),
            edge_pos: EdgePositions::start(),
            edge_orr: EdgeOrientations::start(),
        }
    }

    #[inline(always)]
    fn u(&self) -> Self {
        Self {
            axials: self.axials.u(),
            edge_pos: self.edge_pos.u(),
            edge_orr: self.edge_orr.u(),
        }
    }

    #[inline(always)]
    fn r(&self) -> Self {
        Self {
            axials: self.axials.r(),
            edge_pos: self.edge_pos.r(),
            edge_orr: self.edge_orr.r(),
        }
    }

    #[inline(always)]
    fn l(&self) -> Self {
        Self {
            axials: self.axials.l(),
            edge_pos: self.edge_pos.l(),
            edge_orr: self.edge_orr.l(),
        }
    }

    #[inline(always)]
    fn b(&self) -> Self {
        Self {
            axials: self.axials.b(),
            edge_pos: self.edge_pos.b(),
            edge_orr: self.edge_orr.b(),
        }
    }
}

impl State for Pyraminx {
    type UniqueKey = Self;

    fn uniq_key(&self) -> Self {
        self.clone()
    }

    fn neighbors<Recv>(&self, to_add: &mut Recv)
    where
        Recv: FnMut(Self),
    {
        // U
        to_add(self.u());
        to_add(self.u().u());

        // R
        to_add(self.r());
        to_add(self.r().r());

        // L
        to_add(self.l());
        to_add(self.l().l());

        // B
        to_add(self.b());
        to_add(self.b().b());
    }

    fn start() -> Self {
        <Self as PyraminxState>::start()
    }
}

pub fn gn_count_with_tips(gn_count_no_tips: HashMap<u128, u128>) -> HashMap<u128, u128> {
    let mut out = HashMap::default();

    for (cost, count) in gn_count_no_tips {
        // For each no-tips state, there is one way for the tips to all be correct
        *out.entry(cost).or_insert(0) += count;

        // For each no-tips state, there are eight ways for the tips to be wrong one time --
        // four ways to pick the wrong tip, two ways for it to be wrong
        *out.entry(cost + 1).or_insert(0) += count * 8;

        // For each no-tips state, there are 24 ways for the tips to be wrong twice --
        // six ways to pick two wrong tips, and two ways (each) for them to be wrong
        *out.entry(cost + 2).or_insert(0) += count * 24;

        // For each no-tips state, there are 32 ways for the tips to be wrong thrice --
        // four ways to pick the three wrong tips, and two ways (each) for them to be wrong
        *out.entry(cost + 3).or_insert(0) += count * 32;

        // For each no-tips state, there are 16 ways for the tips to all be wrong --
        // one way to pick every tip (obviously?), and two ways (each) for them to be wrong
        *out.entry(cost + 4).or_insert(0) += count * 16;
    }

    out
}

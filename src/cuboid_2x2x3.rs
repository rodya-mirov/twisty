use crate::cubesearch::SimpleState;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
enum CornerCubelet {
    UFL,
    UFR,
    UBL,
    UBR,
    DFL,
    DFR,
    DBL,
    DBR,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
enum CenterCubelet {
    // we fix the BL center cubelet, so we don't need it here
    FL,
    FR,
    BR,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Cuboid2x2x3 {
    // eight corners
    ufl: CornerCubelet,
    ufr: CornerCubelet,
    ubl: CornerCubelet,
    ubr: CornerCubelet,
    dfl: CornerCubelet,
    dfr: CornerCubelet,
    dbl: CornerCubelet,
    dbr: CornerCubelet,

    // three movable centers
    flc: CenterCubelet,
    frc: CenterCubelet,
    brc: CenterCubelet,
}

impl Cuboid2x2x3 {
    fn solved() -> Self {
        Self {
            ufl: CornerCubelet::UFL,
            ufr: CornerCubelet::UFR,
            ubl: CornerCubelet::UBL,
            ubr: CornerCubelet::UBR,
            dfl: CornerCubelet::DFL,
            dfr: CornerCubelet::DFR,
            dbl: CornerCubelet::DBL,
            dbr: CornerCubelet::DBR,
            flc: CenterCubelet::FL,
            frc: CenterCubelet::FR,
            brc: CenterCubelet::BR,
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

    #[inline(always)]
    fn d(&self) -> Self {
        Self {
            dfl: self.dbl,
            dbl: self.dbr,
            dbr: self.dfr,
            dfr: self.dfl,
            ..*self
        }
    }

    #[inline(always)]
    fn f2(&self) -> Self {
        Self {
            // cycle front corners
            dfl: self.ufr,
            ufr: self.dfl,
            dfr: self.ufl,
            ufl: self.dfr,
            // mess with front two centers
            flc: self.frc,
            frc: self.flc,
            ..*self
        }
    }

    #[inline(always)]
    fn r2(&self) -> Self {
        Self {
            // cycle right corners
            dfr: self.ubr,
            ubr: self.dfr,
            dbr: self.ufr,
            ufr: self.dbr,
            // mess with right two centers
            frc: self.brc,
            brc: self.frc,
            ..*self
        }
    }

}

impl SimpleState for Cuboid2x2x3 {
    fn neighbors<Recv>(&self, to_add: &mut Recv)
    where
        Recv: FnMut(Self),
    {
        // U
        to_add(self.u());
        to_add(self.u().u());
        to_add(self.u().u().u());

        // D
        to_add(self.d());
        to_add(self.d().d());
        to_add(self.d().d().d());

        // F2
        to_add(self.f2());

        // R2
        to_add(self.r2());
    }

    fn start() -> Self {
        Self::solved()
    }
}

use crate::cubesearch::State;

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

impl State for Cuboid2x2x3 {
    fn neighbors<Recv>(&self, to_add: &mut Recv)
    where
        Recv: FnMut(Self),
    {
        // three neighbors: U2, R2, D2
        let Self {
            ufl,
            ufr,
            ubl,
            ubr,
            dfl,
            dfr,
            dbl,
            dbr,
            flc,
            frc,
            brc,
        } = *self;

        // U
        to_add(Self {
            ufl: ufr,
            ufr: ubr,
            ubr: ubl,
            ubl: ufl,
            ..*self
        });

        // U2
        to_add(Self {
            ufl: ubr,
            ubr: ufl,
            ufr: ubl,
            ubl: ufr,
            ..*self
        });

        // U'
        to_add(Self {
            ufl: ubl,
            ubl: ubr,
            ubr: ufr,
            ufr: ufl,
            ..*self
        });

        // D
        to_add(Self {
            dfl: dbl,
            dbl: dbr,
            dbr: dfr,
            dfr: dfl,
            ..*self
        });

        // D2
        to_add(Self {
            dfl: dbr,
            dbr: dfl,
            dfr: dbl,
            dbl: dfr,
            ..*self
        });

        // D'
        to_add(Self {
            dfl: dfr,
            dfr: dbr,
            dbr: dbl,
            dbl: dfl,
            ..*self
        });

        // F2
        to_add(Self {
            // cycle front corners
            dfl: ufr,
            ufr: dfl,
            dfr: ufl,
            ufl: dfr,
            // mess with front two centers
            flc: frc,
            frc: flc,
            ..*self
        });

        // R2
        to_add(Self {
            // cycle right corners
            dfr: ubr,
            ubr: dfr,
            dbr: ufr,
            ufr: dbr,
            // mess with right two centers
            frc: brc,
            brc: frc,
            ..*self
        });
    }

    fn start() -> Self {
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
}

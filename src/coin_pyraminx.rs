use crate::cubesearch::State;
use crate::orientations::CornerOrientation;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Ord, PartialOrd)]
enum FaceFacelet {
    F,
    D,
    L,
    R,
}

impl FaceFacelet {
    /// No logic to it, but encodes a facelet in two bits.
    #[inline(always)]
    fn as_u8_two_bits(self) -> u8 {
        match self {
            FaceFacelet::F => 0,
            FaceFacelet::D => 1,
            FaceFacelet::L => 2,
            FaceFacelet::R => 3,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Ord, PartialOrd)]
pub struct CoinPyraminx {
    // we envision the cube as having a flat face in front of us and a flat face down,
    // so the axials are U/L/R/B and the corresponding faces are D/R/L/F

    // axial states
    u_axial: CornerOrientation,
    l_axial: CornerOrientation,
    r_axial: CornerOrientation,
    b_axial: CornerOrientation,

    // F facelets; note that the face is the first letter, and the position within the face
    // (which is also the corresponding axial) is the second letter. Sometimes the order matters.
    fu: FaceFacelet,
    fl: FaceFacelet,
    fr: FaceFacelet,

    // L facelets; note that the face is the first letter, and the position within the face
    // (which is also the corresponding axial) is the second letter. Sometimes the order matters.
    lu: FaceFacelet,
    lb: FaceFacelet,
    ll: FaceFacelet,

    // R facelets; note that the face is the first letter, and the position within the face
    // (which is also the corresponding axial) is the second letter. Sometimes the order matters.
    ru: FaceFacelet,
    rb: FaceFacelet,
    rr: FaceFacelet,

    // D facelets; note that the face is the first letter, and the position within the face
    // (which is also the corresponding axial) is the second letter. Sometimes the order matters.
    dl: FaceFacelet,
    dr: FaceFacelet,
    db: FaceFacelet,
}

impl CoinPyraminx {
    #[inline(always)]
    fn r_axial(&self) -> Self {
        Self {
            r_axial: self.r_axial.cw(),
            fr: self.dr,
            dr: self.rr,
            rr: self.fr,
            ..*self
        }
    }

    #[inline(always)]
    fn l_axial(&self) -> Self {
        Self {
            l_axial: self.l_axial.cw(),
            fl: self.ll,
            ll: self.dl,
            dl: self.fl,
            ..*self
        }
    }

    #[inline(always)]
    fn u_axial(&self) -> Self {
        Self {
            u_axial: self.u_axial.cw(),
            fu: self.ru,
            ru: self.lu,
            lu: self.fu,
            ..*self
        }
    }

    #[inline(always)]
    fn b_axial(&self) -> Self {
        Self {
            b_axial: self.b_axial.cw(),
            rb: self.db,
            db: self.lb,
            lb: self.rb,
            ..*self
        }
    }

    #[inline(always)]
    fn r_face(&self) -> Self {
        Self {
            rr: self.rb,
            rb: self.ru,
            ru: self.rr,
            ..*self
        }
    }

    #[inline(always)]
    fn l_face(&self) -> Self {
        Self {
            ll: self.lu,
            lu: self.lb,
            lb: self.ll,
            ..*self
        }
    }

    #[inline(always)]
    fn f_face(&self) -> Self {
        Self {
            fr: self.fu,
            fu: self.fl,
            fl: self.fr,
            ..*self
        }
    }

    #[inline(always)]
    fn d_face(&self) -> Self {
        Self {
            db: self.dr,
            dr: self.dl,
            dl: self.db,
            ..*self
        }
    }
}

impl State for CoinPyraminx {
    type UniqueKey = u32;

    fn uniq_key(&self) -> u32 {
        let mut out: u32 = 0;

        debug_assert!(8 + 6 + 6 + 6 < 32, "Bitpacking should fit");

        // 8 bits for the axials
        out = (out << 2) | self.r_axial.as_u8_two_bits() as u32;
        out = (out << 2) | self.l_axial.as_u8_two_bits() as u32;
        out = (out << 2) | self.b_axial.as_u8_two_bits() as u32;
        out = (out << 2) | self.u_axial.as_u8_two_bits() as u32;

        // 6 more bits for F face
        out = (out << 2) | self.fr.as_u8_two_bits() as u32;
        out = (out << 2) | self.fl.as_u8_two_bits() as u32;
        out = (out << 2) | self.fu.as_u8_two_bits() as u32;

        // 6 more bits for L face
        out = (out << 2) | self.ll.as_u8_two_bits() as u32;
        out = (out << 2) | self.lb.as_u8_two_bits() as u32;
        out = (out << 2) | self.lu.as_u8_two_bits() as u32;

        // 6 more bits for R face
        out = (out << 2) | self.rr.as_u8_two_bits() as u32;
        out = (out << 2) | self.rb.as_u8_two_bits() as u32;
        out = (out << 2) | self.ru.as_u8_two_bits() as u32;

        // 6 more bits for D face
        out = (out << 2) | self.db.as_u8_two_bits() as u32;
        out = (out << 2) | self.dl.as_u8_two_bits() as u32;
        out = (out << 2) | self.dr.as_u8_two_bits() as u32;

        out
    }
    
    fn neighbors<Recv>(&self, to_add: &mut Recv)
    where
        Recv: FnMut(Self),
    {
        to_add(self.u_axial());
        to_add(self.u_axial().u_axial());

        to_add(self.r_axial());
        to_add(self.r_axial().r_axial());

        to_add(self.l_axial());
        to_add(self.l_axial().l_axial());

        to_add(self.b_axial());
        to_add(self.b_axial().b_axial());

        to_add(self.f_face());
        to_add(self.f_face().f_face());

        to_add(self.r_face());
        to_add(self.r_face().r_face());

        to_add(self.l_face());
        to_add(self.l_face().l_face());

        to_add(self.d_face());
        to_add(self.d_face().d_face());
    }

    fn start() -> Self {
        Self {
            // axials
            u_axial: CornerOrientation::Normal,
            l_axial: CornerOrientation::Normal,
            r_axial: CornerOrientation::Normal,
            b_axial: CornerOrientation::Normal,

            // F facelets
            fu: FaceFacelet::F,
            fl: FaceFacelet::F,
            fr: FaceFacelet::F,

            // L facelets
            lu: FaceFacelet::L,
            lb: FaceFacelet::L,
            ll: FaceFacelet::L,

            // R facelets
            ru: FaceFacelet::R,
            rb: FaceFacelet::R,
            rr: FaceFacelet::R,

            // D facelets
            dl: FaceFacelet::D,
            dr: FaceFacelet::D,
            db: FaceFacelet::D,
        }
    }
}

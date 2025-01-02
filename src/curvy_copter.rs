//! Curvy copter. No jumbling today.

use crate::cubesearch::SimpleStartState;
use crate::idasearch::heuristic_helpers::bounded_cache;
use crate::idasearch::{Heuristic, Solvable};
use crate::moves::CanReverse;
use crate::orientations::{CornerOrientation, EdgeOrientation};
use crate::random_helpers;
use crate::random_helpers::{shuffle_with_parity, TwoParity};
use crate::scrambles::RandomInit;
use derive_more::Display;
use rand::Rng;

type PackedBits = (u64, u64);

// pretty clearly 12 bits to pack this, no matter what you do
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
struct EdgeStates {
    uf: EdgeOrientation,
    ur: EdgeOrientation,
    ul: EdgeOrientation,
    ub: EdgeOrientation,

    df: EdgeOrientation,
    dr: EdgeOrientation,
    dl: EdgeOrientation,
    db: EdgeOrientation,

    fl: EdgeOrientation,
    fr: EdgeOrientation,
    bl: EdgeOrientation,
    br: EdgeOrientation,
}

macro_rules! flip_edge {
    ($edge_name:ident) => {
        #[inline(always)]
        fn $edge_name(&self) -> Self {
            Self {
                $edge_name: self.$edge_name.flipped(),
                ..*self
            }
        }
    };
}

impl EdgeStates {
    fn solved() -> Self {
        Self::default()
    }

    // 12 * 1 == 12 bits
    fn pack(&self, bits: &mut u64) {
        self.uf.pack(bits);
        self.ur.pack(bits);
        self.ul.pack(bits);
        self.ub.pack(bits);

        self.df.pack(bits);
        self.dr.pack(bits);
        self.dl.pack(bits);
        self.db.pack(bits);

        self.fl.pack(bits);
        self.fr.pack(bits);
        self.bl.pack(bits);
        self.br.pack(bits);
    }

    flip_edge!(uf);
    flip_edge!(ur);
    flip_edge!(ul);
    flip_edge!(ub);

    flip_edge!(df);
    flip_edge!(dr);
    flip_edge!(dl);
    flip_edge!(db);

    flip_edge!(fl);
    flip_edge!(fr);
    flip_edge!(bl);
    flip_edge!(br);
}

// 3 bits each
// can pack a little tighter if we multiply by 6 at each point instead of <<< 3
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum CenterCubelet {
    F,
    B,
    L,
    R,
    U,
    D,
}

impl CenterCubelet {
    #[inline(always)]
    fn pack_tight(self, bits: &mut u64) {
        *bits = (*bits * 6) + ((self as u8) as u64);
    }
}

// 24 pieces at 3 bits each equals 72 bits to pack
// with tighter packing (mul by 6 each time) we can fit into 63 bits (!)
#[derive(Clone, Eq, PartialEq)]
struct CenterStates {
    // front face
    f_ul: CenterCubelet,
    f_ur: CenterCubelet,
    f_dl: CenterCubelet,
    f_dr: CenterCubelet,

    // back face
    b_ul: CenterCubelet,
    b_ur: CenterCubelet,
    b_dl: CenterCubelet,
    b_dr: CenterCubelet,

    // left face
    l_ub: CenterCubelet,
    l_uf: CenterCubelet,
    l_db: CenterCubelet,
    l_df: CenterCubelet,

    // right face
    r_ub: CenterCubelet,
    r_uf: CenterCubelet,
    r_db: CenterCubelet,
    r_df: CenterCubelet,

    // up face
    u_bl: CenterCubelet,
    u_br: CenterCubelet,
    u_fl: CenterCubelet,
    u_fr: CenterCubelet,

    // down face
    d_bl: CenterCubelet,
    d_br: CenterCubelet,
    d_fl: CenterCubelet,
    d_fr: CenterCubelet,
}

macro_rules! swap_centers {
    // turning edge_name should swap a_1 and a_2 as well as b_1 and b_2
    ($edge_name:ident, $a_1:ident, $a_2:ident, $b_1:ident, $b_2:ident) => {
        #[inline(always)]
        fn $edge_name(&self) -> Self {
            Self {
                $a_2: self.$a_1,
                $a_1: self.$a_2,

                $b_2: self.$b_1,
                $b_1: self.$b_2,

                ..*self
            }
        }
    };
}

impl CenterStates {
    fn solved() -> Self {
        Self {
            f_ul: CenterCubelet::F,
            f_ur: CenterCubelet::F,
            f_dl: CenterCubelet::F,
            f_dr: CenterCubelet::F,

            b_ul: CenterCubelet::B,
            b_ur: CenterCubelet::B,
            b_dl: CenterCubelet::B,
            b_dr: CenterCubelet::B,

            l_ub: CenterCubelet::L,
            l_uf: CenterCubelet::L,
            l_db: CenterCubelet::L,
            l_df: CenterCubelet::L,

            r_ub: CenterCubelet::R,
            r_uf: CenterCubelet::R,
            r_db: CenterCubelet::R,
            r_df: CenterCubelet::R,

            u_bl: CenterCubelet::U,
            u_br: CenterCubelet::U,
            u_fl: CenterCubelet::U,
            u_fr: CenterCubelet::U,

            d_bl: CenterCubelet::D,
            d_br: CenterCubelet::D,
            d_fl: CenterCubelet::D,
            d_fr: CenterCubelet::D,
        }
    }

    fn pack(&self, bits: &mut u64) {
        // F face
        self.f_ul.pack_tight(bits);
        self.f_ur.pack_tight(bits);
        self.f_dl.pack_tight(bits);
        self.f_dr.pack_tight(bits);

        // B face
        self.b_ul.pack_tight(bits);
        self.b_ur.pack_tight(bits);
        self.b_dl.pack_tight(bits);
        self.b_dr.pack_tight(bits);

        // L face
        self.l_db.pack_tight(bits);
        self.l_df.pack_tight(bits);
        self.l_ub.pack_tight(bits);
        self.l_uf.pack_tight(bits);

        // R face
        self.r_db.pack_tight(bits);
        self.r_df.pack_tight(bits);
        self.r_ub.pack_tight(bits);
        self.r_uf.pack_tight(bits);

        // U face
        self.u_bl.pack_tight(bits);
        self.u_br.pack_tight(bits);
        self.u_fl.pack_tight(bits);
        self.u_fr.pack_tight(bits);

        // D face
        self.d_bl.pack_tight(bits);
        self.d_br.pack_tight(bits);
        self.d_fl.pack_tight(bits);
        self.d_fr.pack_tight(bits);
    }

    // macros to define all the edge flips, otherwise so much repeated code

    // top layer edges
    swap_centers!(uf, u_fl, f_ur, u_fr, f_ul);
    swap_centers!(ul, u_bl, l_uf, u_fl, l_ub);
    swap_centers!(ur, u_br, r_uf, u_fr, r_ub);
    swap_centers!(ub, u_bl, b_ur, u_br, b_ul);

    // bottom layer edges
    swap_centers!(df, d_fl, f_dr, d_fr, f_dl);
    swap_centers!(dl, d_bl, l_df, d_fl, l_db);
    swap_centers!(dr, d_br, r_df, d_fr, r_db);
    swap_centers!(db, d_bl, b_dr, d_br, b_dl);

    // mid layer edges
    swap_centers!(fl, f_ul, l_df, f_dl, l_uf);
    swap_centers!(fr, f_ur, r_df, f_dr, r_uf);
    swap_centers!(bl, b_ul, l_db, b_dl, l_ub);
    swap_centers!(br, b_ur, r_db, b_dr, r_ub);
}

// 8 values; takes 3 bits no matter how you slice it
#[repr(u8)]
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
enum CornerCubelet {
    FUL,
    FUR,
    BUL,
    BUR,
    FDL,
    FDR,
    BDL,
    BDR,
}

impl CornerCubelet {
    // 3 bits
    fn pack(self, bits: &mut u64) {
        *bits = (*bits << 3) + (self as u8 as u64);
    }
}

// 3 bits each * 8 corners -> 24 bits total, although we can skip the last one
// since it's a permutation
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct CornersPositionState {
    ful: CornerCubelet,
    fur: CornerCubelet,
    fdl: CornerCubelet,
    fdr: CornerCubelet,

    bul: CornerCubelet,
    bur: CornerCubelet,
    bdl: CornerCubelet,
    bdr: CornerCubelet,
}

macro_rules! swap_corner_pos {
    ($edge_name: ident, $corner_a:ident, $corner_b:ident) => {
        #[inline(always)]
        fn $edge_name(&self) -> Self {
            Self {
                $corner_a: self.$corner_b,
                $corner_b: self.$corner_a,
                ..*self
            }
        }
    };
}

impl CornersPositionState {
    fn solved() -> Self {
        Self {
            ful: CornerCubelet::FUL,
            fur: CornerCubelet::FUR,
            fdl: CornerCubelet::FDL,
            fdr: CornerCubelet::FDR,
            bul: CornerCubelet::BUL,
            bur: CornerCubelet::BUR,
            bdl: CornerCubelet::BDL,
            bdr: CornerCubelet::BDR,
        }
    }

    // 8 * 3 == 24 bits
    fn pack(&self, bits: &mut u64) {
        self.fur.pack(bits);
        self.ful.pack(bits);
        self.fdr.pack(bits);
        self.fdl.pack(bits);

        self.bur.pack(bits);
        self.bul.pack(bits);
        self.bdr.pack(bits);
        self.bdl.pack(bits);
    }

    swap_corner_pos!(uf, fur, ful);
    swap_corner_pos!(ul, ful, bul);
    swap_corner_pos!(ur, fur, bur);
    swap_corner_pos!(ub, bur, bul);

    swap_corner_pos!(df, fdr, fdl);
    swap_corner_pos!(dl, fdl, bdl);
    swap_corner_pos!(dr, fdr, bdr);
    swap_corner_pos!(db, bdr, bdl);

    swap_corner_pos!(fl, ful, fdl);
    swap_corner_pos!(fr, fur, fdr);
    swap_corner_pos!(bl, bul, bdl);
    swap_corner_pos!(br, bur, bdr);
}

// 2 bits each * 8 corners -> 16 bits total
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct CornersOrientationState {
    ful: CornerOrientation,
    fur: CornerOrientation,
    fdl: CornerOrientation,
    fdr: CornerOrientation,

    bul: CornerOrientation,
    bur: CornerOrientation,
    bdl: CornerOrientation,
    bdr: CornerOrientation,
}

macro_rules! swap_corner_orr {
    ($edge_name:ident, $corner_a:ident, $a_swap:ident, $corner_b:ident, $b_swap:ident) => {
        #[inline(always)]
        fn $edge_name(&self) -> Self {
            Self {
                $corner_a: self.$corner_b.$b_swap(),
                $corner_b: self.$corner_a.$a_swap(),
                ..*self
            }
        }
    };
}

impl CornersOrientationState {
    fn solved() -> Self {
        Self {
            ful: CornerOrientation::Normal,
            fur: CornerOrientation::Normal,
            fdl: CornerOrientation::Normal,
            fdr: CornerOrientation::Normal,
            bul: CornerOrientation::Normal,
            bur: CornerOrientation::Normal,
            bdl: CornerOrientation::Normal,
            bdr: CornerOrientation::Normal,
        }
    }

    // 16 bits
    fn pack(&self, bits: &mut u64) {
        self.fur.pack_two_bits_u64(bits);
        self.ful.pack_two_bits_u64(bits);
        self.fdr.pack_two_bits_u64(bits);
        self.fdl.pack_two_bits_u64(bits);

        self.bur.pack_two_bits_u64(bits);
        self.bul.pack_two_bits_u64(bits);
        self.bdr.pack_two_bits_u64(bits);
        self.bdl.pack_two_bits_u64(bits);
    }

    // top layer; this CW's one of the corners, and CCW's the other
    // to figure out which is which i just tried it out :shrug:
    swap_corner_orr!(uf, ful, ccw, fur, cw);
    swap_corner_orr!(ub, bur, ccw, bul, cw);
    swap_corner_orr!(ul, ful, cw, bul, ccw);
    swap_corner_orr!(ur, fur, ccw, bur, cw);

    // mid layer is simpler -- no orientations change
    swap_corner_orr!(fr, fur, no_swap, fdr, no_swap);
    swap_corner_orr!(fl, ful, no_swap, fdl, no_swap);
    swap_corner_orr!(br, bur, no_swap, bdr, no_swap);
    swap_corner_orr!(bl, bul, no_swap, bdl, no_swap);

    // bot layer is pretty much like the top layer
    swap_corner_orr!(df, fdl, cw, fdr, ccw);
    swap_corner_orr!(db, bdr, cw, bdl, ccw);
    swap_corner_orr!(dl, fdl, ccw, bdl, cw);
    swap_corner_orr!(dr, fdr, cw, bdr, ccw);
}

// can BARELY be packed into a u128 (or probably a pair of u64)
#[derive(Clone, Eq, PartialEq)]
pub struct CurvyCopter {
    // 12 bits
    edges: EdgeStates,
    // 63 bits if you pack tight
    centers: CenterStates,
    // 24 bits
    corner_positions: CornersPositionState,
    // 16 bits
    corner_orientations: CornersOrientationState,
}

macro_rules! pass_through {
    ($move_name:ident) => {
        #[inline(always)]
        fn $move_name(&self) -> Self {
            Self {
                edges: self.edges.$move_name(),
                centers: self.centers.$move_name(),
                corner_positions: self.corner_positions.$move_name(),
                corner_orientations: self.corner_orientations.$move_name(),
            }
        }
    };
}

impl CurvyCopter {
    #[inline(always)]
    fn solved() -> Self {
        CurvyCopter {
            edges: EdgeStates::solved(),
            centers: CenterStates::solved(),
            corner_positions: CornersPositionState::solved(),
            corner_orientations: CornersOrientationState::solved(),
        }
    }

    pass_through!(uf);
    pass_through!(ur);
    pass_through!(ul);
    pass_through!(ub);

    pass_through!(df);
    pass_through!(dr);
    pass_through!(dl);
    pass_through!(db);

    pass_through!(fr);
    pass_through!(fl);
    pass_through!(br);
    pass_through!(bl);
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display)]
pub enum Move {
    UF,
    UL,
    UR,
    UB,
    DF,
    DL,
    DR,
    DB,
    FL,
    FR,
    BL,
    BR,
}

impl CanReverse for Move {
    fn reverse(&self) -> Self {
        // all moves are self inverse
        *self
    }
}

fn total_parity(eo: &[EdgeOrientation]) -> TwoParity {
    let mut total_flipped = EdgeOrientation::Normal;

    for e in eo.iter().copied() {
        total_flipped = match e {
            EdgeOrientation::Normal => total_flipped,
            EdgeOrientation::Flipped => total_flipped.flipped(),
        };
    }

    match total_flipped {
        EdgeOrientation::Normal => TwoParity::Even,
        EdgeOrientation::Flipped => TwoParity::Odd,
    }
}

fn take_six<T: Copy>(v: Vec<T>) -> [T; 6] {
    assert_eq!(v.len(), 6);

    // i feel like i should be able to use const generics here but whatever

    [v[0], v[1], v[2], v[3], v[4], v[5]]
}

impl RandomInit for CurvyCopter {
    fn random_state<R: Rng>(r: &mut R) -> Self {
        // parity of edge flips and corner orientations should match
        let corners = vec![
            CornerCubelet::FUL,
            CornerCubelet::FUR,
            CornerCubelet::FDL,
            CornerCubelet::FDR,
            CornerCubelet::BUL,
            CornerCubelet::BUR,
            CornerCubelet::BDL,
            CornerCubelet::BDR,
        ];
        let (corner_cubelets, perm_parity) = random_helpers::shuffle_any(r, corners);
        let edge_flips = random_helpers::flips_with_parity(r, 12, perm_parity);

        let corner_positions = CornersPositionState {
            ful: corner_cubelets[0],
            fur: corner_cubelets[1],
            fdl: corner_cubelets[2],
            fdr: corner_cubelets[3],
            bul: corner_cubelets[4],
            bur: corner_cubelets[5],
            bdl: corner_cubelets[6],
            bdr: corner_cubelets[7],
        };

        let edges = EdgeStates {
            uf: edge_flips[0],
            ur: edge_flips[1],
            ul: edge_flips[2],
            ub: edge_flips[3],
            df: edge_flips[4],
            dr: edge_flips[5],
            dl: edge_flips[6],
            db: edge_flips[7],
            fl: edge_flips[8],
            fr: edge_flips[9],
            bl: edge_flips[10],
            br: edge_flips[11],
        };

        let mut corner_orientations: Vec<CornerOrientation> =
            vec![r.gen(), r.gen(), r.gen(), r.gen(), r.gen(), r.gen(), r.gen()];
        let total_orientation = CornerOrientation::total(&corner_orientations);
        corner_orientations.push(total_orientation.flip());

        // total orientation of corners should be zero
        let corner_orientations = CornersOrientationState {
            ful: corner_orientations[0],
            fur: corner_orientations[1],
            fdl: corner_orientations[2],
            fdr: corner_orientations[3],
            bul: corner_orientations[4],
            bur: corner_orientations[5],
            bdl: corner_orientations[6],
            bdr: corner_orientations[7],
        };

        // now the pain of it is that each orbit of centers has to be permuted separately
        // and that orbit's parity is equal to the flips from the edges that touch it

        // Orbit 1 -- U_FL, F_UR, R_DF, D_BR, B_DL, L_UB
        //      uses edges UF, FR, DR, DB, BL, UL
        let parity = total_parity(&[edges.uf, edges.fr, edges.dr, edges.db, edges.bl, edges.ul]);
        let centers = vec![
            CenterCubelet::U,
            CenterCubelet::F,
            CenterCubelet::R,
            CenterCubelet::D,
            CenterCubelet::B,
            CenterCubelet::L,
        ];
        let centers = shuffle_with_parity(r, &centers, parity);
        let [u_fl, f_ur, r_df, d_br, b_dl, l_ub] = take_six(centers);

        // Orbit 2 -- U_FR, R_UB, B_DR, D_BL, L_DF, F_UL
        //      uses edges UR, BR, DB, DL, FL, UF
        let parity = total_parity(&[edges.ur, edges.br, edges.db, edges.dl, edges.fl, edges.uf]);
        let centers = vec![
            CenterCubelet::U,
            CenterCubelet::R,
            CenterCubelet::B,
            CenterCubelet::D,
            CenterCubelet::L,
            CenterCubelet::F,
        ];
        let centers = shuffle_with_parity(r, &centers, parity);
        let [u_fr, r_ub, b_dr, d_bl, l_df, f_ul] = take_six(centers);

        // Orbit 3 -- U_BL, L_UF, F_DL, D_FR, R_DB, B_UR
        //      uses edges UL, FL, DF, DR, BR, UB
        let parity = total_parity(&[edges.ul, edges.fl, edges.df, edges.dr, edges.br, edges.ub]);
        let centers = vec![
            CenterCubelet::U,
            CenterCubelet::L,
            CenterCubelet::F,
            CenterCubelet::D,
            CenterCubelet::R,
            CenterCubelet::B,
        ];
        let centers = shuffle_with_parity(r, &centers, parity);
        let [u_bl, l_uf, f_dl, d_fr, r_db, b_ur] = take_six(centers);

        // Orbit 4 -- U_BR, R_UF, F_DR, D_FL, L_DB, B_UL
        //      uses edges UR, FR, DF, DL, BL, UB
        let parity = total_parity(&[edges.ur, edges.fr, edges.df, edges.dl, edges.bl, edges.ub]);
        let centers = vec![
            CenterCubelet::U,
            CenterCubelet::R,
            CenterCubelet::F,
            CenterCubelet::D,
            CenterCubelet::L,
            CenterCubelet::B,
        ];
        let centers = shuffle_with_parity(r, &centers, parity);
        let [u_br, r_uf, f_dr, d_fl, l_db, b_ul] = take_six(centers);

        let centers = CenterStates {
            // orbit 1
            u_fl,
            f_ur,
            r_df,
            d_br,
            b_dl,
            l_ub,

            // orbit 2
            u_fr,
            r_ub,
            b_dr,
            d_bl,
            l_df,
            f_ul,

            // orbit 3
            u_bl,
            l_uf,
            f_dl,
            d_fr,
            r_db,
            b_ur,

            // orbit 4
            u_br,
            r_uf,
            f_dr,
            d_fl,
            l_db,
            b_ul,
        };

        CurvyCopter {
            edges,
            centers,
            corner_positions,
            corner_orientations,
        }
    }
}

impl SimpleStartState for CurvyCopter {
    type UniqueKey = PackedBits;

    fn start() -> Self {
        Self::solved()
    }

    fn uniq_key(&self) -> Self::UniqueKey {
        // center state needs 63 bits
        let mut center_bits: u64 = 0;

        self.centers.pack(&mut center_bits);

        // everything else fits in 52 bits
        let mut other_bits: u64 = 0;

        self.edges.pack(&mut other_bits); // 12 bits
        self.corner_positions.pack(&mut other_bits); // 24 bits
        self.corner_orientations.pack(&mut other_bits); // 16 bits

        (center_bits, other_bits)
    }
}

impl Solvable for CurvyCopter {
    type Move = Move;

    fn max_fuel() -> usize {
        // i guess? this probably isn't enough but we need some A* business in this business
        24
    }

    fn apply(&self, m: Self::Move) -> Self {
        match m {
            Move::UF => self.uf(),
            Move::UL => self.ul(),
            Move::UR => self.ur(),
            Move::UB => self.ub(),
            Move::DF => self.df(),
            Move::DL => self.dl(),
            Move::DR => self.dr(),
            Move::DB => self.db(),
            Move::FL => self.fl(),
            Move::FR => self.fr(),
            Move::BL => self.bl(),
            Move::BR => self.br(),
        }
    }

    fn available_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        [
            Move::UF,
            Move::UL,
            Move::UR,
            Move::UB,
            Move::DF,
            Move::DL,
            Move::DR,
            Move::DB,
            Move::FL,
            Move::FR,
            Move::BL,
            Move::BR,
        ]
    }

    fn is_solved(&self) -> bool {
        self == &Self::solved()
    }

    fn is_redundant(last_move: Self::Move, next_move: Self::Move) -> bool {
        // lots of edges commute with each other; we can cut the branching factor significantly
        // with ordering. Basically we'll say if A > B, then A has to go last. Edge ordering:
        //      UF > UL > UR > UB > DF > DL > DR > DB > FL > FR > BL > BR
        // an edge interacts with another edge if they have one letter in common and the other letter
        // is adjacent (e.g. UL interacts with UF but not UR)
        // also obviously everything commutes with itself and there's no point in repeating a move
        match last_move {
            // top layer
            Move::UF => next_move == Move::UF,
            Move::UL => next_move == Move::UL,
            Move::UR => next_move == Move::UR || next_move == Move::UL,
            Move::UB => next_move == Move::UB || next_move == Move::UF,
            // bottom later; these commute with all top layer edges and a little bit within the layer
            Move::DF => {
                next_move == Move::DF
                    || next_move == Move::UF
                    || next_move == Move::UL
                    || next_move == Move::UR
                    || next_move == Move::UB
            }
            Move::DL => {
                next_move == Move::DL
                    || next_move == Move::UF
                    || next_move == Move::UL
                    || next_move == Move::UR
                    || next_move == Move::UB
            }
            Move::DR => {
                next_move == Move::DR
                    || next_move == Move::UF
                    || next_move == Move::UL
                    || next_move == Move::UR
                    || next_move == Move::UB
                    || next_move == Move::DL
            }
            Move::DB => {
                next_move == Move::DB
                    || next_move == Move::UF
                    || next_move == Move::UL
                    || next_move == Move::UR
                    || next_move == Move::UB
                    || next_move == Move::DF
            }
            // mid layer; these commute with some top and some bottom edges as well as ALL mid layer edges
            Move::FL => {
                next_move == Move::FL
                    || next_move == Move::UB
                    || next_move == Move::UR
                    || next_move == Move::DB
                    || next_move == Move::DR
            }
            Move::FR => {
                next_move == Move::FR
                    || next_move == Move::UB
                    || next_move == Move::UL
                    || next_move == Move::DB
                    || next_move == Move::DL
                    || next_move == Move::FL
            }
            Move::BL => {
                next_move == Move::BL
                    || next_move == Move::UF
                    || next_move == Move::UR
                    || next_move == Move::DF
                    || next_move == Move::DR
                    || next_move == Move::FL
                    || next_move == Move::FR
            }
            Move::BR => {
                next_move == Move::BR
                    || next_move == Move::UF
                    || next_move == Move::UL
                    || next_move == Move::DF
                    || next_move == Move::DL
                    || next_move == Move::FL
                    || next_move == Move::FR
                    || next_move == Move::BL
            }
        }
    }
}

pub fn make_heuristic() -> impl Heuristic<CurvyCopter> {
    // max depth is picked to keep the compute time low
    bounded_cache::<CurvyCopter>(9)
}


use super::*;

const FLOPPY_133: Floppy1xMxN<1, 1> = Floppy1xMxN::<1, 1> {
    ul: CornerCubelet::UL,
    ur: CornerCubelet::UR,
    dr: CornerCubelet::DR,

    centers: [[true]],

    left_edge_pos: [true],
    left_edge_orr: [true],

    right_edge_pos: [true],
    right_edge_orr: [true],

    top_edge_pos: [true],
    top_edge_orr: [true],

    bot_edge_pos: [true],
    bot_edge_orr: [true],
};

#[test]
fn test_u2_133() {
    let start = FLOPPY_133;

    let actual = start.u2(0);

    let expected = Floppy1xMxN::<1, 1> {
        ul: CornerCubelet::UR,
        ur: CornerCubelet::UL,

        top_edge_pos: [true],
        top_edge_orr: [false],

        ..start
    };

    assert_eq!(actual, expected);
}

#[test]
fn test_uw2_133() {
    let start = FLOPPY_133;

    let actual = start.u2(1);

    let expected = Floppy1xMxN::<1, 1> {
        ul: CornerCubelet::UR,
        ur: CornerCubelet::UL,

        top_edge_pos: [true],
        top_edge_orr: [false],

        left_edge_orr: [false],
        left_edge_pos: [false],

        centers: [[false]],

        right_edge_orr: [false],
        right_edge_pos: [false],

        ..start
    };

    assert_eq!(actual, expected);
}

#[test]
fn test_r2_133() {
    let start = FLOPPY_133;

    let actual = start.r2(0);

    let expected = Floppy1xMxN::<1, 1> {
        ur: CornerCubelet::DR,
        dr: CornerCubelet::UR,

        right_edge_orr: [false],

        ..start
    };

    assert_eq!(actual, expected);
}

#[test]
fn test_rw2_133() {
    let start = FLOPPY_133;

    let actual = start.r2(1);

    let expected = Floppy1xMxN::<1, 1> {
        ur: CornerCubelet::DR,
        dr: CornerCubelet::UR,

        right_edge_orr: [false],

        top_edge_pos: [false],
        top_edge_orr: [false],

        centers: [[false]],

        bot_edge_pos: [false],
        bot_edge_orr: [false],

        ..start
    };

    assert_eq!(actual, expected);
}
use super::*;

// 1 center row, 2 center columns
const FLOPPY_134: Floppy1xMxN<1, 2> = Floppy1xMxN::<1, 2> {
    ul: CornerCubelet::UL,
    ur: CornerCubelet::UR,
    dr: CornerCubelet::DR,

    centers: [[true, true]],

    left_edge_pos: [true],
    left_edge_orr: [true],

    right_edge_pos: [true],
    right_edge_orr: [true],

    top_edge_pos: [true, true],
    top_edge_orr: [true, true],

    bot_edge_pos: [true, true],
    bot_edge_orr: [true, true],
};

#[test]
fn test_u2_134() {
    let start = FLOPPY_134;

    let actual = start.u2(0);

    let expected = Floppy1xMxN::<1, 2> {
        ul: CornerCubelet::UR,
        ur: CornerCubelet::UL,

        top_edge_pos: [true, true],
        top_edge_orr: [false, false],

        ..start
    };

    assert_eq!(actual, expected);
}

#[test]
fn test_uw2_134() {
    let start = FLOPPY_134;

    let actual = start.u2(1);

    let expected = Floppy1xMxN::<1, 2> {
        ul: CornerCubelet::UR,
        ur: CornerCubelet::UL,

        top_edge_pos: [true, true],
        top_edge_orr: [false, false],

        left_edge_orr: [false],
        left_edge_pos: [false],

        centers: [[false, false]],

        right_edge_orr: [false],
        right_edge_pos: [false],

        ..start
    };

    assert_eq!(actual, expected);
}

#[test]
fn test_r2_134() {
    let start = FLOPPY_134;

    let actual = start.r2(0);

    let expected = Floppy1xMxN::<1, 2> {
        ur: CornerCubelet::DR,
        dr: CornerCubelet::UR,

        right_edge_orr: [false],

        ..start
    };

    assert_eq!(actual, expected);
}

#[test]
fn test_rw2_134() {
    let start = FLOPPY_134;

    let actual = start.r2(1);

    let expected = Floppy1xMxN::<1, 2> {
        ur: CornerCubelet::DR,
        dr: CornerCubelet::UR,

        right_edge_orr: [false],

        // indexing is right first, then left
        top_edge_pos: [false, true],
        top_edge_orr: [false, true],

        // indexing is top first, then down
        centers: [[false, true]],

        bot_edge_pos: [false, true],
        bot_edge_orr: [false, true],

        ..start
    };

    assert_eq!(actual, expected);
}

#[test]
fn test_rww2_134() {
    let start = FLOPPY_134;

    let actual = start.r2(2);

    let expected = Floppy1xMxN::<1, 2> {
        ur: CornerCubelet::DR,
        dr: CornerCubelet::UR,

        right_edge_orr: [false],

        // indexing is right first, then left
        top_edge_pos: [false, false],
        top_edge_orr: [false, false],

        // indexing is top first, then down
        centers: [[false, false]],

        bot_edge_pos: [false, false],
        bot_edge_orr: [false, false],

        ..start
    };

    assert_eq!(actual, expected);
}

//! A relation an author adds or removes moves the graph in both directions.

use crate::common::{Commit, Edge_Count, SYNTHETIC, With_Synthetic};

/// A relation the author adds reaches the graph, and one they remove leaves it — including its
/// inverse, or half the fact stays behind.
#[test]
fn Test_Editing_A_Relation_Should_Move_The_Graph_Both_Ways()
{
    let mut store = With_Synthetic();
    let retargeted = SYNTHETIC.replace(
        "  - target: D-129\n    type: relates-to\n",
        "  - target: D-130\n    type: affects\n",
    );

    Commit(&mut store, &retargeted, None);

    assert_eq!(Edge_Count(&store, "D-900", "D-130"), 1, "the added edge is missing");
    assert_eq!(Edge_Count(&store, "D-130", "D-900"), 1, "its inverse is missing");
    assert_eq!(Edge_Count(&store, "D-900", "D-129"), 0, "the removed edge is still there");
    assert_eq!(
        Edge_Count(&store, "D-129", "D-900"),
        0,
        "half the removed fact stayed behind"
    );
}

//! What this module promises, exercised.

use super::*;
use super::labels::{Label_Of, Order};

fn Revision(label: &str, documents: &[(&str, &str)]) -> RevisionFingerprint
{
    return RevisionFingerprint {
        label: label.to_owned(),
        documents: documents
            .iter()
            .map(|(path, hash)| return ((*path).to_owned(), (*hash).to_owned()))
            .collect(),
    };
}

/// The set no pairwise diff can produce. A path gone and then back is not an addition,
/// and calling it one is how an accidental restoration reads as ordinary authoring.
#[test]
fn Test_A_Path_That_Comes_Back_Should_Be_Reappeared_Not_Appeared()
{
    let walk = Walk(&[
        Revision("v14.1", &[("a.md", "sha256:01")]),
        Revision("v14.2", &[]),
        Revision("v14.3", &[("a.md", "sha256:01")]),
    ]);

    assert_eq!(walk.len(), 2);
    assert_eq!(walk.first().map(|pair| pair.disappeared.clone()), Some(vec!["a.md".to_owned()]));
    assert_eq!(walk.get(1).map(|pair| pair.reappeared.clone()), Some(vec!["a.md".to_owned()]));
    assert_eq!(walk.get(1).map(|pair| pair.appeared.clone()), Some(Vec::new()));
}

#[test]
fn Test_A_Path_Seen_For_The_First_Time_Should_Be_Appeared()
{
    let walk = Walk(&[Revision("v14.1", &[]), Revision("v14.2", &[("a.md", "sha256:01")])]);

    assert_eq!(walk.first().map(|pair| pair.appeared.clone()), Some(vec!["a.md".to_owned()]));
    assert_eq!(walk.first().map(|pair| pair.reappeared.clone()), Some(Vec::new()));
}

#[test]
fn Test_A_Changed_Hash_Should_Be_Changed_In_Place()
{
    let walk = Walk(&[
        Revision("v14.1", &[("a.md", "sha256:01")]),
        Revision("v14.2", &[("a.md", "sha256:02")]),
    ]);

    assert_eq!(walk.first().map(|pair| pair.changed.clone()), Some(vec!["a.md".to_owned()]));
    assert_eq!(walk.first().map(|pair| pair.appeared.clone()), Some(Vec::new()));
    assert_eq!(walk.first().map(|pair| pair.disappeared.clone()), Some(Vec::new()));
}

#[test]
fn Test_An_Unchanged_Path_Should_Be_In_No_Set()
{
    let walk = Walk(&[
        Revision("v14.1", &[("a.md", "sha256:01")]),
        Revision("v14.2", &[("a.md", "sha256:01")]),
    ]);
    let Some(pair) = walk.first()
    else
    {
        // Two revisions must walk to exactly one pair. With none, the four emptiness
        // assertions below would all hold over nothing and the test would pass while
        // reporting that an unchanged path is in no set — which it never checked.
        panic!("no pair");
    };

    assert!(pair.appeared.is_empty() && pair.disappeared.is_empty());
    assert!(pair.changed.is_empty() && pair.reappeared.is_empty());
    assert_eq!(pair.Summary(), "v14.1 -> v14.2");
}

#[test]
fn Test_A_Skipped_Revision_Number_Should_Be_Named()
{
    let labels = ["v14.25", "v14.27", "v14.28"].map(str::to_owned).to_vec();

    assert_eq!(Gaps(&labels), vec!["v14.26".to_owned()]);
}

/// A major-version step is not a gap: v15.0 does not skip v14.37.
#[test]
fn Test_A_Major_Step_Should_Not_Be_Reported_As_A_Gap()
{
    let labels = ["v14.36", "v15.0"].map(str::to_owned).to_vec();

    assert!(Gaps(&labels).is_empty());
}

#[test]
fn Test_Only_Full_Suite_Archives_Should_Be_Revisions()
{
    assert_eq!(
        Label_Of("nomos-spec-internal-artifacts-v14.36.zip"),
        Some("v14.36".to_owned())
    );
    assert_eq!(Label_Of("nomos-spec-v15.0.zip"), Some("v15.0".to_owned()));
    assert_eq!(Label_Of("nomos_v14_31_ocaml_semantic_kernel.zip"), None);
    assert_eq!(Label_Of("xvpe-spec-seed-v0.1.zip"), None);
    assert_eq!(Label_Of("nomos full game plan.txt"), None);
}

#[test]
fn Test_Versions_Should_Order_Numerically_Rather_Than_As_Text()
{
    assert!(Order("v14.9") < Order("v14.10"));
    assert!(Order("v14.36") < Order("v15.0"));
}

#[test]
fn Test_The_Archive_Directory_Should_Be_Stripped_From_A_Path()
{
    assert_eq!(
        Within("nomos-spec-internal-artifacts-v14.36/01_authoring/a.md"),
        "01_authoring/a.md"
    );
}

//! P3: the same store renders the same bytes, whoever built it and wherever they built it.
//!
//! Four things could reach the output and must not — a second build, a bundle round trip,
//! the order the store was written in, and the machine holding it. Each is asserted over the
//! whole catalogue rather than over one profile, because a renderer that leaks one of them
//! leaks it in exactly one format.

use crate::store::{For_Building, Order, Populated, Populated_In_Order, Shipped};
use nomos_spec_bundle::{Bundle, Export, Import_Bundle};
use nomos_spec_project::{Build, Profile, GENERATED_FILE_NOTICE};
use nomos_spec_store::SpecificationStore;

/// How much of `GENERATED_FILE_NOTICE` a body must carry for the notice to count as explained.
///
/// A prefix rather than the whole sentence: what is checked is that the marker arrives with the
/// words that explain it, and a body is free to wrap the notice across lines.
const GENERATED_NOTICE_PREFIX_CHARS: usize = 40;

/// P3, first half.
#[test]
fn Test_A_Rebuild_Should_Be_Byte_Identical()
{
    let store = Populated();

    for profile in Shipped().Profiles()
    {
        let buildable = For_Building(profile);
        let first = Build(&store, &buildable)
            .expect("the fixture renders every shipped profile it declares");
        let second = Build(&store, &buildable)
            .expect("the same fixture and profile produced the first body");

        assert_eq!(first.body, second.body, "{} does not rebuild to itself", profile.id);
        assert_eq!(
            first.Sidecar().expect("every built output carries a rendered stamp"),
            second.Sidecar().expect("every built output carries a rendered stamp")
        );
    }
}

/// P3, second half.
#[test]
fn Test_Building_From_A_Fresh_Import_Should_Equal_Building_From_The_Original()
{
    let source = Populated();
    let rebuilt = Reimport_Through_A_Bundle(&source);

    for profile in Shipped().Profiles()
    {
        let buildable = For_Building(profile);
        let original = Build(&source, &buildable).expect("builds from the original");
        let imported = Build(&rebuilt, &buildable).expect("builds from the import");

        assert_eq!(
            original.body, imported.body,
            "{} renders differently after a bundle round trip",
            profile.id
        );
        assert_eq!(original.stamp, imported.stamp, "{}", profile.id);
    }
}

/// The fixture's store, exported and imported back into a fresh in-memory one.
///
/// This is the store a second machine holds, which is the whole point of the comparison: the
/// import has no uid from the original's insertion order to inherit.
fn Reimport_Through_A_Bundle(source: &SpecificationStore) -> SpecificationStore
{
    let bundle = Export(source)
        .expect("the fixture store is exportable as a bundle")
        .Write()
        .expect("the export writes into a buffer it owns");
    let mut rebuilt = SpecificationStore::In_Memory()
        .expect("In_Memory applies the schema MIGRATIONS before it returns");
    Import_Bundle(
        &mut rebuilt,
        &Bundle::Parse(&bundle).expect("the bytes came out of Export in the line above"),
    )
    .expect("the bundle carries every table the fixture wrote");

    return rebuilt;
}

/// Insertion order decides `uid`, and nothing in a projection may be ordered by one.
#[test]
fn Test_Insertion_Order_Should_Not_Reach_The_Output()
{
    let forwards = Populated_In_Order(Order::Forwards);
    let backwards = Populated_In_Order(Order::Backwards);

    for profile in Shipped().Profiles()
    {
        let buildable = For_Building(profile);
        let first = Build(&forwards, &buildable)
            .expect("the fixture renders every shipped profile it declares");
        let second = Build(&backwards, &buildable)
            .expect("the same fixture renders the profile in either insertion order");

        assert_eq!(
            first.body, second.body,
            "{} orders its output by the order the store was written in",
            profile.id
        );
    }
}

#[test]
fn Test_Every_Body_Should_Be_Lf_Utf8_Without_A_Bom()
{
    let store = Populated();

    for profile in Shipped().Profiles()
    {
        let output = Build(&store, &For_Building(profile))
            .expect("the fixture renders every shipped profile it declares");

        assert!(!output.body.contains('\r'), "{} carries a carriage return", profile.id);
        assert!(
            !output.body.starts_with('\u{feff}'),
            "{} starts with a byte order mark",
            profile.id
        );
        assert!(
            !output
                .Sidecar()
                .expect("every built output carries a rendered stamp")
                .contains('\r'),
            "{}: the sidecar carries a carriage return",
            profile.id
        );
    }
}

#[test]
fn Test_Every_Body_Should_Declare_Itself_Generated()
{
    let store = Populated();

    for profile in Shipped().Profiles()
    {
        let output = Build(&store, &For_Building(profile))
            .expect("the fixture renders every shipped profile it declares");

        Assert_Declares_Itself_Generated(&output.body, profile);
    }
}

/// A body has to say it is generated, say it must not be edited, and carry the sentence that
/// explains the marker rather than the marker on its own.
///
/// The profile arrives whole rather than as its `id`, so the two arguments are of different
/// types and a caller cannot pass the body where the profile goes.
fn Assert_Declares_Itself_Generated(body: &str, profile: &Profile)
{
    let id = &profile.id;

    assert!(
        body.contains("nomos_generated") || body.contains("nomos-generated"),
        "{id} does not say it is generated"
    );
    assert!(
        body.contains("do_not_edit") || body.contains("do-not-edit"),
        "{id} does not say it must not be edited"
    );
    assert!(
        body.contains(
            GENERATED_FILE_NOTICE
                .get(..GENERATED_NOTICE_PREFIX_CHARS)
                .unwrap_or(GENERATED_FILE_NOTICE)
        ) || body.contains("Generated by nomos"),
        "{id} carries the marker without the sentence that explains it"
    );
}

/// The environment-specific strings a generated body must never carry.
fn Environmental_Strings() -> Vec<String>
{
    let directory = std::env::current_dir().expect("has a working directory");
    let temporary = std::env::temp_dir();
    let host = std::env::var("COMPUTERNAME")
        .or_else(|_| return std::env::var("HOSTNAME"))
        .unwrap_or_default();

    return vec![directory.display().to_string(), temporary.display().to_string(), host];
}

/// The done-when, checked against the environment this build actually runs in rather than
/// against a pattern that guesses what one looks like.
#[test]
fn Test_No_Body_Should_Carry_This_Machine()
{
    let store = Populated();

    for profile in Shipped().Profiles()
    {
        let output = Build(&store, &For_Building(profile))
            .expect("the fixture renders every shipped profile it declares");

        for environmental in Environmental_Strings()
        {
            if environmental.trim().is_empty()
            {
                continue;
            }
            assert!(
                !output.body.contains(&environmental),
                "{} carries {environmental}, so the output depends on the machine that built it",
                profile.id
            );
        }

        assert!(
            !output.body.contains("://") && !output.body.contains(":\\"),
            "{} carries an absolute location",
            profile.id
        );
    }
}

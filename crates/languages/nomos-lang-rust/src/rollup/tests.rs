//! What this module promises, exercised.

use super::*;
use nomos_capability::Registry;
use nomos_capability::{Requirement, Resolution, Unmet};
use nomos_model::Content_Digest;

fn Subject(path: &str) -> SubjectId
{
    return SubjectId::From_Digest(Content_Digest(path.as_bytes()));
}

fn An_Index() -> ModuleIndex
{
    return ModuleIndex {
        module: Subject("the/module"),
        members: vec![
            MemberReading {
                subject: Subject("alpha.rs"),
                outcome: Outcome::Read,
            },
            MemberReading {
                subject: Subject("beta.rs"),
                outcome: Outcome::Approximate,
            },
            MemberReading {
                subject: Subject("gamma.rs"),
                outcome: Outcome::Unreachable,
            },
        ],
        items: vec![IndexEntry {
            member: Subject("alpha.rs"),
            ordinal: 0,
            kind: "Function".to_owned(),
            visibility: "Public".to_owned(),
            qualified_name: "inner::Deep".to_owned(),
        }],
    };
}

#[test]
fn Test_An_Index_Should_Survive_A_Round_Trip()
{
    let index = An_Index();

    assert_eq!(Parse_Index(&Encode_Index(&index)), Ok(index));
}

/// The encoding is the fact's content address, so it must not vary with anything but
/// the index — and it must be readable by a person, because two rollups that disagree
/// are compared in a diff before they are compared by a tool.
#[test]
fn Test_The_Encoding_Should_Be_Line_Oriented_And_Local_To_Nothing()
{
    let rendered = String::from_utf8(Encode_Index(&An_Index()))
        .expect("the encoding is ASCII tabs around hexadecimal and UTF-8 identifiers");

    assert!(!rendered.contains('\r'), "line endings must not be local");
    assert!(rendered.starts_with("module\t"));
    assert_eq!(rendered.lines().count(), 5);
}

/// A member that could not be read must not encode like a member that declares
/// nothing.
///
/// An index over one module and one member, differing only in what became of it.
///
/// The three outcomes are the point of these tests: each pair is the same member count
/// and a different answer, and an encoding that collapsed any two of them would let a
/// run report a clean module it never read.
fn One_Member(outcome: Outcome) -> ModuleIndex
{
    return ModuleIndex {
        module: Subject("the/module"),
        members: vec![MemberReading {
            subject: Subject("alpha.rs"),
            outcome,
        }],
        items: Vec::new(),
    };
}

/// The two are the same number of items and different answers, and collapsing them is
/// how a run comes to report a clean module it never read.
#[test]
fn Test_An_Unreachable_Member_Should_Not_Encode_Like_A_Silent_One()
{
    let unreachable = One_Member(Outcome::Unreachable);
    let silent = One_Member(Outcome::Read);

    assert_ne!(Encode_Index(&unreachable), Encode_Index(&silent));
}

/// An approximated rollup must not encode like an exact one.
///
/// `OD-CAPABILITY-003`'s third condition. If the distinction did not reach the bytes,
/// buying coverage from a weaker provider would also buy the appearance of precision
/// and nothing downstream could tell the two rollups apart.
#[test]
fn Test_An_Approximated_Member_Should_Not_Encode_Like_An_Exact_One()
{
    let exact = One_Member(Outcome::Read);
    let approximated = One_Member(Outcome::Approximate);

    assert_ne!(Encode_Index(&exact), Encode_Index(&approximated));
}

/// The negative controls for the reader.
///
/// Every one of these would decode to an empty or partial index under a reader that
/// fell back to a default, and an empty index reads as a module that declares
/// nothing — indistinguishable from a module that genuinely does.
#[test]
fn Test_Bytes_That_Are_Not_An_Index_Should_Be_Refused()
{
    assert!(
        Parse_Index(b"").is_err(),
        "the empty payload is not a module with no members"
    );
    assert!(
        Parse_Index(b"member\t00000000000000000000000000000000\tread\n").is_err(),
        "a payload whose first record is not `module` names no subject"
    );
    assert!(
        Parse_Index(b"module\tnot-a-digest\n").is_err(),
        "a subject that is not a digest is not a subject"
    );
    assert!(
        Parse_Index(b"module\t0000000000000000000000000000000g\n").is_err(),
        "the right length and the wrong alphabet is still not a digest"
    );

    let module = format!("module\t{}\n", Subject("the/module").Digest());
    assert!(
        Parse_Index(format!("{module}{module}").as_bytes()).is_err(),
        "a second `module` record would leave two answers to which module this is"
    );
    assert!(
        Parse_Index(format!("{module}surface\t1\n").as_bytes()).is_err(),
        "an unknown record tag is most likely a newer schema, which is exactly the \
         case where guessing loses the information that was added"
    );
    assert!(
        Parse_Index(
            format!("{module}member\t{}\tmaybe\n", Subject("alpha.rs").Digest()).as_bytes()
        )
        .is_err(),
        "an outcome this build does not know is not `read`"
    );
    assert!(
        Parse_Index(
            format!("{module}item\t{}\t0\tFunction\n", Subject("alpha.rs").Digest()).as_bytes()
        )
        .is_err(),
        "an item record missing its name is not an item with no name"
    );
}

/// A record longer than the grammar is refused, not read down to what this build knows.
///
/// The half `P10-INDEX-FIELD-STRICTNESS` was written for. Every payload here is what a
/// v2 of this schema would plausibly look like from a v1 reader: the fields it knows,
/// followed by one it does not. Truncating instead of refusing hands a caller a clean
/// decode of a payload it only partly understood, and the field it dropped is the one
/// that changed.
#[test]
fn Test_A_Record_With_A_Field_This_Build_Does_Not_Know_Should_Be_Refused()
{
    let module = format!("module\t{}\n", Subject("the/module").Digest());
    let alpha = Subject("alpha.rs").Digest();

    assert!(
        Parse_Index(format!("module\t{alpha}\tv2\n").as_bytes()).is_err(),
        "a `module` record with a field after the subject is not this schema"
    );
    assert!(
        Parse_Index(format!("{module}member\t{alpha}\tread\t42\n").as_bytes()).is_err(),
        "a `member` record carrying something after its outcome is not this schema"
    );
    assert!(
        Parse_Index(
            format!("{module}item\t{alpha}\t0\tFunction\tPublic\tOne\tv2\n").as_bytes()
        )
        .is_err(),
        "an `item` record carrying an eighth field is not this schema"
    );
}

/// The control for the test above, and the one that stops it from being satisfied by a
/// reader that refuses everything.
///
/// A stricter reader that also refused this provider's own output would be a schema with
/// no conforming writer, which is a worse defect than the laxness it replaced — and it
/// would fail here rather than in whatever consumes a rollup three commits from now.
#[test]
fn Test_Every_Record_This_Encoder_Writes_Should_Still_Be_Accepted()
{
    let index = An_Index();
    let encoded = Encode_Index(&index);
    let rendered = String::from_utf8(encoded.clone()).expect("the encoding is text");

    assert_eq!(Parse_Index(&encoded), Ok(index));

    // Every record form the encoder can emit is present above, so the round trip is a
    // statement about the grammar rather than about one record. A test that round-tripped
    // a payload with no `item` record would say nothing about the longest record there is.
    assert!(rendered.contains("\nmember\t"), "{rendered}");
    assert!(rendered.contains("\nitem\t"), "{rendered}");
    for outcome in [READ, APPROXIMATE, UNREACHABLE]
    {
        assert!(rendered.contains(outcome), "{outcome} is not exercised: {rendered}");
    }
}

/// The ceiling admits a weaker offer, which is what a ceiling is for.
#[test]
fn Test_The_Ceiling_Should_Admit_This_Providers_Offer()
{
    let mut registry = Registry::New();
    registry.Declare(Capability_Contract()).expect("declared once");

    assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
}

/// And refuses one above it. The property the contract exists for, asserted where the
/// ceiling is written.
#[test]
fn Test_The_Ceiling_Should_Refuse_A_Claim_Of_Resolution()
{
    let mut registry = Registry::New();
    registry.Declare(Capability_Contract()).expect("declared once");

    let resolved = ProviderOffer {
        guarantee: Guarantee::New(
            FactVariant::SemanticallyResolved,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::File,
        ),
        ..Provider_Offer()
    };

    assert!(
        registry.Offer(resolved).is_err(),
        "an index of what files declare is a statement about what they say on their \
         face, and a provider claiming resolution would satisfy every rule that needs it"
    );
}

/// The ceiling is not this provider's own claim.
///
/// A ceiling equal to the incumbent's guarantee has to be raised whenever somebody
/// improves something, and silently forbids a better second provider in the meantime.
#[test]
fn Test_The_Ceiling_Should_Leave_Room_Above_This_Provider()
{
    assert_ne!(Ceiling(), Declared_Guarantee());

    let mut registry = Registry::New();
    registry.Declare(Capability_Contract()).expect("declared once");
    registry.Offer(Provider_Offer()).expect("within the ceiling");

    let finer_refresh = Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );
    let needs_a_finer_refresh = Requirement::New(Capability(), CONTRACT_VERSION, finer_refresh);

    assert!(
        matches!(
            registry.Resolve(&needs_a_finer_refresh),
            Resolution::Unsatisfied {
                reason: Unmet::BelowRequirement { .. },
                ..
            }
        ),
        "a caller that must refresh one file at a time cannot be served by a rollup \
         that can only refresh a whole module, and the contract admits the provider \
         that could be"
    );
}

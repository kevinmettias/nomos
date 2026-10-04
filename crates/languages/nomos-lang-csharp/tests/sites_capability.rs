//! The C# sites offer through the public API a consumer has: every C# file declines the labeled jump
//! with the corpus's reason, and the offer stands under the family's contract.

use nomos_cap_syntax::{KindStance, LABELED_JUMP, Parse_Sites_Payload};
use nomos_contracts::{BuildVariantId, ConfigurationId, Digest128, GenerationId, SnapshotId, SubjectId};
use nomos_lang_csharp::sites::{Declared_Guarantee, LABELED_JUMP_DECLINE, Materialize_Sites_Fact, Provider_Offer};
use nomos_lang_csharp::{FactContext, Materialization};

/// The byte this fixture's snapshot identity seeds, unique among the three.
const SNAPSHOT_SEED: u8 = 1;
/// The byte this fixture's build variant identity seeds.
const BUILD_VARIANT_SEED: u8 = 2;
/// The byte this fixture's configuration identity seeds.
const CONFIGURATION_SEED: u8 = 3;

fn Context() -> FactContext
{
    return FactContext {
        snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([SNAPSHOT_SEED; Digest128::BYTE_LENGTH])),
        variant: BuildVariantId::From_Digest(Digest128::From_Bytes([BUILD_VARIANT_SEED; Digest128::BYTE_LENGTH])),
        configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([CONFIGURATION_SEED; Digest128::BYTE_LENGTH])),
        generation: GenerationId::INITIAL,
    };
}

/// The payload the provider writes for `source`, decoded through the one reader every consumer uses.
fn Payload_For(source: &str) -> nomos_cap_syntax::SitesPayload
{
    let Materialization::Materialized(fact) = Materialize_Sites_Fact(SubjectId::From_Digest(Digest128::From_Bytes([0; Digest128::BYTE_LENGTH])), source, Context())
    else
    {
        panic!("a decline does not wait on a parse, so every C# file materializes");
    };
    assert_eq!(fact.payload.schema, nomos_cap_syntax::Sites_Payload_Schema());
    return Parse_Sites_Payload(&fact.payload.bytes).expect("this provider writes the family's schema");
}

/// Every C# file -- one with loops and a `goto` label, and one that does not parse -- declines the
/// labeled jump with the corpus's reason and records nothing: what the declared guarantee claims.
#[test]
fn Test_Every_Csharp_File_Should_Decline_The_Labeled_Jump_With_Its_Reason()
{
    let loops = "class C { void F(int[] xs) { foreach (var x in xs) { if (x == 0) { break; } } retry: goto retry; } }\n";
    let broken = "class C { void F( {\n";

    for source in [loops, broken]
    {
        let payload = Payload_For(source);

        assert_eq!(payload.Stance(LABELED_JUMP.name), KindStance::Declined(LABELED_JUMP_DECLINE), "{source}");
        assert!(payload.offered.is_empty() && payload.records.is_empty(), "{payload:?}");
    }
}

/// The reason carries the substance of the corpus kernel's: no loop argument, and goto's label
/// names a point rather than a loop.
#[test]
fn Test_The_Decline_Should_State_Why_The_Construct_Does_Not_Exist_In_Csharp()
{
    for substance in ["cannot name a loop", "take no loop argument", "goto", "a point to jump to rather than a loop to leave"]
    {
        assert!(LABELED_JUMP_DECLINE.contains(substance), "the decline must say `{substance}`: {LABELED_JUMP_DECLINE}");
    }
}

#[test]
fn Test_The_Sites_Offer_Should_Be_Accepted_Under_The_Familys_Contract()
{
    let mut registry = nomos_capability::Registry::New();

    let accepted = registry.Declare_And_Offer(nomos_cap_syntax::Sites_Capability_Contract(), Provider_Offer());

    assert_eq!(accepted, Ok(()));
    assert!(nomos_cap_syntax::Sites_Ceiling().Satisfies(&Declared_Guarantee()));
}

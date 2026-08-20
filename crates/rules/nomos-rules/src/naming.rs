//! A function's own name must match this workspace's `Pascal_Snake_Case` convention.
//!
//! `README.md`'s Conventions section states the rule directly: "Function names are
//! `Pascal_Snake_Case`... Types stay `UpperCamelCase`." `Cargo.toml`'s
//! `[workspace.lints.rust]` names the resulting gap in its own comment — the one rustc
//! lint that would have caught a deviation from Rust's *own* convention, `non_snake_case`,
//! is turned off "because... this is the workspace convention, not an oversight" — and
//! nothing was turned on to check the convention that replaced it. This rule is that check.
//!
//! # A second rule, deliberately different in shape from the first
//!
//! [`crate::Check_Completeness_Mirrors`] reconciles a claim embedded in one place (a doc
//! comment) against a fact derived from another (whether a name it claims exists). This
//! rule is a one-sided lexical predicate over each function's own name, stated once in
//! prose rather than declared per-subject — no claimed mirror, no cross-file resolution,
//! no [`crate::Reading::Unobserved`] handling, because nothing here depends on
//! documentation being observed at all. `OD-PACKAGE-008` asked whether a `RulePackage`
//! manifest's field boundaries generalize past a population of one rule; this is the
//! second data point that question needs.
//!
//! # Why this has no `CONTRACT_RECORD`
//!
//! Unlike [`crate::Check_Completeness_Mirrors`], whose contract is `D-134` — a versioned
//! governing record `tests/contract/tests/rule_contract_citation.rs` checks against — this
//! rule's contract is `README.md`'s Conventions section: prose, not a record with a
//! `version:` front-matter field that same mechanism could read. `PKG-014`'s traceability
//! requirement is accordingly not mechanically checked for this rule, the same way it was
//! not checked for the first one before that citation existed. Left named rather than
//! manufactured — a record authored only to give this rule something to cite would be the
//! record standing in for the check, not the other way around.

use crate::{SourceFile, Syntax_Requirement};
use nomos_analysis::{FactReader, InputDigest, MaterializedFact};
use nomos_cap_syntax::{PayloadItem, SyntaxPayload, FUNCTION, IMPLEMENTATION, TRAIT};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};
use nomos_model::Content_Digest;

/// This rule's own identifier.
pub const NAMING_CONVENTION: &str = "function-naming-convention";

/// The one literal exemption: every binary's entry point is spelled `main`, fixed by the
/// language rather than by this workspace's naming choice.
const MAIN: &str = "main";

/// Judges every function `sources` declares against the workspace's naming convention.
///
/// One fact per file, the same shape [`crate::Check_Completeness_Mirrors`] reads — this
/// rule states the same [`Syntax_Requirement`] floor for the same reason: a name spelled
/// inside a comment or a string literal must not resolve as a real declaration, and
/// nothing weaker than a sound parse can promise that.
#[must_use]
pub fn Check_Naming_Convention(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        match Payload_Of(source, facts)
        {
            Ok(payload) =>
            {
                let violations = Violations_In(&payload, &source.path);
                findings.extend(violations);
            }
            Err(finding) => findings.push(finding),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// One file's decoded syntax payload, or a finding reporting why it could not be read.
///
/// A subject whose fact could not be read must not render as a subject that declared
/// nothing — the same principle [`crate::Check_Completeness_Mirrors`] holds for the same
/// reason, restated here because this rule keeps its own, simpler index rather than
/// sharing the mirror rule's.
fn Payload_Of(source: &SourceFile, facts: &mut dyn FactReader) -> Result<SyntaxPayload, Finding>
{
    let fact = Require_Fact(source, facts)?;
    Check_Schema(source, fact)?;
    return Parse_Fact(source, fact);
}

/// Requires this file's syntax fact, turning an inadmissible answer into an [`Unread`]
/// finding.
fn Require_Fact<'a>(source: &SourceFile, facts: &'a mut dyn FactReader) -> Result<&'a MaterializedFact, Finding>
{
    let need = Syntax_Requirement();
    let capability = nomos_cap_syntax::Capability();
    let inputs = InputDigest::Of(&[source.text.as_bytes()]);

    return match facts.Require(&capability, &source.subject, inputs, &need)
    {
        Ok(fact) => Ok(fact),
        Err(applicability) => Err(Unread(
            source,
            applicability,
            &format!("no admitted provider answered for it ({})", applicability.Label()),
        )),
    };
}

/// Confirms `fact`'s payload schema is the one this rule knows how to decode.
fn Check_Schema(source: &SourceFile, fact: &MaterializedFact) -> Result<(), Finding>
{
    if fact.payload.schema != nomos_cap_syntax::Payload_Schema()
    {
        return Err(Unread(
            source,
            Applicability::Unparseable,
            &format!(
                "the fact for this file carries payload schema `{}`, which this build does \
                 not read",
                fact.payload.schema
            ),
        ));
    }

    return Ok(());
}

/// Decodes `fact`'s payload bytes into this rule's own [`SyntaxPayload`] shape.
fn Parse_Fact(source: &SourceFile, fact: &MaterializedFact) -> Result<SyntaxPayload, Finding>
{
    return nomos_cap_syntax::Parse_Payload(&fact.payload.bytes)
        .map_err(|refusal| return Unread(source, Applicability::Unparseable, &refusal.Describe()));
}

/// A finding for a subject whose fact this rule could not read.
fn Unread(source: &SourceFile, applicability: Applicability, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(NAMING_CONVENTION),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("this file's naming could not be judged: {because}"),
        locations: vec![source.path.clone()],
    };
}

/// Every function `payload` declares that does not conform, as findings.
///
/// A pure function of an already-decoded payload, so the naming judgment itself is
/// testable against hand-written fixture text the way [`crate::facts::Check_Names_In`]
/// is — no registry, no store, no reader.
fn Violations_In(payload: &SyntaxPayload, path: &str) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for (index, item) in payload.items.iter().enumerate()
    {
        if item.kind != FUNCTION || item.Own_Name() == MAIN || Is_Trait_Method(payload, index)
        {
            continue;
        }

        if !Is_Pascal_Snake_Case(item.Own_Name())
        {
            let violation = Violation(path, item);
            findings.push(violation);
        }
    }

    return findings;
}

/// Whether the item at `ordinal` is declared inside a trait implementation.
///
/// Its name is fixed by the trait it implements — often a foreign one, such as `Display`
/// or `Iterator` — and the compiler forces an exact match regardless of this workspace's
/// own convention. The same [`SyntaxPayload::Enclosing`] and shape check
/// `crate::universe::Enumeration_Universe` already uses to tell an inherent `impl` from a
/// trait one.
fn Is_Trait_Method(payload: &SyntaxPayload, ordinal: usize) -> bool
{
    let Some(owner) = payload.Enclosing(ordinal)
    else
    {
        return false;
    };

    return owner.kind == IMPLEMENTATION && owner.shape.Value() == Some(TRAIT);
}

/// Whether `name` is `Pascal_Snake_Case`: every `_`-separated segment starts with an
/// uppercase ASCII letter, or is entirely ASCII digits (`Test_CHK_003_...` is real in this
/// workspace and its numeric segment is not a casing violation).
///
/// A single leading underscore is stripped first — Rust's own convention for "intentionally
/// unused," orthogonal to this workspace's casing choice and not something `README.md`'s
/// Conventions section speaks to.
#[must_use]
fn Is_Pascal_Snake_Case(name: &str) -> bool
{
    let name = name.strip_prefix('_').unwrap_or(name);

    if name.is_empty()
    {
        return false;
    }

    return name.split('_').all(|segment| {
        if segment.is_empty()
        {
            return false;
        }

        if segment.chars().all(|character| return character.is_ascii_digit())
        {
            return true;
        }

        return segment.chars().next().is_some_and(|first| return first.is_ascii_uppercase());
    });
}

/// A finding for one function whose name does not conform.
fn Violation(path: &str, item: &PayloadItem) -> Finding
{
    let qualified = format!("{path}::{}", item.qualified_name);

    return Finding {
        rule: RuleId::New(NAMING_CONVENTION),
        subject: SubjectId::From_Digest(Content_Digest(qualified.as_bytes())),
        subject_name: item.qualified_name.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!(
            "`{}` is not Pascal_Snake_Case: README.md's Conventions section requires \
             function names to be Pascal_Snake_Case, and Cargo.toml disables rustc's own \
             non_snake_case lint specifically because this workspace uses a different \
             convention — nothing else was checking it.",
            item.Own_Name()
        ),
        locations: vec![path.to_owned()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    mod casing
    {
        use super::*;

        #[test]
        fn Test_A_Single_Word_Starting_Uppercase_Should_Conform()
        {
            assert!(Is_Pascal_Snake_Case("New"));
        }

        #[test]
        fn Test_Two_Segments_Both_Uppercase_Should_Conform()
        {
            assert!(Is_Pascal_Snake_Case("As_Str"));
        }

        #[test]
        fn Test_A_Numeric_Segment_Should_Conform()
        {
            assert!(Is_Pascal_Snake_Case("Test_CHK_003_Something_Should_Hold"));
        }

        #[test]
        fn Test_Ordinary_Lower_Snake_Case_Should_Not_Conform()
        {
            assert!(!Is_Pascal_Snake_Case("as_str"));
        }

        #[test]
        fn Test_A_Lowercase_Second_Segment_Should_Not_Conform()
        {
            assert!(!Is_Pascal_Snake_Case("Foo_bar"));
        }

        #[test]
        fn Test_A_Single_Leading_Underscore_Should_Be_Stripped_Before_Judging()
        {
            assert!(Is_Pascal_Snake_Case("_Unused"));
            assert!(!Is_Pascal_Snake_Case("_unused"));
        }

        #[test]
        fn Test_A_Double_Underscore_Should_Not_Conform()
        {
            assert!(!Is_Pascal_Snake_Case("Foo__Bar"));
        }

        #[test]
        fn Test_An_Empty_Name_Should_Not_Conform()
        {
            assert!(!Is_Pascal_Snake_Case(""));
        }
    }

    mod scanning
    {
        use super::*;

        fn Payload(text: &str) -> SyntaxPayload
        {
            return nomos_cap_syntax::Parse_Payload(text.as_bytes())
                .expect("this fixture payload is well formed");
        }

        #[test]
        fn Test_A_Conforming_Function_Should_Produce_No_Finding()
        {
            let payload = Payload(
                "unexpanded\t0\n\
                 item\t0\tFunction\tPublic\tGood_Name\t.\t+fn/0\n",
            );

            let findings = Violations_In(&payload, "src/lib.rs");

            assert!(findings.is_empty(), "{findings:?}");
        }

        #[test]
        fn Test_A_Non_Conforming_Function_Should_Produce_One_Finding()
        {
            let payload = Payload(
                "unexpanded\t0\n\
                 item\t0\tFunction\tPublic\tbad_name\t.\t+fn/0\n",
            );

            let findings = Violations_In(&payload, "src/lib.rs");

            assert_eq!(findings.len(), 1, "{findings:?}");
            let found = findings.first().expect("asserted len 1 above");
            assert_eq!(found.subject_name, "bad_name");
            assert_eq!(found.gate, GateCategory::Advisory);
        }

        #[test]
        fn Test_Main_Should_Be_Exempt()
        {
            let payload = Payload(
                "unexpanded\t0\n\
                 item\t0\tFunction\tPrivate\tmain\t.\t+fn/0\n",
            );

            let findings = Violations_In(&payload, "src/main.rs");

            assert!(findings.is_empty(), "{findings:?}");
        }

        #[test]
        fn Test_A_Trait_Methods_Non_Conforming_Name_Should_Be_Exempt()
        {
            let payload = Payload(
                "unexpanded\t0\n\
                 item\t0\tImplementation\tNotApplicable\tDisplay\t.\t+trait\n\
                 item\t1\tFunction\tPublic\tDisplay::fmt\t.\t+fn/1\n",
            );

            let findings = Violations_In(&payload, "src/lib.rs");

            assert!(findings.is_empty(), "a trait method's fixed name was judged: {findings:?}");
        }

        #[test]
        fn Test_An_Inherent_Methods_Non_Conforming_Name_Should_Not_Be_Exempt()
        {
            let payload = Payload(
                "unexpanded\t0\n\
                 item\t0\tImplementation\tNotApplicable\tTable\t.\t+inherent\n\
                 item\t1\tFunction\tPublic\tTable::bad_name\t.\t+fn/1\n",
            );

            let findings = Violations_In(&payload, "src/lib.rs");

            assert_eq!(findings.len(), 1, "an inherent method's own name was exempted: {findings:?}");
        }

        #[test]
        fn Test_A_File_Declaring_Nothing_Should_Produce_No_Finding()
        {
            let payload = Payload("unexpanded\t0\n");

            assert!(Violations_In(&payload, "src/lib.rs").is_empty());
        }
    }

    /// [`Check_Naming_Convention`] itself, through a real registry, store and reader — the
    /// half [`Violations_In`]'s own tests do not reach, because a fact has to be required
    /// and decoded before there is a payload to judge at all.
    mod reading_a_fact
    {
        use super::*;
        use nomos_analysis::{
            Context, FactKey, FactPayload, GuaranteeDigest, MaterializedFact, MemoryFactStore, Reader,
        };
        use nomos_capability::{ProviderOffer, Registry};
        use nomos_contracts::{
            Assurance, BuildVariantId, ConfigurationId, Digest128, FactVariant, GenerationId,
            Guarantee, IncrementalGranularity, ProviderId, SnapshotId,
        };

        const PARSER: &str = "nomos.test.naming.parses";

        fn Guarantee_At_Floor() -> Guarantee
        {
            return Guarantee::New(
                FactVariant::Syntactic,
                Assurance::Sound,
                Assurance::Unknown,
                IncrementalGranularity::File,
            );
        }

        fn Source(path: &str, text: &str) -> SourceFile
        {
            return SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        }

        fn Test_Context() -> Context
        {
            return Context {
                snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; 16])),
                variant: BuildVariantId::From_Digest(Digest128::From_Bytes([2; 16])),
                configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([3; 16])),
                generation: GenerationId::INITIAL,
            };
        }

        fn Offering() -> (MemoryFactStore, Registry, ProviderOffer)
        {
            let mut registry = Registry::New();
            registry
                .Declare(nomos_cap_syntax::Capability_Contract())
                .expect("the syntax capability is declared once");

            let offer = ProviderOffer {
                provider: ProviderId::New(PARSER),
                capability: nomos_cap_syntax::Capability(),
                version: nomos_cap_syntax::CONTRACT_VERSION,
                guarantee: Guarantee_At_Floor(),
            };
            registry.Offer(offer.clone()).expect("within the ceiling");

            return (MemoryFactStore::New(), registry, offer);
        }

        fn Materialize(store: &mut MemoryFactStore, source: &SourceFile, offer: &ProviderOffer, payload: &str)
        {
            let context = Test_Context();
            let key = FactKey {
                contract: nomos_cap_syntax::Capability(),
                contract_version: offer.version,
                subject: source.subject,
                semantic_inputs: InputDigest::Of(&[source.text.as_bytes()]),
                provider: offer.provider.clone(),
                provider_version: offer.version,
                guarantee: GuaranteeDigest::Of(&offer.guarantee),
                variant: context.variant,
                configuration: context.configuration,
            };

            store
                .Materialize(
                    MaterializedFact {
                        identity: key.At(context.generation),
                        snapshot: context.snapshot,
                        evidence: EvidenceClass::Verified,
                        guarantee: offer.guarantee,
                        payload: FactPayload::New(
                            nomos_cap_syntax::Payload_Schema(),
                            payload.as_bytes().to_vec(),
                        ),
                    },
                    &[],
                )
                .expect("nothing here is backdated");
        }

        #[test]
        fn Test_A_Real_Fact_Should_Be_Read_And_Judged()
        {
            let source = Source("src/lib.rs", "fn bad_name() {}");
            let (mut store, registry, offer) = Offering();
            Materialize(
                &mut store,
                &source,
                &offer,
                "unexpanded\t0\nitem\t0\tFunction\tPublic\tbad_name\t.\t+fn/0\n",
            );

            let mut reader = Reader::On(&store, &registry, Test_Context());
            let findings = Check_Naming_Convention(&[source], &mut reader);

            assert_eq!(findings.len(), 1, "{findings:?}");
            assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "bad_name");
        }

        #[test]
        fn Test_A_Subject_With_No_Fact_Should_Be_Reported_Rather_Than_Silently_Clean()
        {
            let source = Source("src/lib.rs", "fn bad_name() {}");
            let (store, registry, _offer) = Offering();

            let mut reader = Reader::On(&store, &registry, Test_Context());
            let findings = Check_Naming_Convention(&[source], &mut reader);

            assert_eq!(findings.len(), 1, "an unread subject must not render as a clean one: {findings:?}");
            assert_eq!(
                findings.first().expect("asserted len 1 above").subject_name,
                "src/lib.rs"
            );
        }
    }
}

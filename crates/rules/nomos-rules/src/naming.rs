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
//!
//! # Split by responsibility
//!
//! [`reading`] requires and decodes one file's own syntax fact. [`violations`] judges an
//! already-decoded payload against the naming convention. This file keeps only what
//! composes the two: the rule's own identifier and [`Check_Naming_Convention`] itself, plus
//! the end-to-end tests that exercise both together through a real reader.

mod reading;
mod violations;

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_contracts::Finding;
use reading::Payload_Of;
use violations::Violations_In;

/// This rule's own identifier.
pub const NAMING_CONVENTION: &str = "function-naming-convention";

/// Judges every function `sources` declares against the workspace's naming convention.
///
/// One fact per file, the same shape [`crate::Check_Completeness_Mirrors`] reads — this
/// rule states the same [`crate::Syntax_Requirement`] floor for the same reason: a name
/// spelled inside a comment or a string literal must not resolve as a real declaration,
/// and nothing weaker than a sound parse can promise that.
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

/// [`Check_Naming_Convention`] itself, through a real registry, store and reader — the
/// half [`violations::tests`] does not reach, because a fact has to be required and
/// decoded before there is a payload to judge at all.
#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_analysis::{
        Context, FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact, MemoryFactStore, Reader,
    };
    use nomos_capability::{ProviderOffer, Registry};
    use nomos_contracts::{
        Assurance, BuildVariantId, ConfigurationId, Digest128, EvidenceClass, FactVariant, GenerationId,
        Guarantee, IncrementalGranularity, ProviderId, SnapshotId, SubjectId,
    };
    use nomos_model::Content_Digest;

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

    /// `path` and `text` are both `&str`; without a distinct type per position a call site
    /// like `Source("src/lib.rs", "fn bad_name() {}")` reads as two interchangeable
    /// strings and a swap compiles silently. These wrappers give each position a type the
    /// other cannot satisfy.
    struct Path<'a>(&'a str);
    struct Text<'a>(&'a str);

    fn Source(path: Path<'_>, text: Text<'_>) -> SourceFile
    {
        return SourceFile::New(path.0, SubjectId::From_Digest(Content_Digest(path.0.as_bytes())), text.0);
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

    /// A fresh fact store, registry, and the one [`ProviderOffer`] declared into it — named
    /// so a call site reads `offering.store`, not a position it has to count.
    struct TestOffering
    {
        store: MemoryFactStore,
        registry: Registry,
        offer: ProviderOffer,
    }

    fn Offering() -> TestOffering
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

        return TestOffering { store: MemoryFactStore::New(), registry, offer };
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
        let source = Source(Path("src/lib.rs"), Text("fn bad_name() {}"));
        let TestOffering { mut store, registry, offer } = Offering();
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
        let source = Source(Path("src/lib.rs"), Text("fn bad_name() {}"));
        let TestOffering { store, registry, .. } = Offering();

        let mut reader = Reader::On(&store, &registry, Test_Context());
        let findings = Check_Naming_Convention(&[source], &mut reader);

        assert_eq!(findings.len(), 1, "an unread subject must not render as a clean one: {findings:?}");
        assert_eq!(
            findings.first().expect("asserted len 1 above").subject_name,
            "src/lib.rs"
        );
    }
}

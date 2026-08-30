//! Every universe the run could read, indexed by the check that claims to mirror it.

use super::{BTreeSet, Unread, Shortfall, Applicability, SourceFile, FactReader, Reading, Syntax_Requirement_For, InputDigest, Check_Names_In, Read_Universes, SubjectId, Content_Digest};
use crate::DeclaredUniverse;

/// Every check name in scope for this run, and every subject that is missing from it.
///
/// The two fields are one value because they are one claim. A set of names alone cannot
/// say whether it is the whole set, and a rule that treats a short index as a complete one
/// reports a name it never looked for as a name that does not exist.
pub(super) struct CheckIndex<'source>
{
    /// Check names, from the facts that were read.
    pub(super) names: BTreeSet<String>,
    /// The universes those same facts declare, in source order.
    ///
    /// Read from the fact rather than from the file, which is what this rule stopped
    /// parsing for itself. One fact answers both halves of the subject.
    pub(super) universes: Vec<DeclaredUniverse>,
    /// Files whose provider could not observe what a universe is read from, and why.
    ///
    /// Separate from `unread`, because the fact *was* read. What is missing is a field
    /// inside it, and a file here is a file about whose mirrors nothing may be concluded.
    pub(super) unobserved: Vec<(String, String)>,
    /// Subjects whose facts were not read, in source order.
    pub(super) unread: Vec<Unread<'source>>,
}

impl CheckIndex<'_>
{
    /// Whether this index is short of anything that could have resolved `claimed`, and what
    /// to report if it is.
    ///
    /// `None` means the index is not short *for this name* — either nothing is missing from
    /// it, or what is missing could not have contained the name. Both are the same statement
    /// about the claim, which is the statement the caller needs: the rule looked wherever the
    /// name could have been and it was not there.
    ///
    /// This is the function `OD-RULES-002` replaces a whole-run flag with. The counter it
    /// has to answer is that any unread file might have held the check, and the answer is
    /// that "any" is doing the work: a file whose bytes do not spell the name held nothing
    /// named that. [`Unread::Can_Have_Declared`] states the bound and where it stops.
    pub(super) fn Shortfall_For(&self, claimed: &str) -> Option<Shortfall<'_>>
    {
        if self
            .unread
            .iter()
            .any(|subject| return subject.applicability == Applicability::MissingCapability)
        {
            return Some(Shortfall::NoIndex);
        }

        let bearing: Vec<&Unread<'_>> = self
            .unread
            .iter()
            .filter(|subject| return subject.Can_Have_Declared(claimed))
            .collect();
        let first = bearing.first()?;
        let subjects = bearing.iter().map(|subject| return subject.path.as_str()).collect();

        return Some(Shortfall::Withheld {
            applicability: first.applicability,
            subjects,
        });
    }
}

/// What one file's syntax fact declares, or why the rule could not read it.
///
/// The reading is reduced to owned values before returning, because the fact is borrowed
/// from the reader and the next subject needs the reader back.
pub(super) fn Declared_By<'source>(
    source: &'source SourceFile,
    facts: &mut dyn FactReader,
) -> Result<(BTreeSet<String>, Reading), Unread<'source>>
{
    let need = Syntax_Requirement_For(source.preferred_syntax_provider.clone());
    let capability = nomos_cap_syntax::Capability();
    let inputs = InputDigest::Of(&[source.text.as_bytes()]);

    let read = match facts.Require(&capability, &source.subject, inputs, &need)
    {
        Ok(fact) => Decoded_Syntax_Payload(fact, &source.path),
        Err(applicability) =>
        {
            let because =
                format!("no admitted provider answered for it ({})", applicability.Label());

            return Err(Unread_Of(source, applicability, because));
        }
    };

    // A payload this build cannot read is not an empty payload. Folding the two together
    // would make a fact nobody could decode indistinguishable from a file that declares no
    // checks, and the second is a real answer.
    return read.map_err(|because| return Unread_Of(source, Applicability::Unparseable, because));
}

/// What one fact's payload says, or why this build cannot read it.
///
/// Decoded once. Two decodes of one fact would be two answers to what the bytes say, which
/// is the objection `OD-SYNTAX-001` settled for the whole tree.
pub(super) fn Decoded_Syntax_Payload(
    fact: &nomos_analysis::MaterializedFact,
    path: &str,
) -> Result<(BTreeSet<String>, Reading), String>
{
    if fact.payload.schema != nomos_cap_syntax::Payload_Schema()
    {
        return Err(format!(
            "the fact for this file carries payload schema `{}`, which this build does not \
             read",
            fact.payload.schema
        ));
    }

    let payload = nomos_cap_syntax::Parse_Payload(&fact.payload.bytes)
        .map_err(|refusal| return refusal.Describe())?;

    return Ok((Check_Names_In(&payload), Read_Universes(path, &payload)));
}

/// A subject whose check names could not be read, however the reading failed.
///
/// A fact nobody answered for and a payload this build cannot decode are two causes with
/// one consequence. Spelling the record out at both sites was one edit away from the two
/// disagreeing about how an unread subject is identified.
pub(super) fn Unread_Of(
    source: &SourceFile,
    applicability: Applicability,
    because: String,
) -> Unread<'_>
{
    return Unread {
        path: source.path.clone(),
        text: &source.text,
        inputs: SubjectId::From_Digest(Content_Digest(source.text.as_bytes())),
        applicability,
        because,
    };
}

/// Files one file's reading into the index.
pub(super) fn Note_Reading<'source>(
    index: &mut CheckIndex<'source>,
    source: &'source SourceFile,
    names: BTreeSet<String>,
    reading: Reading,
)
{
    index.names.extend(names);

    match reading
    {
        Reading::Observed(found) => index.universes.extend(found),
        Reading::Unobserved { because } => index.unobserved.push((source.path.clone(), because)),
    }
}

/// Reads one syntax fact per source and collects the check names they declare.
///
/// A check is a test function by this workspace's naming convention: `fn Test_…`. It comes
/// from a fact rather than from a parse here, and the floor
/// [`crate::Syntax_Requirement`] states is what keeps that safe — a `fn Test_X` written
/// inside a fixture string or a block comment would resolve a claim that nothing actually
/// checks, which is the precise defect this rule exists to find arriving through the rule's
/// own back door. A line scanner reports those; a parser does not; the floor admits only
/// the second.
///
/// The semantic inputs are recomputed here from the text rather than carried on
/// [`SourceFile`]. If they disagreed with what the provider wrote, the lookup would miss —
/// loudly, as an unread subject — and a shared helper would make that class of mismatch
/// untestable.
pub(super) fn Check_Index_Of<'source>(
    sources: &'source [SourceFile],
    facts: &mut dyn FactReader,
) -> CheckIndex<'source>
{
    let mut index = CheckIndex {
        names: BTreeSet::new(),
        universes: Vec::new(),
        unobserved: Vec::new(),
        unread: Vec::new(),
    };

    for source in sources
    {
        match Declared_By(source, facts)
        {
            Ok((names, reading)) => Note_Reading(&mut index, source, names, reading),
            Err(unread) => index.unread.push(unread),
        }
    }

    return index;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::checks::test_support::Test_Context;
    use nomos_analysis::{FactKey, FactPayload, GuaranteeDigest};
    use nomos_contracts::{Assurance, EvidenceClass, FactVariant, Guarantee, IncrementalGranularity, ProviderId, SchemaId};

    fn Guarantee_At_Floor() -> Guarantee
    {
        return Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File);
    }

    /// A fact carrying `schema`/`bytes`, addressed the way `test_support::Materialize`
    /// files one — the identity fields are plumbing [`Decoded_Syntax_Payload`] never reads,
    /// so a fixed floor-guarantee key serves every case here.
    fn Fact_With(schema: SchemaId, bytes: Vec<u8>) -> nomos_analysis::MaterializedFact
    {
        let context = Test_Context();
        let guarantee = Guarantee_At_Floor();
        let key = FactKey {
            contract: nomos_cap_syntax::Capability(),
            contract_version: nomos_cap_syntax::CONTRACT_VERSION,
            subject: SubjectId::From_Digest(Content_Digest(b"a.rs")),
            semantic_inputs: InputDigest::Of(&[b"a.rs" as &[u8]]),
            provider: ProviderId::New("nomos.test.index.decodes"),
            provider_version: nomos_cap_syntax::CONTRACT_VERSION,
            guarantee: GuaranteeDigest::Of(&guarantee),
            variant: context.variant,
            configuration: context.configuration,
        };

        return nomos_analysis::MaterializedFact {
            identity: key.At(context.generation),
            snapshot: context.snapshot,
            evidence: EvidenceClass::Verified,
            guarantee,
            payload: FactPayload::New(schema, bytes),
        };
    }

    #[test]
    fn Test_Decoded_Syntax_Payload_Should_Refuse_An_Unrecognized_Schema()
    {
        let fact = Fact_With(SchemaId::New("nomos.syntax.items.v9"), Vec::new());

        let refused = Decoded_Syntax_Payload(&fact, "a.rs");

        assert!(refused.is_err(), "{refused:?}");
    }

    #[test]
    fn Test_Decoded_Syntax_Payload_Should_Decode_The_Agreed_Schema_Into_Names_And_Universes()
    {
        let fact = Fact_With(
            nomos_cap_syntax::Payload_Schema(),
            b"unexpanded\t0\nitem\t0\tFunction\tPrivate\ttests::Test_Something_Should_Hold\t.\t+fn/0\n".to_vec(),
        );

        let (names, reading) = Decoded_Syntax_Payload(&fact, "a.rs").expect("this payload is well formed");

        assert!(names.contains("Test_Something_Should_Hold"), "{names:?}");
        assert_eq!(reading, Reading::Observed(Vec::new()), "no universe is declared in this fixture");
    }

    #[test]
    fn Test_Unread_Of_Should_Carry_The_Sources_Own_Text_And_Digest()
    {
        let source = SourceFile::New("a.rs", SubjectId::From_Digest(Content_Digest(b"a.rs")), "fn Test_Something() {}");

        let unread = Unread_Of(&source, Applicability::DependencyUnavailable, "no admitted provider answered for it".to_owned());

        assert_eq!(unread.path, "a.rs");
        assert_eq!(unread.text, "fn Test_Something() {}");
        assert_eq!(unread.applicability, Applicability::DependencyUnavailable);
        assert_eq!(unread.because, "no admitted provider answered for it");
    }

    #[test]
    fn Test_Note_Reading_Should_Extend_The_Indexs_Names_And_Record_An_Observed_Universe()
    {
        let source = SourceFile::New("a.rs", SubjectId::From_Digest(Content_Digest(b"a.rs")), "pub const TABLES: &[&str] = &[];");
        let mut index = CheckIndex { names: BTreeSet::new(), universes: Vec::new(), unobserved: Vec::new(), unread: Vec::new() };
        let universe = crate::DeclaredUniverse {
            path: "a.rs".to_owned(),
            name: "TABLES".to_owned(),
            kind: crate::UniverseKind::Constant,
            claimed_mirror: None,
        };
        let mut names = BTreeSet::new();
        names.insert("Test_Something".to_owned());

        Note_Reading(&mut index, &source, names, Reading::Observed(vec![universe.clone()]));

        assert!(index.names.contains("Test_Something"), "{:?}", index.names);
        assert_eq!(index.universes, vec![universe]);
        assert!(index.unobserved.is_empty());
    }

    #[test]
    fn Test_Note_Reading_Should_Record_An_Unobserved_File_Without_Adding_A_Universe()
    {
        let source = SourceFile::New("b.rs", SubjectId::From_Digest(Content_Digest(b"b.rs")), "pub const TABLES: &[&str] = &[];");
        let mut index = CheckIndex { names: BTreeSet::new(), universes: Vec::new(), unobserved: Vec::new(), unread: Vec::new() };

        Note_Reading(&mut index, &source, BTreeSet::new(), Reading::Unobserved { because: "documentation was not observed".to_owned() });

        assert!(index.universes.is_empty());
        assert_eq!(index.unobserved, vec![("b.rs".to_owned(), "documentation was not observed".to_owned())]);
    }

    #[test]
    fn Test_Shortfall_For_Should_Report_No_Index_When_A_Subject_Was_Never_Read()
    {
        let index = CheckIndex {
            names: BTreeSet::new(),
            universes: Vec::new(),
            unobserved: Vec::new(),
            unread: vec![Unread {
                path: "a.rs".to_owned(),
                text: "",
                inputs: SubjectId::From_Digest(Content_Digest(b"a.rs")),
                applicability: Applicability::MissingCapability,
                because: "no admitted provider".to_owned(),
            }],
        };

        let shortfall = index.Shortfall_For("Test_Something").expect("a missing-capability subject always shortfalls");

        assert!(matches!(shortfall, Shortfall::NoIndex));
    }

    #[test]
    fn Test_Shortfall_For_Should_Name_Subjects_Whose_Text_Spells_The_Claimed_Name()
    {
        let index = CheckIndex {
            names: BTreeSet::new(),
            universes: Vec::new(),
            unobserved: Vec::new(),
            unread: vec![Unread {
                path: "b.rs".to_owned(),
                text: "fn Test_Renamed_Away() {}",
                inputs: SubjectId::From_Digest(Content_Digest(b"b.rs")),
                applicability: Applicability::DependencyUnavailable,
                because: "no admitted provider".to_owned(),
            }],
        };

        let shortfall = index.Shortfall_For("Test_Renamed_Away").expect("the unread subject's text spells the claimed name");

        match shortfall
        {
            Shortfall::Withheld { subjects, .. } => assert_eq!(subjects, vec!["b.rs"]),
            Shortfall::NoIndex => panic!("expected a Withheld shortfall naming the subject"),
        }
    }

    fn Admitted_Reader() -> (nomos_capability::Registry, nomos_analysis::MemoryFactStore, nomos_capability::ProviderOffer)
    {
        let mut registry = nomos_capability::Registry::New();
        let offer = nomos_capability::ProviderOffer {
            provider: ProviderId::New("nomos.test.index.declared_by"),
            capability: nomos_cap_syntax::Capability(),
            version: nomos_cap_syntax::CONTRACT_VERSION,
            guarantee: Guarantee_At_Floor(),
        };
        registry.Declare_And_Offer(nomos_cap_syntax::Capability_Contract(), offer.clone()).expect("declared and offered within the ceiling");

        return (registry, nomos_analysis::MemoryFactStore::New(), offer);
    }

    #[test]
    fn Test_Declared_By_Should_Read_A_Materialized_Fact_Into_Names_And_A_Reading()
    {
        let source = SourceFile::New("a.rs", SubjectId::From_Digest(Content_Digest(b"a.rs")), "fn Test_Something() {}");
        let (registry, mut store, offer) = Admitted_Reader();
        crate::checks::test_support::Materialize(
            &mut store,
            source.subject,
            &offer,
            InputDigest::Of(&[source.text.as_bytes()]),
            nomos_cap_syntax::Payload_Schema(),
            b"unexpanded\t0\nitem\t0\tFunction\tPrivate\ttests::Test_Something_Should_Hold\t.\t+fn/0\n".to_vec(),
        );
        let mut facts = nomos_analysis::Reader::On(&store, &registry, Test_Context());

        let (names, reading) = Declared_By(&source, &mut facts).expect("the fact was just materialized");

        assert!(names.contains("Test_Something_Should_Hold"), "{names:?}");
        assert_eq!(reading, Reading::Observed(Vec::new()));
    }

    #[test]
    fn Test_Declared_By_Should_Report_Unread_When_No_Fact_Answers_For_The_Source()
    {
        let source = SourceFile::New("a.rs", SubjectId::From_Digest(Content_Digest(b"a.rs")), "fn Test_Something() {}");
        let (registry, store, _offer) = Admitted_Reader();
        let mut facts = nomos_analysis::Reader::On(&store, &registry, Test_Context());

        let unread = Declared_By(&source, &mut facts).expect_err("nothing was materialized for this subject");

        assert_eq!(unread.path, "a.rs");
    }

    #[test]
    fn Test_Check_Index_Of_Should_Index_Every_Source_It_Could_Read()
    {
        let readable = SourceFile::New("a.rs", SubjectId::From_Digest(Content_Digest(b"a.rs")), "fn Test_Something() {}");
        let (registry, mut store, offer) = Admitted_Reader();
        crate::checks::test_support::Materialize(
            &mut store,
            readable.subject,
            &offer,
            InputDigest::Of(&[readable.text.as_bytes()]),
            nomos_cap_syntax::Payload_Schema(),
            b"unexpanded\t0\nitem\t0\tFunction\tPrivate\ttests::Test_Something_Should_Hold\t.\t+fn/0\n".to_vec(),
        );
        let mut facts = nomos_analysis::Reader::On(&store, &registry, Test_Context());

        let sources = [readable];
        let index = Check_Index_Of(&sources, &mut facts);

        assert!(index.names.contains("Test_Something_Should_Hold"), "{:?}", index.names);
        assert!(index.unread.is_empty());
    }
}

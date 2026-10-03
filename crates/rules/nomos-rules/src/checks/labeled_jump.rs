//! `a-labeled-jump-leaves-one-loop` -- a loop label that no jump naming it needs.
//!
//! ```text
//! 'outer: for row in grid {
//!     if row.is_empty() {
//!         break 'outer;   // == `break`
//!     }
//! }
//! ```
//!
//! Delete the label and every jump naming it lands in exactly the same place. A label promises the
//! reader that control is about to go somewhere a plain jump could not reach -- past a `switch`,
//! past a `select`, out of a loop the jump is not standing in -- and one that delivers none of that
//! is a promise the reader pays for by going to look.
//!
//! # The corpus check, reproduced the way it is made
//!
//! This is code-standards' `check-labeled-loop` and its shared engine `labeledloop` at
//! `f0d820729`, and it is made the way that check is: a provider for each language projects every
//! jump that names a loop, with the two facts only that language can state, and this rule judges
//! the projection. The projection is the labeled-jump kind of `nomos.cap.syntax.sites`, declared in
//! `nomos-cap-syntax` (`OD-CAPABILITY-019`), so this rule names no field and parses no source.
//!
//! **It does not report a jump for leaving several loops.** That was the corpus check's loudest
//! complaint once and it was wrong: crossing loop levels is the one thing a label is for, and no
//! bare jump can do it. **The unit is the label, not the jump**, grouped by the line the label is
//! written on rather than by its name, because two functions may each label a loop `L` and one
//! may need it while the other does not. A label is load-bearing if any jump naming it would be
//! caught on the way out by a bare jump's binding -- `has_same_target_unlabeled` false -- or stands
//! inside a construct that catches a bare `break`; one such jump keeps the name for all of them.
//!
//! # What a file's payload can say, and what each answer reports
//!
//! A file whose provider offers the kind is judged, and a label it reports is a finding on the
//! label's line. `PartiallySupported`, because a jump inside an unexpanded macro is invisible to a
//! parse and could be the one that needs the label: the label was judged over the jumps a parse
//! can see. A file whose provider declines the kind is `NotApplicable`, with the provider's reason;
//! a file whose payload neither offers nor declines it is `MissingCapability`, never a clean pass.
//! Only files a syntax provider recognizes are judged -- the files whose language this workspace
//! reads at all.
//!
//! The corpus check honours an inline `labeled-loop: allow` marker carrying a reason. This rule
//! does not: inline markers are `ARC-CONFORMANCE-003`'s fifth milestone, and a marker honoured by
//! one ported rule ahead of the mechanism would be a second, private suppression channel.

use crate::SourceFile;
use nomos_analysis::{FactReader, InputDigest};
use nomos_cap_syntax::{KindStance, LABELED_JUMP, LabeledJump, Parse_Sites_Payload, SITES_CONTRACT_VERSION, Sites_Capability, Sites_Payload_Schema, SitesPayload};
use nomos_capability::Requirement;
use nomos_contracts::{Applicability, Assurance, EvidenceClass, FactVariant, Finding, GateCategory, Guarantee, IncrementalGranularity, RuleId};
use std::collections::BTreeMap;

/// This rule's own identifier -- the corpus rule's, `a-labeled-jump-leaves-one-loop`.
pub const A_LABELED_JUMP_LEAVES_ONE_LOOP: &str = "a-labeled-jump-leaves-one-loop";

/// The record this implementation's contract is written against: `OD-CAPABILITY-019` made a
/// projection a kind in one family, and this is the rule over its first kind.
/// `tests/contract/tests/rule_contract_citation.rs` compares this against the record's own front
/// matter on every run.
pub const A_LABELED_JUMP_LEAVES_ONE_LOOP_CONTRACT_RECORD: &str = "OD-CAPABILITY-019";

/// The version of [`A_LABELED_JUMP_LEAVES_ONE_LOOP_CONTRACT_RECORD`] this implementation was
/// written against.
pub const A_LABELED_JUMP_LEAVES_ONE_LOOP_CONTRACT_RECORD_VERSION: u32 = 1;

/// What this rule needs of a file's sites before it believes them: `Syntactic`, sound, completeness
/// not required, per file -- narrowed toward the provider that recognized the file, the way
/// [`crate::Syntax_Requirement_For`] narrows the items family, because the family's offers
/// partition by language rather than compete over one file.
#[must_use]
pub(crate) fn Sites_Requirement_For(source: &SourceFile) -> Requirement
{
    let guarantee = Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File);
    let need = Requirement::New(Sites_Capability(), SITES_CONTRACT_VERSION, guarantee);

    return match &source.preferred_syntax_provider
    {
        Some(provider) => need.Preferring(provider.clone()),
        None => need,
    };
}

/// Whether a syntax provider reads `source`'s language -- the predicate this rule's population,
/// `crate::checks::populations::LABELED_JUMP_POPULATION`, is declared by.
pub(super) fn Is_Read_By_A_Syntax_Provider(source: &SourceFile) -> bool
{
    return source.language.is_some();
}

/// Reports every loop label no jump naming it needs, in every source a syntax provider recognizes
/// -- or, for a source whose sites say nothing a judgment can rest on, why it was not judged.
#[must_use]
pub fn Check_A_Labeled_Jump_Leaves_One_Loop(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let mut findings = Vec::new();
    for source in sources.iter().filter(|source| return crate::checks::populations::LABELED_JUMP_POPULATION.Holds(source))
    {
        findings.extend(Judge_Source(source, facts));
    }

    return findings;
}

/// One source's findings: its noise labels, or the one finding saying why it was not judged.
fn Judge_Source(source: &SourceFile, facts: &mut dyn FactReader) -> Vec<Finding>
{
    let payload = match Payload_Of(source, facts)
    {
        Ok(payload) => payload,
        Err(finding) => return vec![*finding],
    };

    return match payload.Stance(LABELED_JUMP.name)
    {
        KindStance::Offered => match LabeledJump::All_In(&payload)
        {
            Some(jumps) => Noise_Labels(&jumps).iter().map(|noise| return Noise_Finding(source, noise)).collect(),
            None => vec![Unjudged_Finding(source, Applicability::Unparseable, "a labeled-jump record in its sites could not be read")],
        },
        KindStance::Declined(reason) => vec![Unjudged_Finding(source, Applicability::NotApplicable, &format!("its language declines the labeled-jump kind: {reason}"))],
        KindStance::Unanswered =>
        {
            vec![Unjudged_Finding(source, Applicability::MissingCapability, "its provider neither offers nor declines the labeled-jump kind")]
        }
    };
}

/// A loop label no jump naming it needs.
#[derive(Debug, PartialEq, Eq)]
struct NoiseLabel<'jumps>
{
    /// The line the label is written on, where the finding lands: the defect is the label.
    line: usize,
    /// The label as the language spells it.
    label: &'jumps str,
    /// How many jumps name it, so the message can say how many spellings the reader un-learns.
    jumps: usize,
}

/// The labels no jump to them needs, one per label, in the order of the lines they are written on.
///
/// Grouped by the label's line rather than its name, because a name is not an identity.
fn Noise_Labels(jumps: &[LabeledJump]) -> Vec<NoiseLabel<'_>>
{
    let mut grouped: BTreeMap<usize, Vec<&LabeledJump>> = BTreeMap::new();
    for jump in jumps
    {
        grouped.entry(jump.label_line).or_default().push(jump);
    }

    return grouped
        .into_iter()
        .filter(|(_, group)| return !Is_Load_Bearing(group))
        .filter_map(|(line, group)| return group.first().map(|first| return NoiseLabel { line, label: first.label.as_str(), jumps: group.len() }))
        .collect();
}

/// Whether a label does work no bare jump could do: one jump that a bare jump's binding would
/// catch on the way out, or that stands inside a construct catching a bare `break`, is enough.
fn Is_Load_Bearing(jumps: &[&LabeledJump]) -> bool
{
    return jumps.iter().any(|jump| return !jump.has_same_target_unlabeled || jump.is_inside_break_capture);
}

/// What the label fails to buy and what to do about it -- the corpus message's substance.
fn Message(noise: &NoiseLabel<'_>) -> String
{
    let spellings = if noise.jumps == 1
    {
        "the one jump that names it would land".to_owned()
    }
    else
    {
        format!("the {} jumps that name it would each land", noise.jumps)
    };

    return format!(
        "the label `{}` buys nothing: {spellings} in exactly the same place with the label deleted. A label promises the reader that \
         control is going somewhere a plain jump could not reach -- past a `switch`, past a `select`, out of a loop the jump is not \
         standing in -- and this one does not deliver it. Delete the label and the names on the jumps",
        noise.label
    );
}

fn Noise_Finding(source: &SourceFile, noise: &NoiseLabel<'_>) -> Finding
{
    let location = format!("{}:{}", source.path, noise.line);
    return Finding {
        address: None,
        rule: RuleId::New(A_LABELED_JUMP_LEAVES_ONE_LOOP),
        subject: source.subject,
        subject_name: location.clone(),
        // Not `Supported`: a jump inside an unexpanded macro could be the one that needs the
        // label, so the label was judged over the jumps a parse can see.
        applicability: Applicability::PartiallySupported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: Message(noise),
        locations: vec![location],
    };
}

/// One source's decoded sites, or the finding saying why they could not be read -- boxed, because
/// a `Finding` is large and the payload is what the ordinary path carries.
fn Payload_Of(source: &SourceFile, facts: &mut dyn FactReader) -> Result<SitesPayload, Box<Finding>>
{
    let inputs = InputDigest::Of(&[source.text.as_bytes()]);
    let fact = facts.Require(&Sites_Capability(), &source.subject, inputs, &Sites_Requirement_For(source)).map_err(|applicability| {
        return Box::new(Unjudged_Finding(source, applicability, &format!("no admitted provider answered for its sites ({})", applicability.Label())));
    })?;
    if fact.payload.schema != Sites_Payload_Schema()
    {
        let because = format!("its sites carry payload schema `{}`, which this build does not read", fact.payload.schema);
        return Err(Box::new(Unjudged_Finding(source, Applicability::Unparseable, &because)));
    }

    return Parse_Sites_Payload(&fact.payload.bytes).map_err(|refusal| return Box::new(Unjudged_Finding(source, Applicability::Unparseable, &refusal.Describe())));
}

fn Unjudged_Finding(source: &SourceFile, applicability: Applicability, because: &str) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(A_LABELED_JUMP_LEAVES_ONE_LOOP),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Advisory,
        summary: format!("this source's loop labels were not judged: {because}"),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
mod tests;

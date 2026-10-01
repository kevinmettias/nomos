//! Materializing the facts a walk-driven provider answers -- one per unit it found, a file under a
//! build or a module -- through whichever provider the composition supplies, and what that
//! produced.

use nomos_analysis::{Context, MemoryFactStore};
use nomos_contracts::{EvidenceClass, Finding, RuleId};
use nomos_platform::{Environment, FileSystem, ProgramLauncher};
use nomos_rules::SourceFile;
use std::collections::BTreeMap;

use crate::composed_providers::{WalkFactsProvider, WalkReading, Unanswered};
use crate::facts::currency::Materialized_Or_Already_Current;

/// What materializing a walk-driven provider produced: one source per fact for its rule to judge,
/// and a finding for every part of the population the provider did not answer.
pub struct WalkMaterialization
{
    pub sources: Vec<SourceFile>,
    pub findings: Vec<Finding>,
}

/// Runs `provider` over `reading`, files every fact it answers into `store`, and returns one source
/// per fact -- the fact's path, its subject, the provider that answered it, and the text of the
/// walked file at that path when there is one -- with a finding under `rule` for each thing it left
/// unanswered.
///
/// The text rides along because a rule reading a per-file fact can still owe part of its verdict to
/// the file's lines: `a-discarded-error-is-explained` takes which values are errors from the fact
/// and whether a comment explains each from the text. A fact about a unit that is not one walked
/// file -- a Go module -- carries none.
///
/// The answering provider rides on the source as `answered_by` because a family a walk-driven
/// provider answers can have another offer answering other subjects -- `go vet` beside clippy for
/// lint -- and `OD-CAPABILITY-009` puts the narrowing on the rule's side, from what the source
/// carries.
///
/// # Every fact's source, filed or not
///
/// A source is returned for every fact the provider answered, including one whose write the store
/// refused. That is the opposite of the per-member families, and deliberately: their rules judge
/// each member alone, so a member with no fact is simply a member not judged. A rule judging a file
/// under several builds judges them all at once, so a build silently missing from its sources
/// would read as a file fewer builds compile -- and a branch only that build compiles would be
/// called dead. Handed the source, the rule asks for the fact, finds none, and leaves the file
/// unjudged and reported instead.
pub fn Materialize_Walk_Facts<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    reading: &WalkReading<'_, Launcher, Fs, Env>,
    store: &mut MemoryFactStore,
    provider: WalkFactsProvider<Launcher, Fs, Env>,
    rule: &'static str,
) -> WalkMaterialization
{
    let answer = provider(reading);
    let texts: BTreeMap<&str, &str> = reading.sources.iter().map(|walked| return (walked.path, walked.text)).collect();
    let sources = answer
        .facts
        .into_iter()
        .map(|fact| {
            let text = texts.get(fact.path.as_str()).map_or_else(String::new, |text| return (*text).to_owned());
            return Filed_Source(fact, text, reading.context, store);
        })
        .collect();
    let findings = answer.unanswered.into_iter().map(|unanswered| return Unanswered_Finding(rule, unanswered)).collect();

    return WalkMaterialization { sources, findings };
}

fn Filed_Source(fact: crate::composed_providers::SubjectFact, text: String, context: &Context, store: &mut MemoryFactStore) -> SourceFile
{
    let mut source = SourceFile::New(fact.path, fact.subject, text);
    source.answered_by = Some(fact.fact.Key().provider.clone());
    // Whether the write landed changes nothing here: the rule asks for the fact either way, and
    // this function's own doc says why a refused one must still be asked for.
    let _filed = Materialized_Or_Already_Current(fact.fact, context, store);
    return source;
}

/// The finding one unanswered part of the population produces, under the rule that would have
/// judged it -- with the applicability and gate the composition chose, since only it knows whether
/// the gap is a repository declaring nothing, a host missing an SDK, or a declaration in error.
fn Unanswered_Finding(rule: &'static str, unanswered: Unanswered) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(rule),
        subject: nomos_model::Subject_Of_Path(&unanswered.path),
        subject_name: unanswered.subject_name,
        applicability: unanswered.applicability,
        evidence: EvidenceClass::Derived,
        gate: unanswered.gate,
        summary: unanswered.reason,
        // A gap about the whole tree names no location, as a refused whole-workspace provider's
        // finding never has; a gap about a path names that path.
        locations: if unanswered.path.is_empty() { Vec::new() } else { vec![unanswered.path] },
    };
}

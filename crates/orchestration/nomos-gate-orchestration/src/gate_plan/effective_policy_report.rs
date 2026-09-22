//! The effective policy as the lines a host prints, so two hosts print one answer.
//!
//! `OD-POLICY-001` decision 4 decides what an effective policy carries per field: the layer
//! and artifact that decided it, every contribution that decision outranked, and every
//! override a lock refused with its reason. `crate::policy` built all of that and nothing
//! rendered it, so a repository could be told why its gate judged as it did only by reading
//! the resolver's own types.
//!
//! This is the one assembly of that answer. A host prints these lines or hands them across a
//! wire; neither composes a sentence of its own, because two assemblies of one fact are two
//! answers to one question, and the one a reader gets would then depend on which surface they
//! asked through.
//!
//! # Rejected is not overridden
//!
//! `OD-POLICY-001` amendment version 3, quoting `CONFIG-003`: "a rejected statement was
//! forbidden rather than outranked, and a report that filed it as overridden would tell its
//! author their value lost a precedence contest it was never admitted to". So the two lists
//! render under separate introductions, and the rejected one always carries the reason the
//! resolver wrote -- which names the layer and artifact that locked the field.
//!
//! # Absent is not empty
//!
//! `OD-POLICY-001`: "a layer with no source contributes nothing and is reported as absent,
//! never as an empty contribution". A layer that had no source at all is named under its own
//! heading, which says so in those words, rather than appearing among the fields as a source
//! that stated nothing.

use nomos_contracts::ConfigurationLayer;

use crate::policy::{EffectivePolicy, ResolvedField};

/// The heading the per-field lines sit under.
const FIELDS_HEADING: &str = "effective policy, field by field:";

/// The heading every layer with no source at all sits under.
///
/// Says which of the two facts it is in its own words, because "Organization" on a line by
/// itself reads equally as a layer that had no source and as one whose source declared
/// nothing, and `OD-POLICY-001` keeps those apart.
const ABSENT_HEADING: &str = "layers with no source at all, absent rather than a source that stated nothing:";

/// How the statement that decided a field is introduced.
const DECIDED_BY: &str = "decided by";

/// How a contribution the deciding statement outranked is introduced.
const OVERRODE: &str = "overrode";

/// How a statement a lock refused is introduced.
///
/// Names what it is *not* as well as what it is, so a line skimmed out of its section still
/// tells its author the difference `CONFIG-003` requires them to be told.
const REJECTED: &str = "rejected, not overridden:";

/// What separates a rejected statement's provenance from the reason it was refused.
const REASON_SEPARATOR: &str = " -- ";

/// The indent a field's own line, and a named layer, sit at.
const ENTRY_INDENT: &str = "  ";

/// The indent a field's overridden and rejected lines sit at.
const DETAIL_INDENT: &str = "    ";

/// `policy` as the ordered lines a host reports it with.
///
/// The fields come in the order the resolution carries them, which is the order a run checks
/// them in, so two reports of one policy are the same text and a difference between two runs
/// is a difference in what decided them.
#[must_use]
pub fn Effective_Policy_Report(policy: &EffectivePolicy) -> Vec<String>
{
    let mut lines = vec![FIELDS_HEADING.to_owned()];

    for resolved in &policy.fields
    {
        lines.extend(Field_Lines(resolved));
    }
    lines.extend(Absent_Lines(&policy.absent_layers));

    return lines;
}

/// One field's block: what decided it, what that outranked, and what a lock refused.
///
/// The deciding line carries [`crate::FieldProvenance::Sentence`] rather than a layer and an
/// artifact spelled out here, because that sentence is also what says a unit decided the
/// field -- "decided with 'phases' as the phase policy" -- which `OD-POLICY-001` version 2
/// requires every field of a resolved unit to report.
fn Field_Lines(resolved: &ResolvedField) -> Vec<String>
{
    let decided = format!("{ENTRY_INDENT}{}: {DECIDED_BY} {}", resolved.field.Label(), resolved.decided_by.Sentence());
    let mut lines = vec![decided];

    for outranked in &resolved.overrode
    {
        lines.push(format!("{DETAIL_INDENT}{OVERRODE} {}", outranked.Sentence()));
    }
    for refused in &resolved.rejected
    {
        lines.push(format!("{DETAIL_INDENT}{REJECTED} {}{REASON_SEPARATOR}{}", refused.offered_by.Sentence(), refused.reason));
    }

    return lines;
}

/// Every layer with no source at all, and nothing at all when every layer had one.
///
/// A heading over an empty list on a host where every layer is observable would be a section
/// that means nothing, and the one case this exists for -- a layer a reader expected to speak
/// and that had no source -- reads the same either way.
fn Absent_Lines(absent: &[ConfigurationLayer]) -> Vec<String>
{
    if absent.is_empty()
    {
        return Vec::new();
    }

    let mut lines = vec![ABSENT_HEADING.to_owned()];
    lines.extend(absent.iter().map(|layer| return format!("{ENTRY_INDENT}{}", layer.Label())));

    return lines;
}

#[cfg(test)]
mod tests;

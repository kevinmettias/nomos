//! Why a `nomos.standards.corpus.v1` payload could not be decoded.

/// A payload's bytes did not decode: not UTF-8, a line with no tag, a line carrying an
/// unrecognized tag or the wrong number of fields for the tag it carries, a value that is not
/// in the vocabulary it is written in, a path declared twice, a rule naming a document no line
/// declared as a rule, or a document declaring `kind: rule` that the payload records neither a
/// rule nor an issue for.
///
/// The last two are this type's reason for existing rather than a generic parse error. Both
/// describe a payload that would read as a complete corpus while a document had silently left
/// it — the one failure a reader over a corpus must never have.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal
{
    pub reason: String,
}

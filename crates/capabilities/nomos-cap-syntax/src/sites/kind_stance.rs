//! What a payload says about one kind.

/// A payload's stance on one kind: three answers, and only the first is a judgment a rule can
/// pass a file on.
///
/// `Offered` with no records of the kind is a clean file. `Declined` is a positive statement that
/// the construct does not exist in the file's language, which a rule reports as
/// `Applicability::NotApplicable`. `Unanswered` is a gap -- nobody said -- which a rule reports as
/// `Applicability::MissingCapability` and never as a pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KindStance<'payload>
{
    Offered,
    Declined(&'payload str),
    Unanswered,
}

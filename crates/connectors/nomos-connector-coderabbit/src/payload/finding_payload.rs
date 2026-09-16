use crate::ReviewFindingId;

/// One review comment's own observed fields, canonically named, kept apart from any
/// judgment about whether the finding is valid or what it bears on.
///
/// `category` and `severity` are kept as the vendor's own reported words (`"🔒 Security &
/// Privacy"`, `"🟡 Minor"`) rather than parsed into a canonical enum: this capability
/// agrees on the fields a finding carries, not on a fixed vocabulary for what a review
/// tool may call a severity, and a second provider using different words remains honestly
/// representable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindingPayload
{
    pub external_system: String,
    pub external_id: ReviewFindingId,
    pub locator: String,
    pub category: String,
    pub severity: String,
    pub path: String,
    pub line: String,
    pub message: String,
}

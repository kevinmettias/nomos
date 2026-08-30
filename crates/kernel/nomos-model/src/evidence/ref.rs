use serde::{Deserialize, Serialize};

/// A pointer to something that supports a claim.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ref
{
    /// What kind of thing is being pointed at — a run record, a capture, a test result,
    /// a source range.
    pub kind: String,
    /// How to find it.
    pub locator: String,
}

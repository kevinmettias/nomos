//! One corpus file that was not there, and what it cost.

/// Something the store was expected to hold and does not.
///
/// Every field is required. An absence that says only "no corpus" leaves the reader to
/// discover the variable's name, the path, and what is missing from the answer they just
/// received — and the last of those is the one they will not think to ask about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Absence
{
    /// What is missing, as a person would name it.
    pub subject: String,
    /// Where it was looked for, concretely.
    pub expected: String,
    /// Why it is not here.
    pub cause: String,
    /// What is therefore not in this store.
    pub cost: String,
}

impl Absence
{
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return format!(
            "absent: {}\n  expected: {}\n  cause:    {}\n  so:       {}",
            self.subject, self.expected, self.cause, self.cost
        );
    }
}

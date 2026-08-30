//! One corpus file that was not there, and what it cost.

/// Something the store was expected to hold and does not.
///
/// Every field is required. An absence that says only "no corpus" leaves the reader to
/// discover the variable's name, the path, and what is missing from the answer they just
/// received — and the last of those is the one they will not think to ask about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Absence
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

#[cfg(test)]
mod tests
{
    use super::Absence;

    /// Every field lands in the formatted line it names, in the order a reader needs them:
    /// what, where, why, and what that costs.
    #[test]
    fn Test_Describe_Should_Report_All_Four_Fields_In_Order()
    {
        let absence = Absence {
            subject: "the v14 authoring corpus".to_owned(),
            expected: "a directory named by A_CORPUS_VARIABLE".to_owned(),
            cause: "A_CORPUS_VARIABLE is not set".to_owned(),
            cost: "the corpus documents are not in this store".to_owned(),
        };

        assert_eq!(
            absence.Describe(),
            "absent: the v14 authoring corpus\n  expected: a directory named by A_CORPUS_VARIABLE\n  cause:    A_CORPUS_VARIABLE is not set\n  so:       the corpus documents are not in this store"
        );
    }
}

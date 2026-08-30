//! What this store was assembled from, and what it is missing.

use crate::corpus::Absence;

/// What went into this store, and what did not.
///
/// The rendering-free half of `nomos-cli::spec::verb::listing::Sources`: everything that
/// function used to write directly, kept here as data so a second caller can render it its
/// own way instead of parsing the lines `nomos-cli` happened to print.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourcesAnswer
{
    /// One line per input the assembly read, in the order it was read.
    pub read: Vec<String>,
    /// Every input the assembly expected and did not find.
    pub absent: Vec<Absence>,
}

impl SourcesAnswer
{
    /// Whether anything this store was expected to hold is missing.
    #[must_use]
    pub fn Is_Complete(&self) -> bool
    {
        return self.absent.is_empty();
    }

    /// Every absence, one after another.
    #[must_use]
    pub fn Describe_Absences(&self) -> String
    {
        return self
            .absent
            .iter()
            .map(Absence::Describe)
            .collect::<Vec<String>>()
            .join("\n");
    }
}

#[cfg(test)]
mod tests
{
    use super::{Absence, SourcesAnswer};

    fn Absence_Named(subject: &str) -> Absence
    {
        return Absence {
            subject: subject.to_owned(),
            expected: "somewhere".to_owned(),
            cause: "not there".to_owned(),
            cost: "nothing".to_owned(),
        };
    }

    #[test]
    fn Test_Is_Complete_Should_Be_True_With_No_Absences()
    {
        let answer = SourcesAnswer { read: Vec::new(), absent: Vec::new() };

        assert!(answer.Is_Complete());
    }

    #[test]
    fn Test_Is_Complete_Should_Be_False_With_At_Least_One_Absence()
    {
        let answer = SourcesAnswer { read: Vec::new(), absent: vec![Absence_Named("the v14 authoring corpus")] };

        assert!(!answer.Is_Complete());
    }

    #[test]
    fn Test_Describe_Absences_Should_Join_Every_Absence_By_Name()
    {
        let answer = SourcesAnswer {
            read: Vec::new(),
            absent: vec![Absence_Named("the v14 authoring corpus"), Absence_Named("the node catalog")],
        };

        let described = answer.Describe_Absences();
        assert!(described.contains("the v14 authoring corpus"), "{described}");
        assert!(described.contains("the node catalog"), "{described}");
    }
}

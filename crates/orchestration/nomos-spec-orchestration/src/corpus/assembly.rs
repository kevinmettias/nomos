//! A store assembled from a corpus, and what it is missing.

use crate::corpus::Absence;
use nomos_spec_store::SpecificationStore;
/// A store, what went into it, and what did not.
pub struct Assembly
{
    pub store: SpecificationStore,
    /// One line per input that was read, in the order it was read.
    pub read: Vec<String>,
    pub absent: Vec<Absence>,
}

impl Assembly
{
    /// Whether anything the store was expected to hold is missing.
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
    use super::{Absence, Assembly};
    use nomos_spec_store::SpecificationStore;

    #[test]
    fn Test_Is_Complete_Should_Be_True_With_No_Absences()
    {
        let assembly = Empty_Assembly();

        assert!(assembly.Is_Complete());
    }

    #[test]
    fn Test_Is_Complete_Should_Be_False_With_At_Least_One_Absence()
    {
        let mut assembly = Empty_Assembly();
        assembly.absent.push(Absence_Named("the v14 authoring corpus"));

        assert!(!assembly.Is_Complete());
    }

    #[test]
    fn Test_Describe_Absences_Should_Join_Every_Absence_By_Name()
    {
        let mut assembly = Empty_Assembly();
        assembly.absent.push(Absence_Named("the v14 authoring corpus"));
        assembly.absent.push(Absence_Named("the node catalog"));

        let described = assembly.Describe_Absences();
        assert!(described.contains("the v14 authoring corpus"), "{described}");
        assert!(described.contains("the node catalog"), "{described}");
    }

    #[test]
    fn Test_Describe_Absences_Should_Be_Empty_With_No_Absences()
    {
        let assembly = Empty_Assembly();

        assert_eq!(assembly.Describe_Absences(), "");
    }

    fn Absence_Named(subject: &str) -> Absence
    {
        return Absence {
            subject: subject.to_owned(),
            expected: "somewhere".to_owned(),
            cause: "not there".to_owned(),
            cost: "nothing".to_owned(),
        };
    }

    fn Empty_Assembly() -> Assembly
    {
        return Assembly {
            store: SpecificationStore::In_Memory().expect("an in-memory store always opens"),
            read: Vec::new(),
            absent: Vec::new(),
        };
    }
}

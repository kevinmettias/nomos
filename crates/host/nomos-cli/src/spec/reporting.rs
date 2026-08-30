//! Turning a refusal into a line the operator can act on.

use super::{Assembly, ExitCode, StoreError, ProjectError};

/// Turns an empty answer into an absence when something was missing.
///
/// The single place the rule lives. Every empty answer in this group goes through it, so
/// "nothing found" and "nothing was read in" cannot start printing the same way again.
pub(super) fn Absent_Or(
    assembly: &Assembly,
    otherwise: ExitCode,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    if assembly.Is_Complete()
    {
        return otherwise;
    }

    let _ = writeln!(
        notes,
        "This store is not whole — see the absence above. An identifier the corpus carries \
         is unknown here for that reason and not because nothing holds it, so this is \
         reported as an absence rather than as an empty result."
    );

    return ExitCode::Absent;
}

pub(super) fn Report_Store_Error(error: &StoreError, notes: &mut dyn std::io::Write) -> ExitCode
{
    let _ = writeln!(notes, "{error}");

    return ExitCode::StoreError;
}

/// A projection failure, as a number a caller can branch on.
///
/// Not every one of these is a store problem, and `5` for all of them was defensible only
/// while every one of them was. A subject the caller did not give and a subject the profile
/// cannot use are both arguments that were wrong before a row was read — an agent told the
/// store failed will retry; an agent told its command line was wrong will fix it. The rest
/// stay `StoreError` because that is what they are: the projection could not be built out
/// of what the store holds.
pub(super) fn Report_Project_Error(error: &ProjectError, notes: &mut dyn std::io::Write) -> ExitCode
{
    let _ = writeln!(notes, "{error}");

    return match *error
    {
        ProjectError::SubjectMissing { .. } | ProjectError::SubjectUnexpected { .. } =>
        {
            ExitCode::Usage
        }
        _ => ExitCode::StoreError,
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_orchestration::corpus::{Assemble_Corpus, CorpusRequest, DEFAULT_REVISION};

    /// A store with the embedded governing records seeded and no corpus, so
    /// [`Assembly::Is_Complete`] is deterministically `false` -- `root: None` records an
    /// absence unconditionally, regardless of what any real environment variable holds.
    fn Corpus_Unset_Assembly() -> Assembly
    {
        let request = CorpusRequest {
            variable: "NOMOS_SPEC_REPORTING_TEST_CORPUS_UNSET".to_owned(),
            root: None,
            revision: DEFAULT_REVISION.to_owned(),
        };

        return Assemble_Corpus(&request).expect("the embedded governing records always seed");
    }

    #[test]
    fn Test_Absent_Or_Should_Report_The_Absence_Over_A_Store_Missing_Its_Corpus()
    {
        let assembly = Corpus_Unset_Assembly();
        let mut notes = Vec::new();

        let code = Absent_Or(&assembly, ExitCode::NotFound, &mut notes);

        assert_eq!(code, ExitCode::Absent);
        assert!(!notes.is_empty(), "must say why NotFound became Absent");
    }

    #[test]
    fn Test_Report_Store_Error_Should_Write_The_Error_And_Report_StoreError()
    {
        let error = StoreError::Sql("boom".to_owned());
        let mut notes = Vec::new();

        let code = Report_Store_Error(&error, &mut notes);

        assert_eq!(code, ExitCode::StoreError);
        assert!(String::from_utf8_lossy(&notes).contains("boom"));
    }

    #[test]
    fn Test_Report_Project_Error_Should_Map_A_Missing_Subject_To_Usage_And_The_Rest_To_StoreError()
    {
        let mut notes = Vec::new();
        let usage_shaped = ProjectError::SubjectMissing { profile: "subject-dossier".to_owned() };
        assert_eq!(Report_Project_Error(&usage_shaped, &mut notes), ExitCode::Usage);

        let mut notes = Vec::new();
        let otherwise = ProjectError::Malformed("bad profile".to_owned());
        assert_eq!(Report_Project_Error(&otherwise, &mut notes), ExitCode::StoreError);
    }
}

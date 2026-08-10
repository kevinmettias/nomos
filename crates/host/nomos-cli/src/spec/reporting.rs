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

/// A document that resolved and then could not be read back.
///
/// Its own function because the situation is a store defect rather than a caller's
/// mistake: the surrogate came out of a query against the same connection.
pub(super) fn Vanished(uid: i64) -> StoreError
{
    return StoreError::Sql(format!(
        "document {uid} resolved and then could not be read back from the same connection"
    ));
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

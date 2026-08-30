//! Reading the ledger file and saying what is wrong with one that will not parse.

use std::path::Path;

use crate::LedgerError;

/// Which of the two parse failures this is.
///
/// A file newer than this build and a file that is simply broken both fail
/// `serde_json::from_str`, and the operator's next action is opposite in the two cases: rebuild
/// the reader, or repair the file. Telling them apart is the whole of what [`SCHEMA_VERSION`]
/// does — it is consulted here, after the refusal, and never to decide whether to refuse.
///
/// A forgotten bump therefore degrades this to [`LedgerError::Malformed`] and costs a sentence.
/// It cannot cost a field.
pub(super) fn Explain_Parse_Failure(path: &Path, text: &str, error: &serde_json::Error) -> LedgerError
{
    use super::SCHEMA_VERSION;
    use crate::store::VersionProbe;

    if let Ok(probe) = serde_json::from_str::<VersionProbe>(text)
        && probe.schema_version > SCHEMA_VERSION
    {
        return LedgerError::Unrecognized {
            understood: SCHEMA_VERSION,
            found: probe.schema_version,
            cause: error.to_string(),
        };
    }

    return LedgerError::Malformed {
        cause: format!("{}: {error}", path.display()),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::LedgerDocument;

    #[test]
    fn Test_Explain_Parse_Failure_Should_Report_Unrecognized_For_A_Schema_Newer_Than_This_Build()
    {
        let path = Path::new("work/ledger.json");
        // Unknown to `LedgerDocument`'s own strict parse (deny_unknown_fields refuses
        // `bogus`), but still readable as a bare schema-version probe -- exactly the two
        // facts this function has to reconcile.
        let text = r#"{"schema_version":999999,"items":[],"bogus":true}"#;
        let error = serde_json::from_str::<LedgerDocument>(text).expect_err("the unknown field must refuse");

        let explained = Explain_Parse_Failure(path, text, &error);

        assert!(
            matches!(explained, LedgerError::Unrecognized { found: 999_999, .. }),
            "got {explained:?}"
        );
    }

    #[test]
    fn Test_Explain_Parse_Failure_Should_Report_Malformed_When_The_Schema_Is_Not_Newer()
    {
        let path = Path::new("work/ledger.json");
        let text = r#"{"schema_version":1,"items":[],"bogus":true}"#;
        let error = serde_json::from_str::<LedgerDocument>(text).expect_err("the unknown field must refuse");

        let explained = Explain_Parse_Failure(path, text, &error);

        assert!(matches!(explained, LedgerError::Malformed { .. }), "got {explained:?}");
    }
}

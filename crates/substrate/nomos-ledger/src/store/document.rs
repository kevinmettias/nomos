//! Reading the ledger file and saying what is wrong with one that will not parse.

use std::path::Path;

use crate::ledger_document::VersionProbe;
use crate::LedgerError;

use super::SCHEMA_VERSION;

/// Which of the two parse failures this is.
///
/// A file newer than this build and a file that is simply broken both fail
/// `serde_json::from_str`, and the operator's next action is opposite in the two cases: rebuild
/// the reader, or repair the file. Telling them apart is the whole of what [`SCHEMA_VERSION`]
/// does — it is consulted here, after the refusal, and never to decide whether to refuse.
///
/// A forgotten bump therefore degrades this to [`LedgerError::Malformed`] and costs a sentence.
/// It cannot cost a field.
pub(super) fn Explain(path: &Path, text: &str, error: &serde_json::Error) -> LedgerError
{
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

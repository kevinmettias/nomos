//! [`Build_Corpus_Request`], the one `CorpusRequest` composition every `Handle_Spec_*`
//! function that touches the store builds from the environment alone.

use nomos_spec_orchestration::corpus::{CorpusRequest, DEFAULT_REVISION};

/// Which environment variable names the corpus root -- the same constant `nomos-cli`'s own
/// `main.rs` keeps privately, ported here rather than shared for the reason every other
/// composition-root value in this crate is: `OD-HOST-002` treats it as this root's own
/// choice, not a shared dependency.
const CORPUS_VARIABLE: &str = "NOMOS_V14_CORPUS";

/// The `CorpusRequest` composition every `Handle_Spec_*` function that touches the store
/// builds from the environment alone -- factored out once enough of them needed exactly the
/// same three fields the first already had inline. Mirrors `crate::work::Ledger_At`.
pub(crate) fn Build_Corpus_Request() -> CorpusRequest
{
    use std::path::PathBuf;

    return CorpusRequest {
        variable: CORPUS_VARIABLE.to_owned(),
        root: std::env::var_os(CORPUS_VARIABLE).map(PathBuf::from),
        revision: DEFAULT_REVISION.to_owned(),
    };
}

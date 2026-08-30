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

#[cfg(test)]
mod tests
{
    use super::*;
    use std::path::PathBuf;

    /// Asserted against the environment itself, rather than a fixed `Some`/`None`, because
    /// whether `NOMOS_V14_CORPUS` is set is a fact of the session this test happens to run in
    /// -- the same tolerance `crate::spec::sources_response`'s own real-call test gives that
    /// variable. What this composition owes is reading the one variable it names and the one
    /// default revision it carries, not a particular environment.
    #[test]
    fn Test_Build_Corpus_Request_Should_Read_The_Same_Environment_Variable_It_Names()
    {
        let request = Build_Corpus_Request();

        assert_eq!(request.variable, CORPUS_VARIABLE);
        assert_eq!(request.revision, DEFAULT_REVISION);
        assert_eq!(request.root, std::env::var_os(CORPUS_VARIABLE).map(PathBuf::from));
    }
}

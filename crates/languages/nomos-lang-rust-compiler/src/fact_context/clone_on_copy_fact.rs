//! The one shape `nomos.cap.rust.copy_clones` hands a caller back.

use nomos_analysis::MaterializedFact;
use nomos_contracts::SubjectId;

/// One materialized fact, addressed to the crate it was produced for -- the same
/// `{subject, fact}` pairing `nomos_lang_rust_deny::PolicyFact` returns, for the
/// identical reason: a caller writing this into a fact store needs the subject the key
/// was built against, and a `FactKey` does not carry it back out.
pub struct CloneOnCopyFact
{
    pub subject: SubjectId,
    pub fact: MaterializedFact,
}

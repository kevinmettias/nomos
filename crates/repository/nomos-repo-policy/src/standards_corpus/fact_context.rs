//! Turning a repository's declared standards corpus into the one fact this capability answers.

use super::guarantee::{Declared_Guarantee, PROVIDER};
use super::reading::{Discover_Workspace, StandardsCorpusError};
use crate::scaffolding;
use nomos_cap_standards_corpus::{Capability, Encode_Payload, Payload_Schema, CONTRACT_VERSION};
use nomos_platform::FileSystem;
use std::path::Path;

pub use crate::scaffolding::{FactContext, PolicyFact};

/// Reads the corpora `root`'s own `nomos-standards-corpus.json` declares and produces the one
/// fact this capability answers for the workspace as a whole — a leaf: nothing here reads
/// another fact this or any other provider produced.
///
/// A repository declaring no corpus still produces a fact, and that is the decision the
/// `done_when` this provider was built under states as a requirement rather than an
/// oversight: "a repository that declares no corpus judges exactly as it does today". An
/// absent fact would instead make a rule that reads this capability report its *own* absence,
/// which is a different statement about the repository and a false one.
///
/// # Errors
///
/// Whatever [`Discover_Workspace`] returns.
pub fn Materialize_Workspace<Fs: FileSystem>(root: &Path, context: FactContext, filesystem: &Fs) -> Result<PolicyFact, StandardsCorpusError>
{
    let payload = Discover_Workspace(root, filesystem)?;

    return Ok(scaffolding::Materialize_Whole_Workspace(scaffolding::WholeWorkspaceFiling {
        subject: scaffolding::Workspace_Subject(),
        guarantee: Declared_Guarantee(),
        identity: scaffolding::Declared_Identity(Capability(), CONTRACT_VERSION, PROVIDER, CONTRACT_VERSION),
        context,
        schema: Payload_Schema(),
        bytes: Encode_Payload(&payload),
    }));
}

#[cfg(test)]
#[path = "fact_context/tests.rs"]
mod tests;

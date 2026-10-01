//! The gate policy file a root that has none can be started with.
//!
//! # Why it declares nothing
//!
//! `nomos_gate_orchestration`'s own reader is explicit that a field left at its default
//! states nothing rather than stating the default -- `GatePolicyFile::Contributed` maps
//! every such field to no contribution at all, under `OD-POLICY-001`'s "a sentinel is not a
//! statement". So a file carrying every field at its empty value is a repository that has
//! declared nothing, which is exactly the state it was in before the file existed, and a
//! run over it answers exactly as it did.
//!
//! That is the conservative choice and it is the whole design. A starter file that
//! suppressed nothing but *demanded* something -- a coverage floor, a phase that blocks --
//! would fail the first run a new adopter ever performs, over findings they have not read
//! yet, on a repository they have not calibrated. What a person learns from that is to pass
//! a flag that turns the gate off, and a gate somebody has learned to bypass is worth less
//! than no gate at all, because it still looks like one in the workflow file.
//!
//! What the file is for, then, is not what it says but what it shows: every key this reader
//! accepts, spelled correctly, in a file that already parses. `deny_unknown_fields` means a
//! misspelled key is refused rather than ignored, so a person editing from here cannot
//! invent a field that reads as nothing -- and it also means this starter's own keys are
//! checked by the real reader the moment anything runs the gate, which is what
//! `Test_The_Starter_Policy_Should_Be_Accepted_By_The_Real_Gate_Reader` asserts rather than
//! trusting the spelling here.
//!
//! # What it deliberately omits
//!
//! No suppression, no baseline entry, no rule calibration: each of those is a statement
//! about a specific finding at a specific path in a repository this file has never seen.
//! No coverage floor beyond `unset`, because `require-completeness` turns an incomplete
//! claim into a withheld verdict and a repository whose providers are not all available
//! here -- which `nomos profile`'s own report is likely to have just told the reader -- would
//! reach no verdict on day one. No phase and no approval, because a phase blocks and an
//! approval names a phase, and neither means anything until somebody has decided what the
//! stages of their own build are.

use super::StarterOutcome;
use nomos_platform::FileSystem;
use std::path::Path;

/// The file a gate run resolves its policies from, relative to the root.
///
/// A literal here rather than `nomos_gate_orchestration`'s own `GATE_POLICY_FILE`, for the
/// reason `nomos_workspace_discovery::PolicyFile` already gives for repeating it: that
/// constant is `pub(crate)`, so no other crate can name it whatever its zone. The spelling
/// is not taken on trust -- the test named in this module's doc writes this file and has the
/// real reader resolve it, which fails if the name is wrong as surely as if the contents
/// were.
const GATE_POLICY_FILE: &str = "nomos-gate.json";

/// Every key `nomos-gate.json`'s reader accepts, each at the value that states nothing.
///
/// Written out rather than serialized from a default value, because this crate has no JSON
/// writer and would be taking a dependency to produce six empty fields. The cost of writing
/// it by hand is that a key could be misspelled, and that cost is paid by the test that has
/// the real reader read it back: `deny_unknown_fields` refuses a key this reader does not
/// know, so a misspelling here is a failing test rather than a field silently doing nothing.
const STARTER_GATE_POLICY: &str = "{\n  \
     \"suppressions\": [],\n  \
     \"baseline\": [],\n  \
     \"adoption\": [],\n  \
     \"coverage\": \"unset\",\n  \
     \"phases\": [],\n  \
     \"approvals\": []\n\
     }\n";

/// Writes the starter policy under `root`, unless something is already there.
///
/// Existence is checked through the same port the write goes through, so the two cannot
/// disagree about what is at the path -- and it is checked rather than relying on the write
/// to fail, because `FileSystem::Replace_Atomically` is documented to replace: a write that
/// found an existing file would succeed and the declaration it replaced would be gone.
pub(crate) fn Write_Starter_Gate_Policy<Fs: FileSystem>(root: &Path, filesystem: &Fs) -> StarterOutcome
{
    let path = root.join(GATE_POLICY_FILE);

    if filesystem.Exists(&path)
    {
        return StarterOutcome::AlreadyDeclared { path };
    }

    return match filesystem.Replace_Atomically(&path, STARTER_GATE_POLICY)
    {
        Ok(()) => StarterOutcome::Written { path },
        Err(error) => StarterOutcome::Unwritable { path, reason: format!("{error:?}") },
    };
}

//! One repository policy file a reader in this workspace opens, and what a first run finds
//! of it.

use crate::{PolicyFilePresence, ROOT_MARKER};
use std::path::Path;

/// The file a gate run resolves its four policies from.
///
/// A literal here rather than `nomos_gate_orchestration`'s own `GATE_POLICY_FILE`, and the
/// reason is reach: that constant is `pub(crate)`, so no other crate can name it whatever
/// its zone. This profile never reads the file for meaning. It asks whether one is there and
/// whether its bytes are text, the same distinction [`ROOT_MARKER`] already draws for
/// `standards.json`: what the file means is `Resolve_Gate_Policy`'s own reading.
const GATE_POLICY_FILE: &str = "nomos-gate.json";

/// The file a repository declares its architecture in.
///
/// `nomos_repo_policy::architecture::ARCHITECTURE_JSON` is the constant that owns this
/// literal, and it is public in a zone (Provider) this crate may reach. It is repeated here
/// because `P123-WORKSPACE-PROFILE-FOR-ADOPTION` adds no dependency to any manifest -- the
/// lock file is another item's territory -- and not because the dependency would be wrong.
/// The item that adds `nomos-repo-policy` to this crate's manifest should replace this
/// literal and [`TEST_MATERIAL_JSON`] with the constants that own them.
const ARCHITECTURE_JSON: &str = "nomos-architecture.json";

/// The file a repository declares its fixture locations in.
///
/// `nomos_repo_policy::test_material::TEST_MATERIAL_JSON` owns this literal, and it is
/// repeated here for exactly the reason [`ARCHITECTURE_JSON`] gives.
const TEST_MATERIAL_JSON: &str = "nomos-test-material.json";

/// One policy file a reader in this workspace opens at a root, named, with what a first run
/// finds of it.
///
/// Named rather than keyed, because the name is what a person adopting nomos acts on: a
/// report that says `nomos-architecture.json: absent` tells them which file to write, and a
/// report that says `architecture: absent` does not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyFile
{
    /// The file's own name at the root -- what a reader opens.
    pub name: &'static str,
    /// What a first run finds of it.
    pub presence: PolicyFilePresence,
}

impl PolicyFile
{
    /// Every policy file a reader in this workspace opens, in one fixed order, each checked
    /// against `root`.
    #[must_use]
    pub fn At_Root(root: &Path) -> Vec<Self>
    {
        return Opened_Policy_Files()
            .into_iter()
            .map(|name| return Self { name, presence: PolicyFilePresence::Of_Path(&root.join(name)) })
            .collect();
    }
}

/// The policy files a profile asks about, in the order a profile reports them: the one this
/// workspace shares with another tool first, then the three it owns outright in the order
/// their readers arrived. A function returning the list rather than a module-level array,
/// the way [`crate::Registered_Extensions`] is.
fn Opened_Policy_Files() -> Vec<&'static str>
{
    return vec![ROOT_MARKER, GATE_POLICY_FILE, ARCHITECTURE_JSON, TEST_MATERIAL_JSON];
}

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

/// The file a gate run reads its own occurrence history from, beside [`GATE_POLICY_FILE`].
///
/// A literal here rather than `nomos_gate_orchestration`'s own `GATE_HISTORY_FILE`, for the
/// reason [`GATE_POLICY_FILE`] gives: that constant is `pub(in crate::gate_environment)`, so no
/// other crate can name it whatever its zone. It is not a policy -- its reader's own doc says
/// the file is evidence a run collected, which `OD-GATE-030` keeps apart from what a repository
/// decided -- and it is listed anyway, because its presence is the whole of its configuration:
/// a run records history only into a file that is already there, so whether one is there is
/// the decision a person adopting nomos makes, and writing it is the act this entry names.
/// `P128-A-BASELINED-FINDING-CANNOT-BE-TOLD-FROM-ONE-REINTRODUCED` gave it a reader after this
/// list existed, without adding it here.
const GATE_HISTORY_FILE: &str = "nomos-gate-history.json";

/// The file a repository declares the standards corpora it owns in.
///
/// `nomos_repo_policy::standards_corpus::STANDARDS_CORPUS_JSON` owns this literal, and it is
/// repeated here for exactly the reason [`ARCHITECTURE_JSON`] gives. `P149` gave it a reader
/// without adding it here, so a profile said nothing about it until `OD-RULES-035`'s limits
/// item edited this list and added both.
const STANDARDS_CORPUS_JSON: &str = "nomos-standards-corpus.json";

/// The file a repository declares its limits in.
///
/// `nomos_repo_policy::limits::LIMITS_JSON` owns this literal, and it is repeated here for
/// exactly the reason [`ARCHITECTURE_JSON`] gives.
const LIMITS_JSON: &str = "nomos-limits.json";

/// The file a repository declares the builds its C# is judged under in.
///
/// `nomos_repo_policy::csharp_builds::CSHARP_BUILDS_JSON` owns this literal, and it is repeated
/// here for exactly the reason [`ARCHITECTURE_JSON`] gives. The item that gave it a reader added
/// it here in the same change, so a profile names every file a reader opens.
const CSHARP_BUILDS_JSON: &str = "nomos-csharp-builds.json";

/// The file a repository declares what an item's predicate must carry in, judged when `work add`
/// and `work widen` reserve a path it names (`OD-GATE-036`).
///
/// `nomos_ledger::PREDICATE_COVERAGE` owns this literal, and it is public, so neither the reach
/// [`GATE_POLICY_FILE`] gives nor the lock file [`ARCHITECTURE_JSON`] gives is the reason it is
/// repeated here. The reason is zone: `nomos-ledger` is Repo Tooling, which
/// `nomos-architecture.json` does not permit Application Service, this crate's zone, to reach,
/// and no exception names the edge -- so, unlike the `nomos_repo_policy` constants above, no
/// manifest edit could let this crate name it. The dependency would be wrong, not merely
/// unreserved. `P196-A-CHANGE-TO-THE-COMPOSED-SET-NAMES-THE-PACKAGES-THAT-ENUMERATE-IT` gave it
/// a reader without adding it here, because this crate lay outside that item.
const PREDICATE_COVERAGE: &str = "nomos-predicate-coverage.json";

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
/// workspace shares with another tool first, then the ones it owns outright in the order
/// their readers arrived -- gate policy, architecture, test material, gate history, standards
/// corpus, limits, C# builds, predicate coverage. A function returning the list rather than a
/// module-level array, the way [`crate::Registered_Extensions`] is.
fn Opened_Policy_Files() -> Vec<&'static str>
{
    return vec![
        ROOT_MARKER,
        GATE_POLICY_FILE,
        ARCHITECTURE_JSON,
        TEST_MATERIAL_JSON,
        GATE_HISTORY_FILE,
        STANDARDS_CORPUS_JSON,
        LIMITS_JSON,
        CSHARP_BUILDS_JSON,
        PREDICATE_COVERAGE,
    ];
}

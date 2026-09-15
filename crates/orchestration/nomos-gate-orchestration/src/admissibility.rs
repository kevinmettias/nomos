//! Whether one crate may name another, asked before any manifest carries the edge.
//!
//! Every other answer this crate gives is retrospective. `Check_Dependency_Direction` reads
//! `nomos.cap.dependency.edges`, which a provider derives by running `cargo metadata` over
//! manifests on disk, so an edge is judged once it has been written and never before. This
//! module is the one prospective answer: a crate pair, and what the architecture says about
//! it, with nothing on disk.
//!
//! # Why this is cheap, and why that is the argument for it existing
//!
//! `OD-GATE-026` measured the two answers. `nomos gate run` over this workspace takes 13,127
//! milliseconds and `nomos gate explain` takes 40, and this answer is in `explain`'s class,
//! not `run`'s — it reads no manifest, runs no subprocess, walks no tree and touches no
//! store. But the thirteen seconds is not the cost that mattered. The cost that mattered is
//! that the retrospective answer requires the edge to exist, so a developer asking "may I?"
//! has to do the thing first. `OD-RULES-020` records what that cost when the answer turned
//! out to be no: `nomos-workflow-orchestration` renumbered three times across two items in a
//! single session.
//!
//! # Where the answer comes from, now that the migration `OD-GATE-026` anticipated has landed
//!
//! That record put one constraint on this module: the answer comes through the lookups and
//! never by reaching around them into a table, because `OD-RULES-029` had decided the layering
//! declaration would become data read from the repository under check. It has. The question
//! this module asks is unchanged and the lookups are unchanged; what moved is that they are
//! now queries over a declaration a caller reads, so this function takes one.
//!
//! A caller is what reads it, and that is not a cost this module absorbed quietly.
//! `OD-GATE-026` argued for this verb partly on being cheap -- "it reads no manifest, runs no
//! subprocess, walks no tree and touches no store" -- and one file read is now part of
//! answering it. The argument survives intact: what that record actually weighed was that the
//! retrospective answer requires the edge to exist, so a developer asking "may I?" has to do
//! the thing first. Reading one declaration does not reintroduce that.

mod depended_crate;
mod depending_crate;

pub use depended_crate::DependedCrate;
pub use depending_crate::DependingCrate;

use nomos_cap_architecture::ArchitecturePayload;
use nomos_platform::FileSystem;
use std::path::Path;

/// The file a repository declares its architecture in, re-stated here for a composition root
/// that has to find it before it can name a root to read it from.
///
/// Re-exported rather than re-spelled: `nomos_repo_policy::architecture` owns the name, and a
/// host that typed the string itself would be a second place to change it.
pub const ARCHITECTURE_DECLARATION_FILE: &str = nomos_repo_policy::architecture::ARCHITECTURE_JSON;

/// What the architecture says about an edge that does not exist yet.
///
/// Three outcomes rather than two, and the third is the one that makes this safe to consult.
/// A declaration places the crates its own repository has placed and no others, so a crate it
/// says nothing about is the ordinary case rather than an error, and `OD-RULES-003` already
/// decided what that case is owed: a positive statement that no judgment was reached, never a
/// judgment.
///
/// **There is deliberately no default of [`Admissibility::Permitted`].** A prospective check
/// that answers "fine" about a crate it has never heard of is worse than no check at all,
/// because the only caller is somebody who does not yet know the answer and has no way to
/// tell a real yes from a shrug.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Admissibility
{
    /// The declaration places both crates and admits the edge.
    Permitted,
    /// The declaration places both crates and does not admit the edge.
    Refused,
    /// The declaration does not place at least one of the two, so there is nothing to judge
    /// against.
    NotJudged,
}

impl Admissibility
{
    /// Whether this answer is a judgment at all.
    ///
    /// A predicate rather than `== NotJudged` at each call site: a caller asking "did I get an
    /// answer" should not have to know which variant means "no", and `OD-RULES-003`'s whole
    /// point is that the absence of a judgment is a thing said rather than a thing inferred.
    #[must_use]
    pub const fn Is_A_Judgment(self) -> bool
    {
        return !matches!(self, Self::NotJudged);
    }
}

/// Whether `depending` may name `depended`, judged against `architecture` alone.
///
/// The same-component case is asked separately because a component order cannot express it: a
/// declaration that admits no component against itself -- which is how a repository keeps two
/// peers from naming each other -- would otherwise refuse every real edge inside one
/// component, and the named exceptions are the list for which that refusal is lifted.
///
/// A crate naming itself is not an edge and is refused rather than admitted by the exception
/// list, which names pairs of two different crates.
#[must_use]
pub fn Admits(architecture: &ArchitecturePayload, depending: DependingCrate<'_>, depended: DependedCrate<'_>) -> Admissibility
{
    let (Some(from), Some(to)) = (architecture.Component_Of(depending.0), architecture.Component_Of(depended.0))
    else
    {
        return Admissibility::NotJudged;
    };

    if from == to
    {
        return Is_A_Declared_Peer(architecture, depending, depended);
    }

    if architecture.Permits(from, to)
    {
        return Admissibility::Permitted;
    }

    return Admissibility::Refused;
}

/// Whether two crates sharing a component are one of the pairs the declaration excepts.
///
/// Directed, as the declaration is: that a composition root may name a provider does not mean
/// the provider may name the composition root, and a list read in both directions would admit
/// exactly the cycles a component forbidding its own members exists to prevent.
fn Is_A_Declared_Peer(architecture: &ArchitecturePayload, depending: DependingCrate<'_>, depended: DependedCrate<'_>) -> Admissibility
{
    if architecture.Excepts(depending.0, depended.0)
    {
        return Admissibility::Permitted;
    }

    return Admissibility::Refused;
}

/// [`Admits`], over the architecture `root` declares, read through `filesystem`.
///
/// The entry a composition root calls, so that reading the declaration is this crate's own
/// concern rather than every host's. It is the same port and the same shape `Run_Gate` already
/// resolves `nomos-gate.json` through.
///
/// A declaration that cannot be read is an empty one, and the answer is then
/// [`Admissibility::NotJudged`] rather than a guess -- the same answer any crate a declaration
/// does not place already gets. There is deliberately no louder failure: this verb exists to
/// answer a question somebody has before the edge exists, and a repository that has not
/// declared an architecture has not answered it, which is a true thing to say.
#[must_use]
pub fn Admits_Under<Fs: FileSystem>(root: &Path, filesystem: &Fs, depending: DependingCrate<'_>, depended: DependedCrate<'_>) -> Admissibility
{
    let architecture = nomos_repo_policy::architecture::Discover_Workspace(root, filesystem).unwrap_or_default();

    return Admits(&architecture, depending, depended);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_cap_architecture::{Exception, Membership, Permission};

    /// The entry the tests below read, so that naming the two ends of the edge is written once
    /// rather than at every call site.
    fn Admits_In(declaration: &ArchitecturePayload, depending: DependingCrate<'_>, depended: DependedCrate<'_>) -> Admissibility
    {
        return Admits(declaration, depending, depended);
    }

    #[test]
    fn Test_An_Edge_The_Architecture_Admits_Should_Be_Permitted()
    {
        assert_eq!(Admits_In(&Declaration(), DependingCrate("http"), DependedCrate("billing")), Admissibility::Permitted);
    }

    #[test]
    fn Test_An_Edge_The_Architecture_Does_Not_Admit_Should_Be_Refused()
    {
        assert_eq!(Admits_In(&Declaration(), DependingCrate("billing"), DependedCrate("http")), Admissibility::Refused);
    }

    #[test]
    fn Test_Two_Peers_In_One_Component_Should_Be_Refused_Without_A_Named_Exception()
    {
        assert_eq!(Admits_In(&Declaration(), DependingCrate("billing"), DependedCrate("invoicing")), Admissibility::Refused);
    }

    #[test]
    fn Test_A_Named_Exception_Should_Be_Permitted()
    {
        assert_eq!(Admits_In(&Declaration(), DependingCrate("billing"), DependedCrate("billing-core")), Admissibility::Permitted);
    }

    /// An exception is a directed statement about one real dependency.
    #[test]
    fn Test_The_Reverse_Of_A_Named_Exception_Should_Be_Refused()
    {
        assert_eq!(Admits_In(&Declaration(), DependingCrate("billing-core"), DependedCrate("billing")), Admissibility::Refused);
    }

    /// A crate naming itself is not an edge, and the exception list names pairs of two
    /// different crates, so it falls out as refused rather than admitted.
    #[test]
    fn Test_A_Crate_Naming_Itself_Should_Be_Refused()
    {
        assert_eq!(Admits_In(&Declaration(), DependingCrate("billing"), DependedCrate("billing")), Admissibility::Refused);
    }

    /// The third outcome, and the one that makes this safe to consult: a crate the declaration
    /// does not place gets no judgment rather than a permissive default.
    #[test]
    fn Test_A_Crate_The_Declaration_Does_Not_Place_Should_Not_Be_Judged()
    {
        assert_eq!(Admits_In(&Declaration(), DependingCrate("unplaced"), DependedCrate("billing")), Admissibility::NotJudged);
        assert_eq!(Admits_In(&Declaration(), DependingCrate("billing"), DependedCrate("unplaced")), Admissibility::NotJudged);
    }

    /// A repository that declared nothing is judged about nothing, which is the state every
    /// repository but this one was in while the table was compiled into `nomos-rules`.
    #[test]
    fn Test_A_Repository_That_Declared_Nothing_Should_Not_Be_Judged()
    {
        assert_eq!(Admits_In(&ArchitecturePayload::default(), DependingCrate("http"), DependedCrate("billing")), Admissibility::NotJudged);
    }

    /// Replace the declaration and the same function enforces the new one -- the property the
    /// whole migration is for, asked of the prospective answer rather than the retrospective.
    #[test]
    fn Test_The_Same_Function_Should_Enforce_A_Different_Declaration()
    {
        let reversed = ArchitecturePayload {
            permissions: vec![Permission { from: "Domain".to_owned(), to: "Api".to_owned() }],
            ..Declaration()
        };

        assert_eq!(Admits_In(&reversed, DependingCrate("http"), DependedCrate("billing")), Admissibility::Refused);
        assert_eq!(Admits_In(&reversed, DependingCrate("billing"), DependedCrate("http")), Admissibility::Permitted);
    }

    /// A declaration in a vocabulary this workspace does not use. Every assertion above is
    /// about the mechanism, and none of them could be written this way while the components
    /// were an enum compiled into `nomos-rules`.
    fn Declaration() -> ArchitecturePayload
    {
        return ArchitecturePayload {
            components: vec!["Domain".to_owned(), "Api".to_owned()],
            membership: vec![
                Membership { package: "billing".to_owned(), component: "Domain".to_owned() },
                Membership { package: "billing-core".to_owned(), component: "Domain".to_owned() },
                Membership { package: "invoicing".to_owned(), component: "Domain".to_owned() },
                Membership { package: "http".to_owned(), component: "Api".to_owned() },
            ],
            permissions: vec![Permission { from: "Api".to_owned(), to: "Domain".to_owned() }],
            exceptions: vec![Exception { from: "billing".to_owned(), to: "billing-core".to_owned() }],
            authorities: Vec::new(),
        };
    }

    #[test]
    fn Test_Is_A_Judgment_Should_Be_False_Only_For_Not_Judged()
    {
        assert!(Admissibility::Permitted.Is_A_Judgment());
        assert!(Admissibility::Refused.Is_A_Judgment());
        assert!(!Admissibility::NotJudged.Is_A_Judgment());
    }
}

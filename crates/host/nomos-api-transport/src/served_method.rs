//! The registry, and the reason it is an enum rather than a list.

use nomos_contracts::OperationName;

/// One operation this transport serves.
///
/// A closed enum rather than a map from names to function pointers, because `OD-HOST-007`
/// requires the exclusion it decided be structural rather than advisory: "A registry that is
/// merely short today, with nothing stopping a later increment from lengthening it, would be
/// the absence-as-boundary `OD-CONNECTOR-001` refuses." The four variants here are Gate's
/// three verbs, admitted from the start, plus `P62-TRANSPORT-MCP-CORRECTION-SURFACE-2`'s own
/// `Correction`: `nomos_api::Handle_Correction_Run` is a real, end-user-facing seam
/// (`P40-CORRECTIONS-CANONICAL-SEAM`), not one of the `Handle_Work_*` and `Handle_Spec_*`
/// handlers `nomos-api` also exports that belong to crates `README.md` marks
/// `[repo tooling]`; adding one of *those* would take a variant, a dispatch arm, and that
/// handler's name written into this crate's own source. `tests/contract`'s
/// `Test_The_Transport_Should_Name_No_Repo_Tooling_Handler` refuses the repo-tooling ones,
/// measured against `nomos-api`'s own blessed surface rather than against a list kept here
/// that could go stale beside it, and its own `ADMITTED` allow-list is what a handler this
/// crate calls must be added to deliberately -- widening `REGISTRY` alone is not enough to
/// admit one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServedMethod
{
    /// What this gate's rule registry holds, over no walk. `nomos_api::Handle_Gate_Plan`.
    GatePlan,
    /// A real judged run over a named tree. `nomos_api::Handle_Gate_Run`.
    GateRun,
    /// One named finding, and whether it would keep a real run from passing.
    /// `nomos_api::Handle_Gate_Explain`.
    GateExplain,
    /// A real correction run over a named tree, staging and, if asked, committing a fix.
    /// `nomos_api::Handle_Correction_Run`.
    Correction,
}

impl ServedMethod
{
    /// Every operation this transport serves.
    ///
    /// The array is the registry `OD-HOST-007` asks to be declared explicitly, and its length
    /// is part of the declaration: a fourth entry is a visible edit here rather than a line
    /// appended to a table somewhere else.
    ///
    /// Mirrored by `Test_The_Transport_Should_Name_No_Repo_Tooling_Handler`. What that check
    /// compares this list against is `nomos-api`'s own blessed surface: every dispatch arm
    /// below calls exactly one `nomos_api::Handle_*`, and it refuses any call outside the
    /// three that record admits while its sibling refuses a transport that calls fewer than
    /// all three. So the set of handlers this registry actually reaches is compared against
    /// the population it selects from, in both directions, which is the completeness question
    /// `OD-COMPLETENESS-001` asks of a declared list. It does not catch a variant added to
    /// this enum and left out of the array: that leaves an operation nothing serves rather
    /// than a registry claiming more than it serves, and this list stays true of what is
    /// served either way.
    pub const REGISTRY: [Self; 4] = [Self::GatePlan, Self::GateRun, Self::GateExplain, Self::Correction];

    /// This operation's canonical name.
    ///
    /// Spelled once, here, and projected rather than restated. `nomos_contracts::
    /// OperationName`'s own doc is the authority: it is "the **only** authoritative name for
    /// an operation. A CLI spelling, an MCP tool name, an HTTP route and a generated SDK
    /// method are all projections of this and none of them may disagree with it or with each
    /// other." This transport's JSON-RPC `method` field is that projection, and it is the
    /// identity projection deliberately -- a wire name transformed out of the canonical one
    /// is a second spelling that can drift from it, and this transport gains nothing by
    /// having one.
    #[must_use]
    pub const fn Name(self) -> &'static str
    {
        return match self
        {
            Self::GatePlan => "nomos.gate.plan",
            Self::GateRun => "nomos.gate.run",
            Self::GateExplain => "nomos.gate.explain",
            Self::Correction => "nomos.correction.run",
        };
    }

    /// The same name as `nomos-contracts`' own type for it.
    ///
    /// Built from [`Self::Name`] rather than beside it, so the literal exists once and the
    /// typed value cannot disagree with the wire string.
    #[must_use]
    pub fn Operation(self) -> OperationName
    {
        return OperationName::New(self.Name());
    }

    /// The operation `name` spells, or `None` for a name outside the registry.
    ///
    /// An unknown name is not an error here: JSON-RPC gives it its own code, and the caller
    /// of this function is what turns absence into that code.
    #[must_use]
    pub fn Named(name: &str) -> Option<Self>
    {
        return Self::REGISTRY.into_iter().find(|method| return method.Name() == name);
    }
}

#[cfg(test)]
mod tests
{
    use super::ServedMethod;

    /// Every registered operation round-trips through its own name -- so a name added to
    /// `Name` without a matching `REGISTRY` entry is unreachable and this fails rather than
    /// silently serving nothing.
    #[test]
    fn Test_Every_Registered_Operation_Should_Be_Found_By_Its_Own_Name()
    {
        for method in ServedMethod::REGISTRY
        {
            assert_eq!(ServedMethod::Named(method.Name()), Some(method), "{method:?}");
        }
    }

    /// The typed name and the wire name are the same string, which is what makes the
    /// projection the identity one this type's doc claims it is.
    #[test]
    fn Test_The_Canonical_Operation_Name_Should_Be_The_Wire_Name()
    {
        for method in ServedMethod::REGISTRY
        {
            assert_eq!(method.Operation().As_Str(), method.Name(), "{method:?}");
        }
    }

    /// A name outside the registry is refused rather than guessed at -- including one that
    /// only differs from a registered operation by the repo-tooling family it names.
    #[test]
    fn Test_A_Repo_Tooling_Operation_Name_Should_Not_Resolve()
    {
        assert_eq!(ServedMethod::Named("nomos.work.list"), None);
        assert_eq!(ServedMethod::Named("nomos.spec.commit"), None);
        assert_eq!(ServedMethod::Named("gate.run"), None);
    }
}

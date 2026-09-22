//! One dispatch target, as the value a resolution reads and dispatches through.

use nomos_model_package::ModelRoutePackage;

use crate::DispatchPort;

/// One dispatch target a composition root declares: the family a profile names it by, the
/// package that declares it a routing target, and the port it answers through.
///
/// `OD-PACKAGE-016` decision 2 decided the resolver resolves against a caller-supplied
/// sequence and discovers nothing; this is that sequence's element, and `port` is what
/// `OD-ROADMAP-005` decision 2 added to it. Before that, a target named a two-variant
/// `Backend` enum living in the generic path, so the generic path held the list of vendors
/// and called their functions directly. It holds neither now: a target arrives already
/// carrying the thing that answers it.
///
/// `family` is a label rather than a value of some closed set, for the same reason the
/// declaration travels in the request: the set of families is whatever a composition root
/// declares, and a family a caller cannot type is not a family anything resolves. Each
/// adapter states its own, so the spelling a person types after `--executor` and the
/// spelling a resolution matches are one string with one origin.
pub struct DeclaredTarget<'port>
{
    /// The family label [`nomos_model_package::ModelSelector::BackendFamily`] names this
    /// target by.
    pub family: String,
    /// The package that declares this target a routing target.
    ///
    /// `ModelRoutePackage`'s own reader already restricts `package_kind` to the two kinds
    /// that declare a model selection, so a value here is one of those by construction.
    pub package: ModelRoutePackage,
    /// What answers a dispatch that resolves to this target.
    pub port: DispatchPort<'port>,
}

//! The walk every body of `nomos workflow` defers to [`super::Run`], and the one guard each
//! walking body owes it.

use nomos_rules::SourceFile;
use nomos_workflow_orchestration::{CheckBody, CorrectionBody, GateBody};
use nomos_workspace_discovery::{Registered_Extensions, Walked_Sources};
use std::path::Path;

/// Every file under `root` this group recognizes, or `None` if `root` is not a directory.
///
/// The registered languages, which is what `correct.rs`'s own walk takes as well: a body's
/// `sources` arrive unwalked through [`super::Run`], so this is what turns the `root` a body
/// named into the files it is actually judged over. The population is deliberately not
/// `check::sources`'s -- the script extensions `check-script-discipline` judges are that
/// group's own additional recognition, and widening these bodies to cover them is a change to
/// what `nomos workflow` judges rather than a change to where its walk lives.
///
/// The walk itself, and the `target`/`.git`/nested-root skips a walk owes, are
/// `nomos-workspace-discovery`'s: `OD-HOST-008` put one beneath every composition root, so
/// this module and `correct.rs` each call it instead of each carrying a copy.
fn Walked(root: &Path) -> Option<Vec<SourceFile>>
{
    return Walked_Sources(root, &Registered_Extensions());
}

/// `check`, with its own `sources` replaced by a real walk of its `root` -- `None` if `root` is
/// not a directory, the guard [`Walked`] itself answers a tree that cannot be read at all with.
pub(super) fn Walked_Check(check: &CheckBody) -> Option<CheckBody>
{
    let sources = Walked(&check.root)?;

    return Some(CheckBody::New(check.root.clone(), sources, check.selected.clone()));
}

/// `correction`, with its own `sources` replaced by a real walk of its `root` -- `None` if `root`
/// is not a directory, the identical guard [`Walked_Check`] gives `Body::Check`.
/// `Run_Correction` itself already reports `NoSourceFound` for an empty, but real, walk, so there
/// is no second empty-source guard to duplicate here the way the parent module keeps one for
/// `Body::Check`.
pub(super) fn Walked_Correction(correction: &CorrectionBody) -> Option<CorrectionBody>
{
    let sources = Walked(&correction.root)?;

    return Some(CorrectionBody::New(correction.root.clone(), sources, correction.commit));
}

/// `gate`, with its own `sources` replaced by a real walk of `gate.command.root` -- `None` if that
/// root is not a directory, the identical guard [`Walked_Check`] and [`Walked_Correction`] both
/// already give.
pub(super) fn Walked_Gate(gate: &GateBody) -> Option<GateBody>
{
    let sources = Walked(&gate.command.root)?;

    return Some(GateBody::New(sources, gate.command.clone()));
}

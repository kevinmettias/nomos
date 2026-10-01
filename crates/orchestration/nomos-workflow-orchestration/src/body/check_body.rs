//! What a check-body step names, before `nomos-check-orchestration::Run` ever sees it.

use nomos_contracts::RuleId;
use nomos_rules::SourceFile;
use std::path::PathBuf;

/// The tree a check body judges, already walked, and the rules it judges against.
///
/// `nomos_check_orchestration::Run` takes its source already walked -- this crate has no
/// [`nomos_platform::FileSystem`] whose `Read_Directory` is one level rather than the
/// recursive walk this needs, the identical reason
/// rather than pushing it into the seam they call. A workflow step's own author walks (or
/// otherwise assembles) `sources` before building this body, the same way a step's
/// `TaskEnvelope` arrives with its `goal` already composed rather than the dispatch
/// constructing one.
///
/// `root` is carried for the same reason `nomos_check_orchestration::RunContext::root` is:
/// the dependency-edges, lint-diagnostics and dependency-policy providers each run their
/// own subprocess against a real tree, and a check body selecting one of those rules needs
/// a real path for that subprocess to run against even though `sources` itself is already
/// in memory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckBody
{
    /// The tree this check runs against, for the providers that launch a subprocess of
    /// their own.
    pub root: PathBuf,
    /// The already-walked source this check judges.
    pub sources: Vec<SourceFile>,
    /// Which rules this check runs. Empty selects every registered rule, the same
    /// "empty is everything" default `nomos_check_orchestration::Run` itself already has.
    pub selected: Vec<RuleId>,
}

impl CheckBody
{
    /// Constructs a check body.
    #[must_use]
    pub fn New(root: PathBuf, sources: Vec<SourceFile>, selected: Vec<RuleId>) -> Self
    {
        return Self { root, sources, selected };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Bodies_With_Equal_Content_Should_Be_Equal()
    {
        let one = CheckBody::New(PathBuf::from("."), Vec::new(), vec![RuleId::New("naming-convention")]);
        let other = CheckBody::New(PathBuf::from("."), Vec::new(), vec![RuleId::New("naming-convention")]);

        assert_eq!(one, other);
    }

    #[test]
    fn Test_A_Different_Selection_Should_Change_Equality()
    {
        let narrow = CheckBody::New(PathBuf::from("."), Vec::new(), vec![RuleId::New("naming-convention")]);
        let wide = CheckBody::New(PathBuf::from("."), Vec::new(), Vec::new());

        assert_ne!(narrow, wide);
    }
}

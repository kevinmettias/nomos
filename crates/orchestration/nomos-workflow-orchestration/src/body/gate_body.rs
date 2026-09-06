//! What a gate-body step names, before `nomos-gate-orchestration::Run_Gate` ever sees it.

use nomos_gate_orchestration::GateCommand;
use nomos_rules::SourceFile;

/// The tree a gate body judges, already walked, and the command `Run_Gate` reduces it
/// against.
///
/// `sources` is already walked for the identical reason [`crate::CheckBody::sources`] and
/// [`crate::CorrectionBody::sources`] both are: this crate has no `nomos_platform::
/// FileSystem` whose `Read_Directory` is one level rather than a recursive walk, so a
/// workflow step's own author assembles
/// it before building this body. `command` is [`nomos_gate_orchestration::GateCommand`]
/// itself, carried whole rather than flattened into a second copy of its fields: it is
/// already the one shape both `nomos gate run` and `nomos-api` construct, and a gate body
/// wants exactly what either of them would pass to `Run_Gate`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GateBody
{
    /// The already-walked source this gate judges.
    pub sources: Vec<SourceFile>,
    /// The root, scope and rule selection `Run_Gate` reduces `sources` against.
    pub command: GateCommand,
}

impl GateBody
{
    /// Constructs a gate body.
    #[must_use]
    pub fn New(sources: Vec<SourceFile>, command: GateCommand) -> Self
    {
        return Self { sources, command };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Bodies_With_Equal_Content_Should_Be_Equal()
    {
        let one = GateBody::New(Vec::new(), GateCommand::default());
        let other = GateBody::New(Vec::new(), GateCommand::default());

        assert_eq!(one, other);
    }

    #[test]
    fn Test_A_Different_Command_Should_Change_Equality()
    {
        let default_root = GateBody::New(Vec::new(), GateCommand::default());
        let named_root = GateBody::New(Vec::new(), GateCommand { root: "elsewhere".into(), ..GateCommand::default() });

        assert_ne!(default_root, named_root);
    }
}

//! One declared part of a system and the purposes it claims to serve.

/// One declared part of a system and the purposes it claims to serve.
///
/// A subsystem with an empty `goals` is the interesting case rather than a degenerate one:
/// it is exactly what `check-goal-traceability` calls a purposeless part, so the encoding
/// has to be able to say "this part is declared and serves nothing" distinctly from "this
/// part was never declared". That is why a subsystem gets a line of its own rather than
/// being implied by the goals it serves.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SubsystemDeclaration
{
    pub name: String,
    pub goals: Vec<String>,
}

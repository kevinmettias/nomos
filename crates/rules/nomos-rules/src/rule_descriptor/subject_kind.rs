//! What a rule reads, as the descriptor declares it.
//!
//! Its own file rather than a second type beside [`super::RuleDescriptor`]: the one-public-type
//! rule wants one file-home type per file, and this one has an identity of its own -- a rule's
//! subject is asked about in `nomos-lsp` and in the run's own reassessment cache without either
//! of them naming a descriptor.

/// What a rule reads to reach a judgment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubjectKind
{
    /// The text of each walked source, judged without consulting the fact store.
    SourceText,
    /// Facts filed under each walked source's own subject.
    SourceFacts,
    /// One whole-workspace fact, with no per-source subject of its own.
    Workspace,
}

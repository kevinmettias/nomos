//! [`FileDefinition`], a `#define` or `#undef` the file itself declares.

use super::definition_effect::DefinitionEffect;

/// A `#define` or `#undef` a file declares in a compiled part of itself, which changes the
/// definition set for the rest of that file and for no other.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileDefinition
{
    /// The one-based line the directive is on.
    pub line: usize,
    /// The symbol it defines or undefines.
    pub symbol: String,
    /// Whether it defines or undefines it.
    pub effect: DefinitionEffect,
}

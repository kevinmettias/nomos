//! One component-to-component dependency permission.

/// One `a component may depend on another` statement from a repository's own declaration.
///
/// The order over components, stated as the pairs it admits rather than as a number line.
/// `OD-RULES-020` decided that shape for this workspace and `OD-RULES-029` decided it travels
/// with the membership map rather than staying behind: a repository authoring its own
/// components in this workspace's permitted-edge matrix would have externalized nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Permission
{
    pub from: String,
    pub to: String,
}

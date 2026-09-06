//! One synchronization primitive whose own guarded value resolved to another
//! synchronization primitive.

/// One `std::sync::Mutex<T>` or `std::sync::RwLock<T>` whose type argument `T` a real
/// compiler frontend resolved -- through whatever type aliases, re-exports, or generic
/// substitution stood between the syntax and the answer -- to another
/// `std::sync::Mutex<U>` or `std::sync::RwLock<U>`: a lock guarding a value that is
/// already, itself, behind a lock.
///
/// No resolved type names, the same reasoning
/// [`crate::payload::cloned_copy_type::ClonedCopyType`] already gives for the identical
/// shape: nothing this capability promises depends on which two lock types were nested,
/// only that one was found inside the other -- a location is what lets a caller find the
/// site, and both types it names are visible there already.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NestedLockFinding
{
    /// Where the outer lock's own type is written, rendered `path:line:column`
    /// (one-based, the same convention a compiler diagnostic uses) against the file this
    /// analysis pass parsed it out of.
    pub location: String,
}

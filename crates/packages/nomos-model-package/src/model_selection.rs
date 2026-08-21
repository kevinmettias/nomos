//! What `MODEL-ROUTE-037`'s opening clause asks every `ModelBackendPackage` and
//! `AgentExecutorPackage` to declare.

/// How a package exposes which model or executor a request actually reaches.
///
/// `MODEL-ROUTE-037`: "Every `ModelBackendPackage` and `AgentExecutorPackage` shall expose
/// a versioned discovered model catalog or explicitly declare that model selection is
/// opaque or executor-controlled." Three variants, matching that sentence exactly rather
/// than inventing a fourth: a package says one of two things about why there is no
/// catalog, or it gives one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelSelection
{
    /// The package does not expose which model answers a request at all.
    Opaque,
    /// The executor chooses which model answers a request, not this package.
    ExecutorControlled,
    /// A versioned, discovered catalog of models this package can route to.
    ///
    /// Raw, unresolved identifiers -- the same choice `nomos_package::PackageManifest::
    /// language_versions` made for its own domain before a typed shape existed to resolve
    /// it against. `MODEL-ROUTE-037`'s second clause names nine states one catalog entry
    /// would eventually need to distinguish (configured identity, discovered identity,
    /// exact revision, mutable alias, deprecation, temporary unavailability, removal,
    /// entitlement availability, executor-selected identity); none of it is attempted
    /// here, the same later-maturity split `OD-PACKAGE-010` draws for the whole
    /// `MODEL-ROUTE-038`..`049` system.
    Catalog(Vec<String>),
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Variants_With_Equal_Content_Should_Be_Equal()
    {
        assert_eq!(
            ModelSelection::Catalog(vec!["gpt-x".to_owned()]),
            ModelSelection::Catalog(vec!["gpt-x".to_owned()])
        );
        assert_ne!(ModelSelection::Opaque, ModelSelection::ExecutorControlled);
    }
}

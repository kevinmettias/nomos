//! Describing the form of a declaration in the words the payload uses.

/// Whether an `impl` block serves a trait or is inherent.
///
/// It is the only thing that tells two `impl` blocks for one type apart. A member of
/// `impl Display for Table` carries the same qualified name as a member of `impl Table` and
/// does not belong to `Table` the same way, which is a distinction a consumer cannot
/// recover from any other field.
pub(super) fn Impl_Shape(serves_a_trait: bool) -> String
{
    if serves_a_trait
    {
        return nomos_cap_syntax::TRAIT.to_owned();
    }

    return nomos_cap_syntax::INHERENT.to_owned();
}

/// The single name a leaf of a use tree binds into this file.
///
/// A rename binds the alias, because the alias is what this file now has — what it aliases
/// is on the other side of a name resolution this provider does not perform. A glob binds
/// names this provider cannot enumerate without reading the module being imported, so it is
/// recorded as `*` rather than mistaken for a set of known bindings.
///
/// The two branching arms never reach here: [`Walk::Record_Use_Tree`] answers `Path` and
/// `Group` itself, because both are structure rather than a binding. They fall through to
/// the empty name, which records the `use` without claiming it introduced anything.
pub(super) fn Bound_By(leaf: &syn::UseTree) -> String
{
    return match leaf
    {
        syn::UseTree::Name(name) => name.ident.to_string(),
        syn::UseTree::Rename(rename) => rename.rename.to_string(),
        syn::UseTree::Glob(_) => "*".to_owned(),
        syn::UseTree::Path(_) | syn::UseTree::Group(_) => String::new(),
    };
}

/// The shape a declared type has, in the payload's vocabulary.
///
/// One distinction, and it is the one a consumer cannot recover from the other fields:
/// `pub const LIMIT: usize` and `pub const TABLES: &[&str]` are identical in every field a
/// v1 payload carried. `&[&str]` and `&'static [Self]` are lists however many references
/// deep; `&str` and `usize` are not.
pub(super) fn Type_Shape(declared: &syn::Type) -> String
{
    return match declared
    {
        syn::Type::Reference(reference) => Type_Shape(&reference.elem),
        syn::Type::Slice(_) | syn::Type::Array(_) => nomos_cap_syntax::SLICE.to_owned(),
        _ => nomos_cap_syntax::VALUE.to_owned(),
    };
}

/// The `shape` a struct's own named fields declare, `OD-CAPABILITY-010`'s extension to this
/// payload's per-kind vocabulary.
///
/// `None` for `syn::Fields::Unit` and `syn::Fields::Unnamed` — a tuple or unit struct
/// declares no name a field-by-field comparison could key on, the same "nothing to say"
/// default every other kind already has for a form it does not describe. Each named
/// field's type is [`Type_Head`]'s answer, not a full generic-aware spelling:
/// `nomos-lang-rust` depends on `syn` with `printing` deliberately absent, and
/// `OD-CAPABILITY-010` reuses this crate's own existing boundary rather than widening it.
pub(super) fn Struct_Shape(fields: &syn::Fields) -> Option<String>
{
    let syn::Fields::Named(named) = fields
    else
    {
        return None;
    };

    let pairs: Vec<(String, String)> = named
        .named
        .iter()
        .filter_map(|field| {
            let name = field.ident.as_ref()?.to_string();
            return Some((name, Type_Head(&field.ty)));
        })
        .collect();

    return nomos_cap_syntax::Struct_Shape(&pairs);
}

/// The shape a function of this many declared parameters has.
///
/// The receiver counts, because it is a declared parameter and because the distinction a
/// consumer wants — `fn All()` against `fn All(&self)` — is exactly the one that vanishes
/// if it does not.
pub(super) fn Function_Shape(arity: usize) -> String
{
    return nomos_cap_syntax::Function_Shape(arity);
}

/// A path rendered as written, without the generic arguments.
pub(crate) fn Path_As_Written(path: &syn::Path) -> String
{
    let leading = if path.leading_colon.is_some() { "::" } else { "" };
    let segments: Vec<String> = path
        .segments
        .iter()
        .map(|segment| return segment.ident.to_string())
        .collect();

    return format!("{leading}{}", segments.join("::"));
}

/// The head of a type, as written.
///
/// Recursive through the wrappers that do not change what the type is *of*, so
/// `impl Foo`, `impl &Foo` and `impl [Foo; 4]` all record `Foo`. Anything with no single
/// head — a tuple, a trait object, a bare function type — records `_`, which says "this
/// provider has no name for it" rather than inventing one.
pub(super) fn Type_Head(kind: &syn::Type) -> String
{
    return match kind
    {
        syn::Type::Path(path) => Path_As_Written(&path.path),
        syn::Type::Reference(inner) => Type_Head(&inner.elem),
        syn::Type::Ptr(inner) => Type_Head(&inner.elem),
        syn::Type::Slice(inner) => Type_Head(&inner.elem),
        syn::Type::Array(inner) => Type_Head(&inner.elem),
        syn::Type::Paren(inner) => Type_Head(&inner.elem),
        syn::Type::Group(inner) => Type_Head(&inner.elem),
        _ => "_".to_owned(),
    };
}

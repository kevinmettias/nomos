//! Describing the form of a declaration in the words the payload uses.

/// Whether an `impl` block serves a trait or is inherent, and which type parameters it
/// declares of its own.
///
/// The first is the only thing that tells two `impl` blocks for one type apart. A member of
/// `impl Display for Table` carries the same qualified name as a member of `impl Table` and
/// does not belong to `Table` the same way, which is a distinction a consumer cannot
/// recover from any other field.
///
/// The second is `OD-CAPABILITY-014`'s extension, and it answers a question no other field
/// can either: an `impl` block's own recorded name is [`Type_Head`] of its self type, so
/// `impl<T: AsRef<[u8]>> ToHex for T` records `T` -- a name the block itself bound, not one
/// anybody chose for a declaration. Without the parameter list beside it, a consumer cannot
/// tell that apart from `impl Trait for T` over a real, one-letter-named type.
///
/// [`syn::Generics::type_params`] yields exactly the type parameters, skipping lifetimes and
/// const generics. That is the whole of what the record put in scope, and it is not a
/// simplification here: neither of those can ever collide with an `impl` block's own
/// recorded name, which is the one need the extension was measured for.
pub(super) fn Impl_Shape(serves_a_trait: bool, generics: &syn::Generics) -> String
{
    let parameters: Vec<String> = generics
        .type_params()
        .map(|parameter| return parameter.ident.to_string())
        .collect();

    return nomos_cap_syntax::Impl_Shape(serves_a_trait, &parameters);
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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Impl_Shape_Should_Distinguish_A_Trait_Impl_From_An_Inherent_One()
    {
        let none = syn::Generics::default();

        assert_eq!(Impl_Shape(true, &none), nomos_cap_syntax::TRAIT);
        assert_eq!(Impl_Shape(false, &none), nomos_cap_syntax::INHERENT);
    }

    /// The blanket-impl shape `OD-CAPABILITY-014` was measured on, parsed rather than
    /// hand-built, so what is asserted is what a real source file produces.
    ///
    /// The lifetime and the const parameter are in the fixture on purpose: the record put
    /// both out of scope, and a test that declared only type parameters could not tell a
    /// provider that honoured that from one that had never been asked.
    #[test]
    fn Test_Impl_Shape_Should_Carry_An_Impls_Own_Type_Parameters_And_Nothing_Else()
    {
        let block: syn::ItemImpl =
            syn::parse_str("impl<'a, T: AsRef<[u8]>, const N: usize> ToHex for T {}").expect("a valid impl parses");

        let shape = Impl_Shape(block.trait_.is_some(), &block.generics);
        let observed = nomos_cap_syntax::Observation::Present(shape);

        assert_eq!(nomos_cap_syntax::Impl_Serves_A_Trait(&observed), Some(true));
        assert_eq!(nomos_cap_syntax::Impl_Generics(&observed), Some(vec!["T".to_owned()]));
    }

    #[test]
    fn Test_Bound_By_Should_Bind_A_Renamed_Leafs_Own_Alias_Or_A_Globs_Star()
    {
        for (source, expected) in Bound_By_Cases()
        {
            let tree: syn::UseTree = syn::parse_str(source).expect("a valid use-tree fixture parses");

            assert_eq!(Bound_By(&tree), expected);
        }
    }

    /// (use-tree leaf source, the name it binds) — a second leaf shape beside these would
    /// extend the table rather than duplicate the test.
    fn Bound_By_Cases() -> Vec<(&'static str, &'static str)>
    {
        return vec![("HashMap as Map", "Map"), ("*", "*")];
    }

    #[test]
    fn Test_Type_Shape_Should_See_Through_A_Reference_To_A_Slice()
    {
        let ty: syn::Type = syn::parse_str("&[String]").expect("a valid type fixture parses");

        assert_eq!(Type_Shape(&ty), nomos_cap_syntax::SLICE);
    }

    #[test]
    fn Test_Type_Shape_Should_Be_A_Value_For_Anything_Else()
    {
        let ty: syn::Type = syn::parse_str("usize").expect("a valid type fixture parses");

        assert_eq!(Type_Shape(&ty), nomos_cap_syntax::VALUE);
    }

    #[test]
    fn Test_Struct_Shape_Should_Record_A_Named_Fields_Fields()
    {
        let item: syn::ItemStruct = syn::parse_str("struct S { n: u32, label: String }").expect("a valid struct fixture parses");

        let shape = Struct_Shape(&item.fields).expect("a named-field struct has a shape");

        assert!(shape.contains('n'), "{shape}");
    }

    #[test]
    fn Test_Struct_Shape_Should_Be_None_For_A_Tuple_Struct()
    {
        let item: syn::ItemStruct = syn::parse_str("struct S(u32);").expect("a valid struct fixture parses");

        assert_eq!(Struct_Shape(&item.fields), None);
    }

    #[test]
    fn Test_Function_Shape_Should_Delegate_To_The_Shared_Vocabulary()
    {
        assert_eq!(Function_Shape(2), nomos_cap_syntax::Function_Shape(2));
    }

    #[test]
    fn Test_Path_As_Written_Should_Keep_A_Leading_Double_Colon()
    {
        let path: syn::Path = syn::parse_str("::std::fmt::Display").expect("a valid path fixture parses");

        assert_eq!(Path_As_Written(&path), "::std::fmt::Display");
    }

    #[test]
    fn Test_Type_Head_Should_See_Through_A_Reference_And_Be_An_Underscore_With_No_Single_Head()
    {
        for (source, expected) in Type_Head_Cases()
        {
            let ty: syn::Type = syn::parse_str(source).expect("a valid type fixture parses");

            assert_eq!(Type_Head(&ty), expected);
        }
    }

    /// (type source, the head it records) — a second wrapper or headless form beside these
    /// would extend the table rather than duplicate the test.
    fn Type_Head_Cases() -> Vec<(&'static str, &'static str)>
    {
        return vec![("&Foo", "Foo"), ("(u32, u32)", "_")];
    }
}

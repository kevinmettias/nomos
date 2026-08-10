//! Reading one file, and what comes back when that fails.

use syn::visit::Visit;

/// What kind of declaration an item is.
///
/// Every item form Rust has, spelled out rather than collapsed into `Other`. An `Other`
/// bucket is where a form goes to be forgotten: the count stays right, nothing reports
/// it, and the day somebody needs `ForeignModule` they find it was never distinguished.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ItemKind
{
    Constant,
    Enum,
    ExternCrate,
    ForeignModule,
    Function,
    Implementation,
    MacroDefinition,
    Module,
    Static,
    Struct,
    Trait,
    TraitAlias,
    TypeAlias,
    Union,
    Use,
}

impl ItemKind
{
    /// The kind's stable `PascalCase` name, as it appears in an encoded payload.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Constant => "Constant",
            Self::Enum => "Enum",
            Self::ExternCrate => "ExternCrate",
            Self::ForeignModule => "ForeignModule",
            Self::Function => "Function",
            Self::Implementation => "Implementation",
            Self::MacroDefinition => "MacroDefinition",
            Self::Module => "Module",
            Self::Static => "Static",
            Self::Struct => "Struct",
            Self::Trait => "Trait",
            Self::TraitAlias => "TraitAlias",
            Self::TypeAlias => "TypeAlias",
            Self::Union => "Union",
            Self::Use => "Use",
        };
    }
}

impl core::fmt::Display for ItemKind
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

/// The visibility an item declares.
///
/// Four values, not a `bool`. A trait method and a private function are both "not
/// public" and they are not the same fact: one has no visibility to declare, and
/// recording it as `Private` would be this provider inventing a declaration the source
/// does not contain. [`Visibility::NotApplicable`] is what a sound provider says there.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Visibility
{
    /// `pub`.
    Public,
    /// `pub(crate)`, `pub(super)`, `pub(in path)` — with the scope as written.
    Restricted
    {
        scope: String,
    },
    /// No visibility keyword, on an item form that permits one.
    Private,
    /// An item form that declares no visibility: an `impl` block, a trait member.
    NotApplicable,
}

impl Visibility
{
    /// The stable label used in an encoded payload.
    #[must_use]
    pub fn Label(&self) -> String
    {
        return match self
        {
            Self::Public => "Public".to_owned(),
            Self::Restricted { scope } => format!("Restricted({scope})"),
            Self::Private => "Private".to_owned(),
            Self::NotApplicable => "NotApplicable".to_owned(),
        };
    }

    fn Of(visibility: &syn::Visibility) -> Self
    {
        return match visibility
        {
            syn::Visibility::Public(_) => Self::Public,
            syn::Visibility::Inherited => Self::Private,
            syn::Visibility::Restricted(restricted) =>
            {
                let path = Path_As_Written(&restricted.path);
                let scope = if restricted.in_token.is_some()
                {
                    format!("in {path}")
                }
                else
                {
                    path
                };

                Self::Restricted { scope }
            }
        };
    }
}

impl core::fmt::Display for Visibility
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(&self.Label());
    }
}

/// One declaration, as the file spells it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SyntaxItem
{
    /// Position in the walk, dense and zero-based. Source order, so it is stable for a
    /// given file and says nothing about any other file.
    pub ordinal: u32,
    pub kind: ItemKind,
    /// The syntactic nesting above this item, outermost first.
    ///
    /// Nesting, not a resolved module path. It does not begin at a crate root, because
    /// this provider does not know which crate the file belongs to — that is workspace
    /// structure, and reading it would make the answer a function of more than this file.
    pub scope: Vec<String>,
    /// The name as written. For an `impl` block, the head of the self type; for a `use`
    /// leaf, the binding it introduces.
    pub name: String,
    pub visibility: Visibility,
    /// The item's documentation, as written, or `None` when it has none.
    ///
    /// `None` is an absence this provider looked for and did not find, never an inability
    /// to look — it parses, so it always sees the attributes. That difference is what the
    /// payload spells, and it is why this is not the same field as the one a line reader
    /// would fill.
    pub documentation: Option<String>,
    /// What the item declares beyond its name, in the payload's vocabulary, or `None` when
    /// the form has nothing to describe.
    pub shape: Option<String>,
}

impl SyntaxItem
{
    /// The item's name qualified by its syntactic nesting.
    #[must_use]
    pub fn Qualified_Name(&self) -> String
    {
        if self.scope.is_empty()
        {
            return self.name.clone();
        }

        return format!("{}::{}", self.scope.join("::"), self.name);
    }
}

/// Everything one file says on its face.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SyntaxFacts
{
    /// Items in source order.
    pub items: Vec<SyntaxItem>,
    /// Places where the parse tree ends and an unexpanded token stream begins.
    ///
    /// A **lower bound**, and the reason completeness is
    /// [`nomos_contracts::Assurance::Unknown`]. Macro invocations and `derive`
    /// attributes are counted because they are syntactically identifiable; an attribute
    /// macro like `#[tokio::main]` is not, because telling it from `#[allow]` requires
    /// resolving the path — the thing this provider does not do.
    ///
    /// Reported rather than hidden so that a caller reading "3 items" can see whether
    /// the file also had 40 places those items could have been generated from.
    pub unexpanded: u32,
}

impl SyntaxFacts
{
    /// Whether the file declared nothing at all.
    ///
    /// A real answer for a file that is empty or entirely comments, and never the answer
    /// for a file that failed to parse — that is [`Reading::Unparseable`], a different
    /// variant reached by a different path.
    #[must_use]
    pub fn Declares_Nothing(&self) -> bool
    {
        return self.items.is_empty();
    }
}

/// Why a file could not be read.
///
/// Carries where, because the point of this variant is that somebody can act on it. A
/// corpus walk that reports "412 unparseable" and cannot say which or where has produced
/// a number nobody can do anything with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseFailure
{
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl core::fmt::Display for ParseFailure
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(
            formatter,
            "line {}, column {}: {}",
            self.line, self.column, self.message
        );
    }
}

impl std::error::Error for ParseFailure {}

/// The result of reading one recognized file.
///
/// Two variants and no third. There is deliberately no `Reading::Empty` and no
/// `impl Default`: a caller that wants to know whether a file declared nothing must ask
/// [`SyntaxFacts::Declares_Nothing`], which is only reachable through
/// [`Reading::Parsed`] — so "the file has no items" is a sentence that can only be said
/// about a file that was successfully read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading
{
    Parsed(SyntaxFacts),
    Unparseable(ParseFailure),
}

/// Reads Rust source.
///
/// Takes the text, not a path. The signature is where
/// [`nomos_contracts::IncrementalGranularity::File`] stops being a claim: this function
/// has no way to reach a second file, so a fact it produces cannot depend on one, and a
/// change elsewhere cannot invalidate it.
#[must_use]
pub fn Read_Source(source: &str) -> Reading
{
    let file = match syn::parse_file(source)
    {
        Ok(file) => file,
        Err(error) =>
        {
            let at = error.span().start();

            return Reading::Unparseable(ParseFailure {
                line: at.line,
                column: at.column,
                message: error.to_string(),
            });
        }
    };

    let mut walk = Walk::New();
    walk.visit_file(&file);

    return Reading::Parsed(SyntaxFacts {
        items: walk.items,
        unexpanded: walk.unexpanded,
    });
}

/// The walk that turns a parsed file into items.
///
/// A visitor rather than a hand-rolled recursion over `syn::Item`, because
/// [`SyntaxFacts::unexpanded`] has to count macro invocations inside function bodies —
/// and a recursion that only descends through items would report zero for a file whose
/// every function body is a macro, which is the exact case the count exists to expose.
struct Walk
{
    items: Vec<SyntaxItem>,
    scope: Vec<String>,
    unexpanded: u32,
}

impl Walk
{
    fn New() -> Self
    {
        return Self {
            items: Vec::new(),
            scope: Vec::new(),
            unexpanded: 0,
        };
    }

    /// Records one declaration, with what this provider observed about it.
    ///
    /// `shape` is `None` where the form has no shape to describe rather than where none
    /// could be seen. This provider parses, so everything it does not record is an absence
    /// it looked for — the distinction the payload spells `.` rather than `-`, and the
    /// whole reason `nomos.syntax.items.v2` exists.
    fn Record(&mut self, declared: Declared, attributes: &[syn::Attribute])
    {
        let ordinal = u32::try_from(self.items.len()).unwrap_or(u32::MAX);

        self.items.push(SyntaxItem {
            ordinal,
            kind: declared.kind,
            scope: self.scope.clone(),
            name: declared.name,
            visibility: declared.visibility,
            documentation: Documentation(attributes),
            shape: declared.shape,
        });
    }

    fn Record_Use_Tree(
        &mut self,
        tree: &syn::UseTree,
        visibility: &Visibility,
        attributes: &[syn::Attribute],
    )
    {
        match tree
        {
            // The prefix is walked through rather than recorded. `use std::collections::
            // HashMap` introduces one name into this file and it is `HashMap`; recording
            // `std` and `collections` as well would be reporting declarations the file
            // does not make.
            syn::UseTree::Path(path) => self.Record_Use_Tree(&path.tree, visibility, attributes),
            syn::UseTree::Group(group) =>
            {
                for branch in &group.items
                {
                    self.Record_Use_Tree(branch, visibility, attributes);
                }
            }
            leaf =>
            {
                let bound = Bound_By(leaf);
                self.Record(
            Declared {
                kind: ItemKind::Use,
                name: bound,
                visibility: visibility.clone(),
                shape: None,
            },
            attributes,
        );
            }
        }
    }
}

/// One declaration, as the walk saw it.
///
/// `shape` is `None` where the form has no shape to describe rather than where none could
/// be seen. This provider parses, so everything it does not record is an absence it looked
/// for — the distinction the payload spells `.` rather than `-`, and the whole reason
/// `nomos.syntax.items.v2` exists.
struct Declared
{
    kind: ItemKind,
    name: String,
    visibility: Visibility,
    shape: Option<String>,
}

/// Whether an `impl` block serves a trait or is inherent.
///
/// It is the only thing that tells two `impl` blocks for one type apart. A member of
/// `impl Display for Table` carries the same qualified name as a member of `impl Table` and
/// does not belong to `Table` the same way, which is a distinction a consumer cannot
/// recover from any other field.
fn Impl_Shape(serves_a_trait: bool) -> String
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
fn Bound_By(leaf: &syn::UseTree) -> String
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
fn Type_Shape(declared: &syn::Type) -> String
{
    return match declared
    {
        syn::Type::Reference(reference) => Type_Shape(&reference.elem),
        syn::Type::Slice(_) | syn::Type::Array(_) => nomos_cap_syntax::SLICE.to_owned(),
        _ => nomos_cap_syntax::VALUE.to_owned(),
    };
}

/// The shape a function of this many declared parameters has.
///
/// The receiver counts, because it is a declared parameter and because the distinction a
/// consumer wants — `fn All()` against `fn All(&self)` — is exactly the one that vanishes
/// if it does not.
fn Function_Shape(arity: usize) -> String
{
    return nomos_cap_syntax::Function_Shape(arity);
}

/// The item's documentation, as one string, or `None` when it has none.
///
/// `///` is `#[doc]` after parsing, so both spellings are read and neither has to be
/// recognised as text. The lines are joined with newlines rather than flattened: a consumer
/// matching a claim written on one line of a paragraph needs the paragraph as the author
/// wrote it, and the payload escapes the newlines rather than losing them.
fn Documentation(attributes: &[syn::Attribute]) -> Option<String>
{
    let lines: Vec<String> = attributes.iter().filter_map(Doc_Line).collect();

    if lines.is_empty()
    {
        return None;
    }

    return Some(lines.join("\n"));
}

/// One `#[doc = "..."]` attribute's text, or nothing when the attribute is something else.
fn Doc_Line(attribute: &syn::Attribute) -> Option<String>
{
    if !attribute.path().is_ident("doc")
    {
        return None;
    }

    let syn::Meta::NameValue(pair) = &attribute.meta
    else
    {
        return None;
    };
    let syn::Expr::Lit(literal) = &pair.value
    else
    {
        return None;
    };
    let syn::Lit::Str(text) = &literal.lit
    else
    {
        return None;
    };

    return Some(text.value());
}

impl<'ast> Visit<'ast> for Walk
{
    fn visit_item_const(&mut self, node: &'ast syn::ItemConst)
    {
        self.Record(
            Declared {
                kind: ItemKind::Constant,
                name: node.ident.to_string(),
                visibility: Visibility::Of(&node.vis),
                shape: Some(Type_Shape(&node.ty)),
            },
            &node.attrs,
        );
        syn::visit::visit_item_const(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum)
    {
        self.Record(
            Declared {
                kind: ItemKind::Enum,
                name: node.ident.to_string(),
                visibility: Visibility::Of(&node.vis),
                shape: None,
            },
            &node.attrs,
        );
        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_extern_crate(&mut self, node: &'ast syn::ItemExternCrate)
    {
        self.Record(
            Declared {
                kind: ItemKind::ExternCrate,
                name: node.ident.to_string(),
                visibility: Visibility::Of(&node.vis),
                shape: None,
            },
            &node.attrs,
        );
        syn::visit::visit_item_extern_crate(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn)
    {
        self.Record(
            Declared {
                kind: ItemKind::Function,
                name: node.sig.ident.to_string(),
                visibility: Visibility::Of(&node.vis),
                shape: Some(Function_Shape(node.sig.inputs.len())),
            },
            &node.attrs,
        );
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_foreign_mod(&mut self, node: &'ast syn::ItemForeignMod)
    {
        // An `extern "C"` block has no name of its own. The ABI is what distinguishes one
        // from another in the same file, so it is what the item is called.
        let abi = node
            .abi
            .name
            .as_ref()
            .map_or_else(|| return "extern".to_owned(), |name| return name.value());

        self.Record(
            Declared {
                kind: ItemKind::ForeignModule,
                name: abi,
                visibility: Visibility::NotApplicable,
                shape: None,
            },
            &node.attrs,
        );

        syn::visit::visit_item_foreign_mod(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl)
    {
        // The self type's head, as written. `impl Display for Foo` records `Foo`, and
        // makes no claim about which `Display` — that is the trait resolution this
        // provider does not do, and a file declaring its own `trait Display` would make
        // any such claim wrong.
        let name = Type_Head(&node.self_ty);

        let shape = Impl_Shape(node.trait_.is_some());

        self.Record(
            Declared {
                kind: ItemKind::Implementation,
                name: name.clone(),
                visibility: Visibility::NotApplicable,
                shape: Some(shape),
            },
            &node.attrs,
        );
        self.scope.push(name);
        syn::visit::visit_item_impl(self, node);
        self.scope.pop();
    }

    fn visit_item_macro(&mut self, node: &'ast syn::ItemMacro)
    {
        // `macro_rules! foo` names `foo`; a bare invocation at item position does not
        // define anything, and the macro's own path is the only name it has.
        let name = node.ident.as_ref().map_or_else(
            || return Path_As_Written(&node.mac.path),
            |ident| return ident.to_string(),
        );

        self.Record(
            Declared {
                kind: ItemKind::MacroDefinition,
                name: name,
                visibility: Visibility::NotApplicable,
                shape: None,
            },
            &node.attrs,
        );
        syn::visit::visit_item_macro(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod)
    {
        self.Record(
            Declared {
                kind: ItemKind::Module,
                name: node.ident.to_string(),
                visibility: Visibility::Of(&node.vis),
                shape: None,
            },
            &node.attrs,
        );
        self.scope.push(node.ident.to_string());
        syn::visit::visit_item_mod(self, node);
        self.scope.pop();
    }

    fn visit_item_static(&mut self, node: &'ast syn::ItemStatic)
    {
        self.Record(
            Declared {
                kind: ItemKind::Static,
                name: node.ident.to_string(),
                visibility: Visibility::Of(&node.vis),
                shape: Some(Type_Shape(&node.ty)),
            },
            &node.attrs,
        );
        syn::visit::visit_item_static(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct)
    {
        self.Record(
            Declared {
                kind: ItemKind::Struct,
                name: node.ident.to_string(),
                visibility: Visibility::Of(&node.vis),
                shape: None,
            },
            &node.attrs,
        );
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait)
    {
        self.Record(
            Declared {
                kind: ItemKind::Trait,
                name: node.ident.to_string(),
                visibility: Visibility::Of(&node.vis),
                shape: None,
            },
            &node.attrs,
        );
        self.scope.push(node.ident.to_string());
        syn::visit::visit_item_trait(self, node);
        self.scope.pop();
    }

    fn visit_item_trait_alias(&mut self, node: &'ast syn::ItemTraitAlias)
    {
        self.Record(
            Declared {
                kind: ItemKind::TraitAlias,
                name: node.ident.to_string(),
                visibility: Visibility::Of(&node.vis),
                shape: None,
            },
            &node.attrs,
        );
        syn::visit::visit_item_trait_alias(self, node);
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType)
    {
        self.Record(
            Declared {
                kind: ItemKind::TypeAlias,
                name: node.ident.to_string(),
                visibility: Visibility::Of(&node.vis),
                shape: None,
            },
            &node.attrs,
        );
        syn::visit::visit_item_type(self, node);
    }

    fn visit_item_union(&mut self, node: &'ast syn::ItemUnion)
    {
        self.Record(
            Declared {
                kind: ItemKind::Union,
                name: node.ident.to_string(),
                visibility: Visibility::Of(&node.vis),
                shape: None,
            },
            &node.attrs,
        );
        syn::visit::visit_item_union(self, node);
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse)
    {
        let visibility = Visibility::Of(&node.vis);

        self.Record_Use_Tree(&node.tree, &visibility, &node.attrs);
        syn::visit::visit_item_use(self, node);
    }

    // Members of an `impl` or a `trait`. They are items — they have names, and in an
    // `impl` they have visibility — and omitting them would mean this provider reported
    // the type and nothing about it.

    fn visit_impl_item_const(&mut self, node: &'ast syn::ImplItemConst)
    {
        self.Record(
            Declared {
                kind: ItemKind::Constant,
                name: node.ident.to_string(),
                visibility: Visibility::Of(&node.vis),
                shape: Some(Type_Shape(&node.ty)),
            },
            &node.attrs,
        );
        syn::visit::visit_impl_item_const(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn)
    {
        self.Record(
            Declared {
                kind: ItemKind::Function,
                name: node.sig.ident.to_string(),
                visibility: Visibility::Of(&node.vis),
                shape: Some(Function_Shape(node.sig.inputs.len())),
            },
            &node.attrs,
        );
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_impl_item_type(&mut self, node: &'ast syn::ImplItemType)
    {
        self.Record(
            Declared {
                kind: ItemKind::TypeAlias,
                name: node.ident.to_string(),
                visibility: Visibility::Of(&node.vis),
                shape: None,
            },
            &node.attrs,
        );
        syn::visit::visit_impl_item_type(self, node);
    }

    fn visit_trait_item_const(&mut self, node: &'ast syn::TraitItemConst)
    {
        self.Record(
            Declared {
                kind: ItemKind::Constant,
                name: node.ident.to_string(),
                visibility: Visibility::NotApplicable,
                shape: Some(Type_Shape(&node.ty)),
            },
            &node.attrs,
        );
        syn::visit::visit_trait_item_const(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn)
    {
        self.Record(
            Declared {
                kind: ItemKind::Function,
                name: node.sig.ident.to_string(),
                visibility: Visibility::NotApplicable,
                shape: Some(Function_Shape(node.sig.inputs.len())),
            },
            &node.attrs,
        );
        syn::visit::visit_trait_item_fn(self, node);
    }

    fn visit_trait_item_type(&mut self, node: &'ast syn::TraitItemType)
    {
        self.Record(
            Declared {
                kind: ItemKind::TypeAlias,
                name: node.ident.to_string(),
                visibility: Visibility::NotApplicable,
                shape: None,
            },
            &node.attrs,
        );
        syn::visit::visit_trait_item_type(self, node);
    }

    fn visit_foreign_item_fn(&mut self, node: &'ast syn::ForeignItemFn)
    {
        self.Record(
            Declared {
                kind: ItemKind::Function,
                name: node.sig.ident.to_string(),
                visibility: Visibility::Of(&node.vis),
                shape: Some(Function_Shape(node.sig.inputs.len())),
            },
            &node.attrs,
        );
        syn::visit::visit_foreign_item_fn(self, node);
    }

    fn visit_foreign_item_static(&mut self, node: &'ast syn::ForeignItemStatic)
    {
        self.Record(
            Declared {
                kind: ItemKind::Static,
                name: node.ident.to_string(),
                visibility: Visibility::Of(&node.vis),
                shape: None,
            },
            &node.attrs,
        );
        syn::visit::visit_foreign_item_static(self, node);
    }

    /// Every place the parse tree ends in tokens. Anywhere — item position, a type, an
    /// expression inside a function body.
    fn visit_macro(&mut self, node: &'ast syn::Macro)
    {
        self.unexpanded = self.unexpanded.saturating_add(1);
        syn::visit::visit_macro(self, node);
    }

    /// `#[derive(…)]` generates items, and is the one attribute macro form this provider
    /// can identify without resolving a path.
    fn visit_attribute(&mut self, node: &'ast syn::Attribute)
    {
        if node.path().is_ident("derive")
        {
            self.unexpanded = self.unexpanded.saturating_add(1);
        }
        syn::visit::visit_attribute(self, node);
    }
}

/// A path rendered as written, without the generic arguments.
fn Path_As_Written(path: &syn::Path) -> String
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
fn Type_Head(kind: &syn::Type) -> String
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

    fn Parsed(source: &str) -> SyntaxFacts
    {
        return match Read_Source(source)
        {
            Reading::Parsed(facts) => facts,
            Reading::Unparseable(failure) => panic!("expected a parse: {failure}"),
        };
    }

    fn Names(source: &str) -> Vec<String>
    {
        return Parsed(source)
            .items
            .iter()
            .map(SyntaxItem::Qualified_Name)
            .collect();
    }

    #[test]
    fn Test_Items_Should_Be_Named_By_Their_Syntactic_Nesting()
    {
        let names = Names(
            "mod outer { pub mod inner { pub fn deep() {} } }\n\
             struct Top;\n\
             impl Top { fn method(&self) {} }\n",
        );

        assert_eq!(
            names,
            vec![
                "outer",
                "outer::inner",
                "outer::inner::deep",
                "Top",
                "Top",
                "Top::method",
            ]
        );
    }

    #[test]
    fn Test_Ordinals_Should_Be_Dense_And_Zero_Based()
    {
        let facts = Parsed("fn a() {}\nfn b() {}\nfn c() {}\n");

        let ordinals: Vec<u32> = facts.items.iter().map(|item| return item.ordinal).collect();

        assert_eq!(ordinals, vec![0, 1, 2]);
    }

    #[test]
    fn Test_Visibility_Should_Be_Recorded_As_Declared()
    {
        let facts = Parsed(
            "pub fn exported() {}\n\
             fn hidden() {}\n\
             pub(crate) fn within() {}\n\
             pub(in some::place) fn nested() {}\n\
             trait Contract { fn required(&self); }\n",
        );

        let visibilities: Vec<String> = facts
            .items
            .iter()
            .map(|item| return item.visibility.Label())
            .collect();

        assert_eq!(
            visibilities,
            vec![
                "Public",
                "Private",
                "Restricted(crate)",
                "Restricted(in some::place)",
                "Private",
                "NotApplicable",
            ],
            "a trait member declares no visibility, and saying `Private` would be \
             recording a keyword the source does not contain"
        );
    }

    /// A `use` introduces the binding it introduces, and nothing else. The prefix
    /// segments are not declarations this file makes.
    #[test]
    fn Test_A_Use_Should_Record_The_Binding_It_Introduces()
    {
        assert_eq!(
            Names("use std::collections::HashMap;\n"),
            vec!["HashMap".to_owned()]
        );
        assert_eq!(
            Names("use std::collections::HashMap as Map;\n"),
            vec!["Map".to_owned()],
            "the file now has `Map`; what `Map` refers to is a resolution away"
        );
        assert_eq!(
            Names("use std::collections::{HashMap, BTreeSet as Ordered};\n"),
            vec!["HashMap".to_owned(), "Ordered".to_owned()]
        );
        assert_eq!(Names("use std::fmt::*;\n"), vec!["*".to_owned()]);
    }

    /// A file that declares nothing parses. This is the variant that must never be how
    /// a failure looks.
    #[test]
    fn Test_A_File_That_Declares_Nothing_Should_Parse()
    {
        for source in ["", "\n\n", "// nothing here\n", "//! only a doc comment\n"]
        {
            let facts = Parsed(source);

            assert!(facts.Declares_Nothing(), "`{source:?}` declares nothing");
            assert_eq!(facts.unexpanded, 0);
        }
    }

    /// The property the whole outcome type exists for.
    #[test]
    fn Test_Broken_Source_Should_Be_Unparseable_Rather_Than_Empty()
    {
        for source in [
            "fn unclosed( {",
            "struct S { field: }",
            "this is not rust at all",
            "fn f() { let x = ; }",
        ]
        {
            match Read_Source(source)
            {
                Reading::Unparseable(failure) =>
                {
                    assert!(
                        failure.line >= 1,
                        "a refusal must name where it refused: {failure:?}"
                    );
                    assert!(!failure.message.is_empty());
                }
                Reading::Parsed(facts) =>
                {
                    panic!("`{source}` parsed to {} items", facts.items.len())
                }
            }
        }
    }

    /// A byte order mark belongs at offset zero or nowhere, and the difference decides
    /// whether a file is source or damage.
    ///
    /// Found by the corpus walk rather than reasoned about: seven files under
    /// `F:/repos/xvpe` carry a `U+FEFF` at byte 15, immediately after a `use super::*;`
    /// somebody prepended to a file that already began with one. They are the only seven
    /// refusals in 7,580 files, and rustc will not compile them either — so the refusal
    /// is this provider agreeing with the compiler, not falling behind it.
    ///
    /// D-131 records the same distinction for the specification corpus, where the mark
    /// belongs to the front matter fence. It is the same rule twice because it is a fact
    /// about byte order marks rather than about either format.
    #[test]
    fn Test_A_Byte_Order_Mark_Should_Be_Leading_Or_Refused()
    {
        let facts = Parsed("\u{feff}pub fn after_the_mark() {}\n");

        assert_eq!(
            facts.items.first().map(|item| return item.name.clone()),
            Some("after_the_mark".to_owned()),
            "a leading mark is an encoding announcement and the file is ordinary Rust"
        );

        let stray = "use super::*;\n\n\u{feff}//! documentation\npub fn hidden() {}\n";

        match Read_Source(stray)
        {
            Reading::Unparseable(failure) => assert_eq!(
                failure.line, 3,
                "the refusal names the line the mark is on: {failure}"
            ),
            Reading::Parsed(facts) => panic!(
                "a mark in the middle of a file is not whitespace and this does not \
                 compile; reading it as {} items would report a broken file as sound",
                facts.items.len()
            ),
        }
    }

    /// The measured reason completeness is `Unknown`, and the negative control for the
    /// claim that this provider is honest about it. If the walk stopped at item level,
    /// the count inside the function body would be zero and the declared weakness would
    /// be undetectable from the output.
    #[test]
    fn Test_Unexpanded_Regions_Should_Be_Counted_Wherever_They_Are()
    {
        let facts = Parsed(
            "#[derive(Clone, Debug)]\n\
             pub struct Held;\n\
             fn body() { println!(\"one\"); vec![1, 2]; }\n\
             generated_items!();\n",
        );

        assert_eq!(
            facts.unexpanded, 4,
            "one derive, two invocations inside a body, one at item position: {:?}",
            facts.items
        );
    }
}

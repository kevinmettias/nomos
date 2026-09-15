//! One pass over a parsed file, recording every declaration it meets.

use super::{Item, Documentation_Of_Attributes, Visibility, Bound_By, ItemKind, Visit, Type_Shape, Function_Shape, Struct_Shape, Type_Head, Impl_Shape, Path_As_Written};

/// The walk that turns a parsed file into items.
///
/// A visitor rather than a hand-rolled recursion over `syn::Item`, because
/// [`Facts::unexpanded`] has to count macro invocations inside function bodies —
/// and a recursion that only descends through items would report zero for a file whose
/// every function body is a macro, which is the exact case the count exists to expose.
pub(super) struct Walk
{
    pub(super) items: Vec<Item>,
    scope: Vec<String>,
    pub(super) unexpanded: u32,
}

impl Walk
{
    pub(super) fn New() -> Self
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

        self.items.push(Item {
            ordinal,
            kind: declared.kind,
            scope: self.scope.clone(),
            name: declared.name,
            visibility: declared.visibility,
            documentation: Documentation_Of_Attributes(attributes),
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
pub(super) struct Declared
{
    kind: ItemKind,
    name: String,
    visibility: Visibility,
    shape: Option<String>,
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

        let shape = Impl_Shape(node);

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
                name,
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
                shape: Struct_Shape(&node.fields),
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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_New_Should_Produce_An_Empty_Walk_With_No_Scope()
    {
        let walk = Walk::New();

        assert!(walk.items.is_empty());
        assert_eq!(walk.unexpanded, 0);
        assert!(walk.scope.is_empty());
    }
}

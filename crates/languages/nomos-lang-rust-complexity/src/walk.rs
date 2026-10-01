//! The walk over one parse tree that counts each function's decision points.

use nomos_cap_complexity::FunctionComplexity;
use syn::visit::{self, Visit};

/// One pass over a file's parse tree.
///
/// A function is recorded when the walk enters it, at complexity one, and every decision point
/// met until the walk leaves it is added to the innermost function still open. That is what makes
/// a closure count into its function and a nested function count into itself.
pub(crate) struct Walk
{
    /// Every function met so far, in the order each one opened.
    pub(crate) functions: Vec<FunctionComplexity>,
    /// The module, `impl`, trait and function names enclosing the walk, outermost first -- what
    /// qualifies a function's name so two of one name in one file read apart.
    scope: Vec<String>,
    /// Where in [`Self::functions`] each function still open sits, innermost last.
    open: Vec<usize>,
}

impl Walk
{
    pub(crate) fn New() -> Self
    {
        return Walk { functions: Vec::new(), scope: Vec::new(), open: Vec::new() };
    }

    /// Records a function opening at `ident`, with the one path every function has.
    fn Open_Function(&mut self, ident: &syn::Ident)
    {
        let name = ident.to_string();
        let function = if self.scope.is_empty() { name.clone() } else { format!("{}::{name}", self.scope.join("::")) };

        self.open.push(self.functions.len());
        self.functions.push(FunctionComplexity { function, line: ident.span().start().line, complexity: 1 });
        self.scope.push(name);
    }

    fn Close_Function(&mut self)
    {
        self.open.pop();
        self.scope.pop();
    }

    /// Adds `decisions` to the innermost open function. A decision met outside every function --
    /// in a `const` initializer, say -- belongs to no subject this capability measures.
    fn Count(&mut self, decisions: usize)
    {
        let Some(index) = self.open.last().copied()
        else
        {
            return;
        };

        if let Some(function) = self.functions.get_mut(index)
        {
            function.complexity = function.complexity.saturating_add(decisions);
        }
    }

    /// Walks `visit_inner` with `segment` on the scope, so every function inside is named under it.
    fn Within(&mut self, segment: String, visit_inner: impl FnOnce(&mut Self))
    {
        self.scope.push(segment);
        visit_inner(self);
        self.scope.pop();
    }
}

impl<'ast> Visit<'ast> for Walk
{
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn)
    {
        self.Open_Function(&node.sig.ident);
        visit::visit_item_fn(self, node);
        self.Close_Function();
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn)
    {
        self.Open_Function(&node.sig.ident);
        visit::visit_impl_item_fn(self, node);
        self.Close_Function();
    }

    /// A trait method is a subject only where it has a body; a bare signature has no path through
    /// it to count.
    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn)
    {
        if node.default.is_none()
        {
            visit::visit_trait_item_fn(self, node);
            return;
        }

        self.Open_Function(&node.sig.ident);
        visit::visit_trait_item_fn(self, node);
        self.Close_Function();
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod)
    {
        self.Within(node.ident.to_string(), |walk| visit::visit_item_mod(walk, node));
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl)
    {
        self.Within(Self_Type_Name(&node.self_ty), |walk| visit::visit_item_impl(walk, node));
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait)
    {
        self.Within(node.ident.to_string(), |walk| visit::visit_item_trait(walk, node));
    }

    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf)
    {
        self.Count(1);
        visit::visit_expr_if(self, node);
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile)
    {
        self.Count(1);
        visit::visit_expr_while(self, node);
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop)
    {
        self.Count(1);
        visit::visit_expr_for_loop(self, node);
    }

    /// A `match` of `n` arms is `n` paths, so `n - 1` decisions; a guard is one more, since an arm
    /// whose guard fails falls through to the next.
    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch)
    {
        let guards = node.arms.iter().filter(|arm| return arm.guard.is_some()).count();
        self.Count(node.arms.len().saturating_sub(1).saturating_add(guards));
        visit::visit_expr_match(self, node);
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary)
    {
        if matches!(node.op, syn::BinOp::And(_) | syn::BinOp::Or(_))
        {
            self.Count(1);
        }
        visit::visit_expr_binary(self, node);
    }

    fn visit_expr_try(&mut self, node: &'ast syn::ExprTry)
    {
        self.Count(1);
        visit::visit_expr_try(self, node);
    }

    /// `let pattern = value else { ... };` is a branch: the pattern either binds or the `else` runs.
    fn visit_local(&mut self, node: &'ast syn::Local)
    {
        if node.init.as_ref().is_some_and(|init| return init.diverge.is_some())
        {
            self.Count(1);
        }
        visit::visit_local(self, node);
    }
}

/// The name an `impl` block qualifies its methods with: its self type's own last segment, or
/// `impl` for a self type that is not a path -- a reference or a tuple -- since rendering one
/// would need `syn`'s printing, which nothing else here needs.
fn Self_Type_Name(self_ty: &syn::Type) -> String
{
    let syn::Type::Path(path) = self_ty
    else
    {
        return "impl".to_owned();
    };

    return path.path.segments.last().map_or_else(|| return "impl".to_owned(), |segment| return segment.ident.to_string());
}

#[cfg(test)]
mod tests;

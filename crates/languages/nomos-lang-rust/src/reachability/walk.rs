//! One pass over a parsed file, recording every `Err(applicability) => <body>` match arm
//! whose body is one of [`ArmShape`]'s four obvious wrong shapes.
//!
//! A second, independent [`Visit`] implementation rather than an extension of
//! `crate::syntax::walk::Walk` — that walker's own module doc states its scope as items,
//! not expressions, and folding a second capability's pattern-matching into it would widen
//! a claim `nomos.cap.syntax.items`' contract does not make.

use nomos_cap_controlflow::{ArmShape, ReachabilitySite};
use syn::visit::Visit;

pub(super) struct Walk
{
    pub(super) sites: Vec<ReachabilitySite>,
    current_function: Option<String>,
}

impl Walk
{
    pub(super) fn New() -> Self
    {
        return Self {
            sites: Vec::new(),
            current_function: None,
        };
    }

    /// One `match` arm's own contribution: recorded only if its pattern binds
    /// `Err(applicability)` and its body is one of [`ArmShape`]'s four obvious wrong shapes.
    fn Visit_Match_Arm(&mut self, arm: &syn::Arm)
    {
        let Some(binding) = Err_Binding(&arm.pat)
        else
        {
            return;
        };

        if binding != "applicability"
        {
            return;
        }

        let Some(shape) = Classify_Arm_Body(&arm.body)
        else
        {
            return;
        };

        self.Record(&binding, shape);
    }

    fn Record(&mut self, binding: &str, shape: ArmShape)
    {
        // A site outside any function has nowhere to name. `syn` never hands a `match`
        // arm to this visitor at file scope — a `match` is an expression, and Rust has no
        // free-standing expression outside a function body — so this is unreachable in
        // practice and left as a silent no-op rather than a panic, the same caution
        // `crate::syntax::walk::Walk::Record_Use_Tree` takes for shapes its own grammar
        // does not admit.
        let Some(function) = self.current_function.clone()
        else
        {
            return;
        };

        self.sites.push(ReachabilitySite {
            function,
            binding: binding.to_owned(),
            shape,
        });
    }
}

/// The identifier one `Err(...)` tuple-struct pattern binds, if it binds exactly one and
/// its path is `Err`.
///
/// Not resolved: a `use my_module::Result::Err as Err;` alias would defeat this, the same
/// way any path-based recognition at `Syntactic` tier can be defeated by a rename this
/// provider has no way to see through.
fn Err_Binding(pattern: &syn::Pat) -> Option<String>
{
    let syn::Pat::TupleStruct(tuple) = pattern
    else
    {
        return None;
    };

    if !tuple.path.segments.last().is_some_and(|segment| return segment.ident == "Err")
    {
        return None;
    }

    let [syn::Pat::Ident(bound)] = tuple.elems.iter().collect::<Vec<_>>().as_slice()
    else
    {
        return None;
    };

    return Some(bound.ident.to_string());
}

/// Which of [`ArmShape`]'s four obvious wrong shapes `body` is, or `None` for anything
/// else — including a body this walker cannot see past, which is not the same claim as a
/// body it looked at and found correct.
fn Classify_Arm_Body(body: &syn::Expr) -> Option<ArmShape>
{
    if let syn::Expr::Block(block) = body
    {
        if block.block.stmts.is_empty()
        {
            return Some(ArmShape::Empty);
        }
    }

    return match Tail_Expression(body)
    {
        syn::Expr::Continue(_) => Some(ArmShape::BareContinue),
        syn::Expr::Return(syn::ExprReturn { expr: None, .. }) => Some(ArmShape::BareReturn),
        syn::Expr::Call(call) => match &*call.func
        {
            syn::Expr::Path(path) if path.path.segments.last().is_some_and(|segment| return segment.ident == "Ok") =>
            {
                Some(ArmShape::TailOk)
            }
            _ => None,
        },
        _ => None,
    };
}

/// One arm body, unwrapped past a single-statement block down to the expression that
/// actually decides its shape.
///
/// `{ continue; }` and `continue` are the same arm to a reader of this rule's own three
/// real instances — none of them write the bare form — so both must classify alike.
fn Tail_Expression(expression: &syn::Expr) -> &syn::Expr
{
    if let syn::Expr::Block(block) = expression
    {
        if let [syn::Stmt::Expr(inner, _)] = block.block.stmts.as_slice()
        {
            return Tail_Expression(inner);
        }
    }

    return expression;
}

impl<'ast> Visit<'ast> for Walk
{
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn)
    {
        let outer = self.current_function.replace(node.sig.ident.to_string());
        syn::visit::visit_item_fn(self, node);
        self.current_function = outer;
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn)
    {
        let outer = self.current_function.replace(node.sig.ident.to_string());
        syn::visit::visit_impl_item_fn(self, node);
        self.current_function = outer;
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn)
    {
        let outer = self.current_function.replace(node.sig.ident.to_string());
        syn::visit::visit_trait_item_fn(self, node);
        self.current_function = outer;
    }

    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch)
    {
        for arm in &node.arms
        {
            self.Visit_Match_Arm(arm);
        }

        syn::visit::visit_expr_match(self, node);
    }
}

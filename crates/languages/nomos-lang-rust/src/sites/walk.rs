//! The walk that finds labeled jumps and resolves each against what encloses it.

use nomos_cap_syntax::LabeledJump;
use proc_macro2::Span;
use syn::visit::{self, Visit};

/// One construct a jump stands inside: a loop, or a labeled block, with the label it carries.
///
/// A loop is pushed whether or not it is labeled, because where a bare `break` lands is the
/// innermost loop of all; a block is pushed only when labeled, because only then can a jump name
/// it. Rust catches a bare `break` in a loop and in nothing else -- not a `match`, not an `if`, not
/// a block -- so "where a bare jump would land" and "what a loop is" are one question here.
struct Enclosure
{
    label: Option<String>,
    label_line: usize,
    is_loop: bool,
}

/// The walk's state: the constructs enclosing the node being visited, and the jumps found so far.
pub(super) struct Walk
{
    stack: Vec<Enclosure>,
    pub(super) jumps: Vec<LabeledJump>,
}

impl Walk
{
    pub(super) const fn New() -> Self
    {
        return Self { stack: Vec::new(), jumps: Vec::new() };
    }

    /// Visits what `within` visits with `enclosure` on top of the stack, and takes it off after.
    fn Inside(&mut self, enclosure: Enclosure, within: impl FnOnce(&mut Self))
    {
        self.stack.push(enclosure);
        within(self);
        self.stack.pop();
    }

    /// Visits what `within` visits on an empty stack, and restores the stack after: a label never
    /// crosses an item or a closure, so neither do the constructs a jump could name.
    fn Fresh(&mut self, within: impl FnOnce(&mut Self))
    {
        let outer = core::mem::take(&mut self.stack);
        within(self);
        self.stack = outer;
    }

    /// Records the jump `keyword` at `at` if its label resolves to a loop enclosing it.
    ///
    /// A label resolving to nothing is a jump out of a closure, which does not compile, and a label
    /// resolving to a block abandons no iteration; neither is a site of this kind.
    fn Record(&mut self, keyword: &str, at: Span, label: &syn::Lifetime)
    {
        let spelled = Spelled(label);
        let Some(target) = self.stack.iter().rposition(|enclosure| return enclosure.label.as_deref() == Some(spelled.as_str()))
        else
        {
            return;
        };
        let Some(enclosure) = self.stack.get(target).filter(|enclosure| return enclosure.is_loop)
        else
        {
            return;
        };

        let innermost_loop = self.stack.iter().rposition(|enclosing| return enclosing.is_loop);
        self.jumps.push(LabeledJump {
            line: at.start().line,
            keyword: keyword.to_owned(),
            text: format!("{keyword} {spelled}"),
            label: spelled,
            label_line: enclosure.label_line,
            has_same_target_unlabeled: innermost_loop == Some(target),
            // Always false in Rust, and not because the walk does not look: nothing but a loop
            // catches a bare `break` here, so no jump stands where the bare form means something
            // else, and a Rust label naming its innermost loop gets no shelter from that exemption.
            is_inside_break_capture: false,
        });
    }
}

/// The enclosure a loop's optional label makes.
fn Loop(label: Option<&syn::Label>) -> Enclosure
{
    return Enclosure {
        label: label.map(|label| return Spelled(&label.name)),
        label_line: label.map_or(0, |label| return label.name.apostrophe.start().line),
        is_loop: true,
    };
}

/// The label as Rust spells it, apostrophe and all.
fn Spelled(label: &syn::Lifetime) -> String
{
    return format!("'{}", label.ident);
}

impl<'ast> Visit<'ast> for Walk
{
    fn visit_item(&mut self, item: &'ast syn::Item)
    {
        self.Fresh(|walk| visit::visit_item(walk, item));
    }

    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure)
    {
        self.Fresh(|walk| visit::visit_expr_closure(walk, closure));
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop)
    {
        self.Inside(Loop(node.label.as_ref()), |walk| visit::visit_expr_for_loop(walk, node));
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile)
    {
        self.Inside(Loop(node.label.as_ref()), |walk| visit::visit_expr_while(walk, node));
    }

    fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop)
    {
        self.Inside(Loop(node.label.as_ref()), |walk| visit::visit_expr_loop(walk, node));
    }

    fn visit_expr_block(&mut self, node: &'ast syn::ExprBlock)
    {
        let Some(label) = &node.label
        else
        {
            visit::visit_expr_block(self, node);
            return;
        };

        let block = Enclosure { label: Some(Spelled(&label.name)), label_line: label.name.apostrophe.start().line, is_loop: false };
        self.Inside(block, |walk| visit::visit_expr_block(walk, node));
    }

    fn visit_expr_break(&mut self, node: &'ast syn::ExprBreak)
    {
        if let Some(label) = &node.label
        {
            self.Record("break", node.break_token.span, label);
        }
        visit::visit_expr_break(self, node);
    }

    fn visit_expr_continue(&mut self, node: &'ast syn::ExprContinue)
    {
        if let Some(label) = &node.label
        {
            self.Record("continue", node.continue_token.span, label);
        }
        visit::visit_expr_continue(self, node);
    }
}

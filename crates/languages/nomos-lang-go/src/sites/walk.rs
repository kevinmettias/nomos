//! The walk that finds labeled jumps in a Go tree and resolves each against what encloses it.

use nomos_cap_syntax::LabeledJump;
use tree_sitter::Node;

/// One construct a jump stands inside, with the label it carries.
///
/// Two properties, because Go has two kinds that matter and they are not one kind: a loop catches
/// both `break` and `continue`, and a `switch`, type switch or `select` catches `break` and is
/// invisible to `continue`, and is not a loop, so a jump leaving one has left no iteration.
#[derive(Clone)]
pub(super) struct Enclosure
{
    label: Option<String>,
    label_line: usize,
    is_loop: bool,
    is_break_capture: bool,
}

/// A label waiting for the statement it is written on -- Go writes a label on a statement that
/// wraps the loop (`Outer: for ...`), so the loop learns its name from its parent.
pub(super) struct Pending
{
    label: String,
    line: usize,
}

/// The walk's state: the source the tree's spans index, and the jumps found so far.
pub(super) struct Walk<'source>
{
    source: &'source [u8],
    pub(super) jumps: Vec<LabeledJump>,
}

impl<'source> Walk<'source>
{
    pub(super) const fn New(source: &'source [u8]) -> Self
    {
        return Self { source, jumps: Vec::new() };
    }

    /// Visits `node` with the constructs enclosing it and the label pending on it, if any.
    ///
    /// The walk starts at the file, not at its function declarations, because a func literal in a
    /// package-level `var` or a composite literal is code too, and a walk that never opened it would
    /// report a clean file it never read -- the regression the corpus kernel's own tests pin. A func
    /// literal starts a fresh stack: a label cannot be jumped to from inside a closure.
    pub(super) fn Descend(&mut self, node: Node<'_>, stack: &[Enclosure], pending: Option<Pending>)
    {
        match node.kind()
        {
            "labeled_statement" => self.Labeled(node, stack),
            "func_literal" => self.Children(node, &[]),
            "for_statement" => self.Children(node, &Pushed(stack, Enclosure::Of(pending, true))),
            "expression_switch_statement" | "type_switch_statement" | "select_statement" =>
            {
                self.Children(node, &Pushed(stack, Enclosure::Of(pending, false)));
            }
            "break_statement" | "continue_statement" => self.Record(node, stack),
            _ => self.Children(node, stack),
        }
    }

    /// Hands a label to the one statement it is written on, and to nothing else.
    fn Labeled(&mut self, node: Node<'_>, stack: &[Enclosure])
    {
        let label = node.child_by_field_name("label");
        let mut cursor = node.walk();
        let statement = node.named_children(&mut cursor).find(|child| return Some(*child) != label);
        let (Some(label), Some(statement)) = (label, statement)
        else
        {
            return;
        };

        let pending = Pending { label: self.Text(label).to_owned(), line: Line_Of(label) };
        self.Descend(statement, stack, Some(pending));
    }

    fn Children(&mut self, node: Node<'_>, stack: &[Enclosure])
    {
        let mut cursor = node.walk();
        let children: Vec<Node<'_>> = node.named_children(&mut cursor).collect();
        for child in children
        {
            self.Descend(child, stack, None);
        }
    }

    /// Records the jump `node` if it names a label that resolves to a loop enclosing it.
    ///
    /// A jump to a label on a `switch` or a `select` abandons no iteration, and a bare jump names
    /// nothing; neither is a site of this kind. `goto` is its own statement and never reaches here.
    fn Record(&mut self, node: Node<'_>, stack: &[Enclosure])
    {
        let mut cursor = node.walk();
        let Some(label) = node.named_children(&mut cursor).find(|child| return child.kind() == "label_name")
        else
        {
            return;
        };
        let spelled = self.Text(label).to_owned();
        let Some(target) = stack.iter().rposition(|enclosure| return enclosure.label.as_deref() == Some(spelled.as_str()))
        else
        {
            return;
        };
        let Some(enclosure) = stack.get(target).filter(|enclosure| return enclosure.is_loop)
        else
        {
            return;
        };

        let is_continue = node.kind() == "continue_statement";
        let keyword = if is_continue { "continue" } else { "break" };
        self.jumps.push(LabeledJump {
            line: Line_Of(node),
            keyword: keyword.to_owned(),
            text: format!("{keyword} {spelled}"),
            label: spelled,
            label_line: enclosure.label_line,
            has_same_target_unlabeled: Bare_Jump_Lands_At(is_continue, stack) == Some(target),
            is_inside_break_capture: stack.iter().skip(target.saturating_add(1)).any(|inner| return !inner.is_loop && inner.is_break_capture),
        });
    }

    /// The text a node spans.
    fn Text(&self, node: Node<'_>) -> &'source str
    {
        return node.utf8_text(self.source).unwrap_or_default();
    }
}

impl Enclosure
{
    /// The enclosure a loop (`is_loop`) or a break-catching construct makes, carrying the label
    /// pending on it. Both catch a bare `break`.
    fn Of(pending: Option<Pending>, is_loop: bool) -> Self
    {
        let (label, label_line) = pending.map_or((None, 0), |pending| return (Some(pending.label), pending.line));
        return Self { label, label_line, is_loop, is_break_capture: true };
    }
}

/// `stack` with `one` on top, as a copy -- two sibling branches must not share one enclosure list.
fn Pushed(stack: &[Enclosure], one: Enclosure) -> Vec<Enclosure>
{
    let mut pushed = stack.to_vec();
    pushed.push(one);
    return pushed;
}

/// Where the same jump would land with its label deleted: a bare `continue` at the nearest loop, a
/// bare `break` at the nearest loop, `switch` or `select` -- the language fact the rule cannot have.
fn Bare_Jump_Lands_At(is_continue: bool, stack: &[Enclosure]) -> Option<usize>
{
    return stack.iter().rposition(|enclosure| return if is_continue { enclosure.is_loop } else { enclosure.is_break_capture });
}

/// The 1-based line a node starts on.
fn Line_Of(node: Node<'_>) -> usize
{
    return node.start_position().row.saturating_add(1);
}

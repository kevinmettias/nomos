//! Mermaid: one subgraph per section, and one name table across all of them.

use crate::GENERATED_FILE_NOTICE;
use crate::Item;
use crate::Projection;
use crate::Section;
use core::fmt::Write as _;
use std::collections::BTreeMap;

use super::text::Slug_Of_Text;

pub(super) fn Render_Mermaid(projection: &Projection) -> String
{
    let mut out = format!(
        "%% nomos_generated: true\n%% do_not_edit: {GENERATED_FILE_NOTICE}\n%% profile: {}\ngraph LR\n",
        projection.profile
    );
    // One name table across every section, because an edge in one subgraph routinely names
    // a node declared in another and two tables would give that node two identifiers.
    let mut names = Names {
        known: BTreeMap::new(),
        taken: Vec::new(),
        labelled: Vec::new(),
    };

    for section in &projection.sections
    {
        Write_Subgraph(&mut out, &mut names, section);
    }

    return out;
}

/// One section as a subgraph.
fn Write_Subgraph(out: &mut String, names: &mut Names, section: &Section)
{
    let _ = writeln!(out, "  subgraph {}", Quoted_For_Mermaid(&section.title));

    for item in &section.items
    {
        Node_Or_Edge(out, names, item);
    }

    out.push_str("  end\n");
}

/// An item that names both ends is an edge; anything else is a node.
///
/// An edge declares both of its ends, because a relation may name a node no section
/// carried and a graph with a dangling reference does not render at all.
fn Node_Or_Edge(out: &mut String, names: &mut Names, item: &Item)
{
    let Some((from, to)) = item.Field("from").zip(item.Field("to"))
    else
    {
        let label = item.Field("title").unwrap_or(&item.identity);
        let declared = names.Declared(Identity(&item.identity), Label(label));
        let _ = writeln!(out, "    {declared}");

        return;
    };

    let relation = item.Field("relation").unwrap_or("relates");
    let tail = names.Declared(Identity(from), Label(from));
    let head = names.Declared(Identity(to), Label(to));
    let _ = writeln!(out, "    {tail} -->|{}| {head}", Quoted_For_Mermaid(relation));
}

/// A mermaid label, quoted and stripped of the two characters that would end the string.
///
/// The double quote the replacement names is written `'\u{22}'` rather than as the
/// equivalent char literal, which a double quote inside a char literal mis-pairs in the
/// gate's own line blanker -- the mis-pairing is reported as an Allman violation elsewhere
/// in this file, and the escape is what keeps the two spellings of one character apart.
fn Quoted_For_Mermaid(value: &str) -> String
{
    return format!("\"{}\"", value.replace('\u{22}', "'").replace(['\n', '\r'], " "));
}

/// A node's identity, kept distinct from [`Label`] so [`Names::Declared`]'s two positions
/// cannot be swapped at a call site — both are plain strings and nothing else would tell
/// them apart.
struct Identity<'a>(&'a str);

/// A node's rendered label, kept distinct from [`Identity`] for the same reason.
struct Label<'a>(&'a str);

struct Names
{
    known: BTreeMap<String, String>,
    taken: Vec<String>,
    labelled: Vec<String>,
}

impl Names
{
    fn Declared(&mut self, identity: Identity<'_>, label: Label<'_>) -> String
    {
        let identity = identity.0;
        let label = label.0;

        let named = self.For(identity);
        if self.labelled.contains(&named)
        {
            return named;
        }
        self.labelled.push(named.clone());

        return format!("{named}[{}]", Quoted_For_Mermaid(label));
    }

    fn For(&mut self, identity: &str) -> String
    {
        if let Some(known) = self.known.get(identity)
        {
            return known.clone();
        }

        let candidate = match Slug_Of_Text(identity).replace('-', "_")
        {
            slug if slug.is_empty() => "n".to_owned(),
            slug => slug,
        };
        let mut unique = candidate.clone();
        let mut ordinal = 1_u32;
        while self.taken.contains(&unique)
        {
            ordinal = ordinal.saturating_add(1);
            unique = format!("{candidate}_{ordinal}");
        }

        self.taken.push(unique.clone());
        self.known.insert(identity.to_owned(), unique.clone());

        return unique;
    }
}

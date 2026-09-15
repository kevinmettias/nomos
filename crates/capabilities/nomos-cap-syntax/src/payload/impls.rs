//! An implementation block's own `shape`: the label it carries, and the generic type parameters
//! written behind that label.
//!
//! `Impl_` is the whole of this module's subject — the label as a value ([`ImplLabel`]),
//! reading it back ([`Impl_Serves_A_Trait`]), reading the parameter list ([`Impl_Generics`])
//! and writing both ([`Impl_Shape`]) — so those live in a file whose name is that subject,
//! rather than folded into [`super::observation`], which is about [`Observation`] and the
//! shapes a struct, a function and a constant declare.
//!
//! `OD-CAPABILITY-014` is why the label is not the whole shape. An implementation block that
//! declares type parameters writes them behind `generics`, one name per line, and a reader that
//! compared the field against [`TRAIT`] or [`INHERENT`] directly would silently stop recognizing
//! the one shape the extension exists for. Both spellings of the label, and the header the list
//! is written behind, are what this file owns.

use super::{Escape, INHERENT, Observation, TRAIT, Unescape_Field};

/// The `shape` header an implementation block's own generic parameter list is written behind.
const IMPL_GENERICS_HEADER: &str = "generics";

/// Which of the two labels an implementation block's `shape` carries.
///
/// A name rather than a `bool`, for the reason the caller that reads one gives: a position is
/// not a name, and `Impl_Shape(true, &generics)` says nothing about what is true. The two
/// states are the two spellings [`TRAIT`] and [`INHERENT`] can take, and [`Self::Label`] is
/// the one place that choice is made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImplLabel
{
    /// The block serves a trait — [`TRAIT`].
    Trait,
    /// The block is inherent to its own type — [`INHERENT`].
    Inherent,
}

impl ImplLabel
{
    /// The `shape` label this state is written as.
    #[must_use]
    pub fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Trait => TRAIT,
            Self::Inherent => INHERENT,
        };
    }
}

/// Whether an implementation block's `shape` says it serves a trait -- `None` for a `shape`
/// that is not an implementation block's at all.
///
/// This exists so that no consumer compares the `shape` field against [`TRAIT`] or
/// [`INHERENT`] directly. `OD-CAPABILITY-014` gave an implementation block a variable-length
/// body behind that label, so an equality test against the bare constant silently stops
/// recognizing a generic `impl` -- which is the one shape the extension exists for.
#[must_use]
pub fn Impl_Serves_A_Trait(shape: &Observation) -> Option<bool>
{
    return Impl_Shape_Parts(shape).map(|(serves_a_trait, _rest)| return serves_a_trait);
}

/// An implementation block's own declared generic type-parameter names, in declaration
/// order -- `Some` of an empty list for a block that declares none, and `None` for a `shape`
/// that is not an implementation block's at all. Those are different answers, for the reason
/// the module's own three-state argument gives, and a consumer that conflated them would
/// read every struct and every function as an `impl` declaring no generics.
///
/// Lifetime and const generic parameters are deliberately absent. `OD-CAPABILITY-014`
/// extended this shape for one checked need -- telling an implementation block's own name
/// apart from one of its own generics -- and neither of those can ever be that name.
///
/// Each name is unescaped a second time here, on top of whatever `Observation::Decode`
/// already did to the whole value, for the reason [`super::Struct_Fields`] gives for its own
/// pair.
#[must_use]
pub fn Impl_Generics(shape: &Observation) -> Option<Vec<String>>
{
    let (_serves_a_trait, rest) = Impl_Shape_Parts(shape)?;

    if rest.is_empty()
    {
        return Some(Vec::new());
    }

    let body = rest
        .strip_prefix('\n')?
        .strip_prefix(IMPL_GENERICS_HEADER)?
        .strip_prefix('\n')?;

    return Some(body.lines().map(Unescape_Field).collect());
}

/// The `shape` an implementation block declares: its own trait-or-inherent label, and, for a
/// block declaring generic type parameters, that parameter list behind a header -- the wire
/// form [`Impl_Generics`] reads back.
///
/// The label arrives as an [`ImplLabel`] rather than a `bool`, so that the call site says
/// which of the two it means instead of leaving a position to carry it.
///
/// A block declaring no generics encodes as the bare label and its bytes do not move. That
/// is the same "nothing to say" default [`super::Struct_Shape`] already gives a struct with no
/// named fields, and it is what keeps `OD-CAPABILITY-014`'s extension from re-addressing
/// every fact for every non-generic `impl` in a repository.
///
/// Each name is escaped here, before this function's own newline is laid down as the list
/// delimiter, for the reason [`super::Struct_Shape`] gives for its own pair: one escaping pass
/// protects the delimiters one layer up from it and never its own.
#[must_use]
pub fn Impl_Shape(label: ImplLabel, generics: &[String]) -> String
{
    let spelled = label.Label();

    if generics.is_empty()
    {
        return spelled.to_owned();
    }

    let body = generics
        .iter()
        .map(|name| return Escape(name))
        .collect::<Vec<_>>()
        .join("\n");

    return format!("{spelled}\n{IMPL_GENERICS_HEADER}\n{body}");
}

/// An implementation block's `shape`, split into whether it serves a trait and whatever the
/// label is followed by -- `None` for a `shape` no implementation block wrote.
///
/// The label is matched as a whole line rather than as a bare prefix: a `shape` spelled
/// `traits` is not a trait `impl` with a one-character body, and a reader that accepted it
/// as one would report a generic parameter list nobody wrote.
fn Impl_Shape_Parts(shape: &Observation) -> Option<(bool, &str)>
{
    let value = shape.Value()?;

    for (label, serves_a_trait) in [(TRAIT, true), (INHERENT, false)]
    {
        let Some(rest) = value.strip_prefix(label)
        else
        {
            continue;
        };

        if rest.is_empty() || rest.starts_with('\n')
        {
            return Some((serves_a_trait, rest));
        }
    }

    return None;
}

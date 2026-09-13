//! What a provider observed about one declaration, and the words it says it in.

use super::{Escape, PayloadRefusal, PayloadRefusalKind, Unescape_Field};

/// The `kind` label every provider writes for a function form.
pub const FUNCTION: &str = "Function";

/// The `kind` label for an implementation block.
pub const IMPLEMENTATION: &str = "Implementation";

/// The `visibility` label for an item that declares itself public.
pub const PUBLIC: &str = "Public";

/// The `visibility` label for an item form that declares no visibility.
///
/// Reserved with a stated meaning: the provider observed a form that has no visibility to
/// declare — a trait member, an `impl` block, a macro definition. Read the module's own
/// caveat before treating its absence as evidence of anything.
pub const NOT_APPLICABLE: &str = "NotApplicable";

/// The `shape` of a typed declaration whose type is a slice or an array.
pub const SLICE: &str = "slice";

/// The `shape` of a typed declaration whose type is anything else.
pub const VALUE: &str = "value";

/// The `shape` of an implementation block that implements no trait.
pub const INHERENT: &str = "inherent";

/// The `shape` of an implementation block that implements a trait.
pub const TRAIT: &str = "trait";

/// The `shape` prefix a function's arity is written behind.
const FUNCTION_SHAPE: &str = "fn/";

/// The `shape` header a struct's field list is written behind.
const STRUCT_SHAPE_HEADER: &str = "fields";

/// The `shape` header an implementation block's own generic parameter list is written behind.
const IMPL_GENERICS_HEADER: &str = "generics";

/// What a provider saw when it looked — including that it could not look.
///
/// Three states rather than an `Option`, because the two empty answers are not the same
/// answer and one of them is a silent downgrade. See the module doc.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Observation
{
    /// The method this provider used cannot see this. Nothing is claimed either way.
    NotObserved,
    /// The provider looked, and there is nothing here.
    Absent,
    /// The provider looked, and this is what it saw.
    Present(String),
}

impl Observation
{
    /// The value, when there is one.
    ///
    /// `None` for both empty states, so a caller that only wants the text can have it —
    /// and a caller that must not confuse the two has [`Observation::Was_Observed`].
    #[must_use]
    pub fn Value(&self) -> Option<&str>
    {
        return match self
        {
            Self::Present(value) => Some(value),
            Self::NotObserved | Self::Absent => None,
        };
    }

    /// Whether the provider was able to look at all.
    #[must_use]
    pub fn Was_Observed(&self) -> bool
    {
        return !matches!(self, Self::NotObserved);
    }

    /// The wire form: `-`, `.`, or `+` and the escaped value.
    #[must_use]
    pub fn Encode(&self) -> String
    {
        return match self
        {
            Self::NotObserved => "-".to_owned(),
            Self::Absent => ".".to_owned(),
            Self::Present(value) => format!("+{}", Escape(value)),
        };
    }

    /// Reads the wire form back.
    ///
    /// A field that is neither of the two marks and does not begin with `+` is not this
    /// schema — refused rather than read as absent, because absent is one of the answers.
    fn Decode(field: &str) -> Option<Self>
    {
        return match field
        {
            "-" => Some(Self::NotObserved),
            "." => Some(Self::Absent),
            _ => field.strip_prefix('+').map(|value| return Self::Present(Unescape_Field(value))),
        };
    }
}

/// The arity a function `shape` declares, if the field is one.
#[must_use]
pub fn Function_Arity(shape: &Observation) -> Option<u32>
{
    return shape
        .Value()?
        .strip_prefix(FUNCTION_SHAPE)
        .and_then(|arity| return arity.parse().ok());
}

/// The `shape` a function of this arity declares.
#[must_use]
pub fn Function_Shape(arity: usize) -> String
{
    return format!("{FUNCTION_SHAPE}{arity}");
}

/// A struct's own field list, if the `shape` this rule reads is one — `None` for a struct
/// with no named fields (`Observation::Absent`) and for anything that is not a struct's
/// shape at all.
///
/// Each pair is `(name, type)`, in declaration order — order is preserved because it is
/// free (the wire form already carries it), even though `OD-CAPABILITY-010`'s own
/// comparison reads this set-wise rather than positionally.
///
/// `name` and `type` are unescaped a second time here, on top of whatever
/// `Observation::Decode` already did to the whole value — see [`Struct_Shape`]'s own doc
/// for why one round is not enough.
#[must_use]
pub fn Struct_Fields(shape: &Observation) -> Option<Vec<(String, String)>>
{
    let value = shape.Value()?;
    let body = value.strip_prefix(STRUCT_SHAPE_HEADER)?.strip_prefix('\n')?;

    return body
        .lines()
        .map(|line| {
            return line
                .split_once('\t')
                .map(|(name, kind)| return (Unescape_Field(name), Unescape_Field(kind)));
        })
        .collect();
}

/// The `shape` a struct with these named fields declares, in declaration order — the wire
/// form [`Struct_Fields`] reads back. `None` for an empty field list, the same convention
/// every provider's own `Declared.shape: Option<String>` already uses for "the form has no
/// shape to describe": a struct this reader observed to have no named fields (Rust's unit
/// and tuple forms) has nothing to say, the same default every other kind already has.
///
/// `name` and `type` are escaped here, before this function's own tab and newline are laid
/// down as the field-pair and field-line delimiters — one escaping pass is not enough,
/// because a field whose own name or type spelling contains a real tab or newline would
/// otherwise read back as a second, wrongly-split field. `Observation::Encode` escapes the
/// whole result a second time when this becomes the wire's `shape` column, which is what
/// protects *this* function's own delimiters from `Observation`'s outer grammar; each
/// escaping pass protects the delimiters one layer up from it, never its own.
#[must_use]
pub fn Struct_Shape(fields: &[(String, String)]) -> Option<String>
{
    if fields.is_empty()
    {
        return None;
    }

    let body = fields
        .iter()
        .map(|(name, kind)| return format!("{}\t{}", Escape(name), Escape(kind)))
        .collect::<Vec<_>>()
        .join("\n");

    return Some(format!("{STRUCT_SHAPE_HEADER}\n{body}"));
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
/// already did to the whole value, for the reason [`Struct_Fields`] gives for its own pair.
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
/// A block declaring no generics encodes as the bare label and its bytes do not move. That
/// is the same "nothing to say" default [`Struct_Shape`] already gives a struct with no
/// named fields, and it is what keeps `OD-CAPABILITY-014`'s extension from re-addressing
/// every fact for every non-generic `impl` in a repository.
///
/// Each name is escaped here, before this function's own newline is laid down as the list
/// delimiter, for the reason [`Struct_Shape`] gives for its own pair: one escaping pass
/// protects the delimiters one layer up from it and never its own.
#[must_use]
pub fn Impl_Shape(serves_a_trait: bool, generics: &[String]) -> String
{
    let label = if serves_a_trait { TRAIT } else { INHERENT };

    if generics.is_empty()
    {
        return label.to_owned();
    }

    let body = generics
        .iter()
        .map(|name| return Escape(name))
        .collect::<Vec<_>>()
        .join("\n");

    return format!("{label}\n{IMPL_GENERICS_HEADER}\n{body}");
}

/// Reads a field that must be an observation.
pub(super) fn Observed_Field(value: &str, field: &'static str, line: usize) -> Result<Observation, PayloadRefusal>
{
    return Observation::Decode(value).ok_or_else(|| {
        return PayloadRefusal::At(
            line,
            PayloadRefusalKind::UnreadableObservation {
                field,
                value: value.to_owned(),
            },
        );
    });
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Value_Should_Return_The_Text_Only_When_Present()
    {
        assert_eq!(Observation::Present("x".to_owned()).Value(), Some("x"));
        assert_eq!(Observation::Absent.Value(), None);
        assert_eq!(Observation::NotObserved.Value(), None);
    }

    #[test]
    fn Test_Was_Observed_Should_Be_False_Only_When_The_Method_Could_Not_Look()
    {
        assert!(!Observation::NotObserved.Was_Observed());
        assert!(Observation::Absent.Was_Observed());
        assert!(Observation::Present("x".to_owned()).Was_Observed());
    }

    #[test]
    fn Test_Encode_Should_Write_The_Mark_Each_State_Declares()
    {
        assert_eq!(Observation::NotObserved.Encode(), "-");
        assert_eq!(Observation::Absent.Encode(), ".");
        assert_eq!(Observation::Present("a\tb".to_owned()).Encode(), "+a\\tb");
    }

    #[test]
    fn Test_Function_Arity_Should_Read_The_Number_Behind_The_Function_Prefix()
    {
        assert_eq!(Function_Arity(&Observation::Present(Function_Shape(3))), Some(3));
        assert_eq!(Function_Arity(&Observation::Present(SLICE.to_owned())), None);
        assert_eq!(Function_Arity(&Observation::NotObserved), None);
    }

    #[test]
    fn Test_Function_Shape_Should_Prefix_The_Arity()
    {
        assert_eq!(Function_Shape(0), "fn/0");
        assert_eq!(Function_Shape(5), "fn/5");
    }

    #[test]
    fn Test_Struct_Fields_Should_Read_Back_What_Struct_Shape_Wrote()
    {
        let fields = vec![("a".to_owned(), "u32".to_owned()), ("b".to_owned(), "String".to_owned())];
        let shape = Struct_Shape(&fields).map(Observation::Present).expect("two fields is not empty");

        assert_eq!(Struct_Fields(&shape), Some(fields));
        assert_eq!(Struct_Fields(&Observation::NotObserved), None);
    }

    #[test]
    fn Test_Struct_Shape_Should_Have_Nothing_To_Say_For_No_Named_Fields()
    {
        assert_eq!(Struct_Shape(&[]), None);
        assert!(Struct_Shape(&[("a".to_owned(), "u32".to_owned())]).is_some());
    }

    #[test]
    fn Test_Observed_Field_Should_Refuse_A_Value_That_Is_Not_One_Of_The_Three_Marks()
    {
        assert_eq!(Observed_Field("-", "documentation", 2), Ok(Observation::NotObserved));
        assert_eq!(Observed_Field(".", "documentation", 2), Ok(Observation::Absent));
        assert_eq!(Observed_Field("+ok", "documentation", 2), Ok(Observation::Present("ok".to_owned())));

        let refused = Observed_Field("none", "documentation", 2).expect_err("not one of the three marks");
        assert_eq!(
            refused,
            PayloadRefusal::At(
                2,
                PayloadRefusalKind::UnreadableObservation {
                    field: "documentation",
                    value: "none".to_owned(),
                },
            )
        );
    }
}

//! What a provider observed about one declaration, and the words it says it in.

use super::*;

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
            _ => field.strip_prefix('+').map(|value| return Self::Present(Unescape(value))),
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

/// Reads a field that must be an observation.
pub(super) fn Observed(value: &str, field: &'static str, line: usize) -> Result<Observation, PayloadRefusal>
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

//! The closed vocabulary a repository's declared naming convention is drawn from.
//!
//! Ported from code-standards' own `rules/general/style/shared/naming/case.go` and
//! `case_validation.go` rather than reinvented: those files name eight case styles and
//! give each an exact shape checked against a real corpus, and `OD-RULES-011` found that a
//! hand-rolled shape in this workspace had already drifted from the vocabulary its own
//! rule id named (`upper-snake` implemented as `screaming-snake`). Every variant's
//! [`Case::Label`] round-trips to code-standards' own lowercase-hyphenated spelling, so a
//! value written in `standards.json` by someone who has never read this crate's source
//! still resolves.

const UPPER_CAMEL_LABEL: &str = "upper-camel";
const LOWER_CAMEL_LABEL: &str = "lower-camel";
const UNDERSCORE_CAMEL_LABEL: &str = "underscore-camel";
const LOWER_SNAKE_LABEL: &str = "lower-snake";
const UPPER_SNAKE_LABEL: &str = "upper-snake";
const SCREAMING_SNAKE_LABEL: &str = "screaming-snake";
const MIXED_SNAKE_LABEL: &str = "mixed-snake";
const LOWER_KEBAB_LABEL: &str = "lower-kebab";
const ANY_LABEL: &str = "any";

/// One of code-standards' eight closed case styles, or [`Case::Any`] (no rule, satisfied
/// by every name).
///
/// Worked examples, from `case.go`'s own doc comments: [`Case::UpperCamel`] is
/// `ComputeTotal`, [`Case::LowerCamel`] is `computeTotal`, [`Case::UnderscoreCamel`] is
/// `_computeTotal`, [`Case::LowerSnake`] is `compute_total`, [`Case::UpperSnake`] is
/// `Compute_Total`, [`Case::ScreamingSnake`] is `COMPUTE_TOTAL`, [`Case::MixedSnake`] is
/// `compute_Total`, [`Case::LowerKebab`] is `compute-total`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Case
{
    UpperCamel,
    LowerCamel,
    UnderscoreCamel,
    LowerSnake,
    UpperSnake,
    ScreamingSnake,
    MixedSnake,
    LowerKebab,
    /// No rule. Every name conforms, the same "declaring nothing leaves each language's
    /// own convention in force" default `check-naming`'s own `spec.go` states.
    Any,
}

impl Case
{
    /// This case's stable, code-standards-compatible wire spelling.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::UpperCamel => UPPER_CAMEL_LABEL,
            Self::LowerCamel => LOWER_CAMEL_LABEL,
            Self::UnderscoreCamel => UNDERSCORE_CAMEL_LABEL,
            Self::LowerSnake => LOWER_SNAKE_LABEL,
            Self::UpperSnake => UPPER_SNAKE_LABEL,
            Self::ScreamingSnake => SCREAMING_SNAKE_LABEL,
            Self::MixedSnake => MIXED_SNAKE_LABEL,
            Self::LowerKebab => LOWER_KEBAB_LABEL,
            Self::Any => ANY_LABEL,
        };
    }

    /// The case named by `label`, or `None` for a spelling outside the closed set —
    /// a repository declaring a ninth case is a load error at the reader, the same
    /// "a casing scheme this check cannot read is one it would silently ignore" refusal
    /// code-standards' own `spec.go` states for the identical reason.
    #[must_use]
    pub fn From_Label(label: &str) -> Option<Self>
    {
        return match label
        {
            UPPER_CAMEL_LABEL => Some(Self::UpperCamel),
            LOWER_CAMEL_LABEL => Some(Self::LowerCamel),
            UNDERSCORE_CAMEL_LABEL => Some(Self::UnderscoreCamel),
            LOWER_SNAKE_LABEL => Some(Self::LowerSnake),
            UPPER_SNAKE_LABEL => Some(Self::UpperSnake),
            SCREAMING_SNAKE_LABEL => Some(Self::ScreamingSnake),
            MIXED_SNAKE_LABEL => Some(Self::MixedSnake),
            LOWER_KEBAB_LABEL => Some(Self::LowerKebab),
            ANY_LABEL => Some(Self::Any),
            _ => None,
        };
    }

    /// Whether `name` already has this case's shape.
    ///
    /// Ported from `case_validation.go`'s own `case_shapes` regular expressions, restated
    /// as segment predicates rather than adding a `regex` dependency for eight fixed
    /// patterns: every shape below is `Line_By_Line` equivalent to its named regex, and
    /// [`tests`] checks each one against `case_validation.go`'s own worked examples.
    #[must_use]
    pub fn Conforms(self, name: &str) -> bool
    {
        if self == Self::Any
        {
            return true;
        }

        if name.is_empty()
        {
            return false;
        }

        return match self
        {
            Self::Any => true,
            Self::UpperCamel => Is_Camel(name, Titled::Yes),
            Self::LowerCamel => Is_Camel(name, Titled::No),
            Self::UnderscoreCamel => name.strip_prefix('_').is_some_and(|rest| return Is_Camel(rest, Titled::No)),
            Self::LowerSnake => Is_Segmented(name, '_', Segment_Is_Lower),
            Self::UpperSnake => Is_Upper_Snake(name),
            Self::ScreamingSnake => Is_Segmented(name, '_', Segment_Is_Screaming),
            Self::MixedSnake => Is_Mixed_Snake(name),
            Self::LowerKebab => Is_Segmented(name, '-', Segment_Is_Lower),
        };
    }
}

/// Whether a camel-cased name's first character is titled (`ComputeTotal`) or not
/// (`computeTotal`) — `^[A-Z][A-Za-z0-9]*$` and `^[a-z][A-Za-z0-9]*$` differ only here.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Titled
{
    Yes,
    No,
}

/// `^[A-Z][A-Za-z0-9]*$` (or lower-led): one run, no separator, alphanumeric throughout.
fn Is_Camel(name: &str, titled: Titled) -> bool
{
    let mut characters = name.chars();
    let Some(first) = characters.next()
    else
    {
        return false;
    };

    let first_ok = match titled
    {
        Titled::Yes => first.is_ascii_uppercase(),
        Titled::No => first.is_ascii_lowercase(),
    };

    return first_ok && characters.all(|character| return character.is_ascii_alphanumeric());
}

/// `^[a-z0-9]+(_[a-z0-9]+)*$` or `^[a-z0-9]+(-[a-z0-9]+)*$`: every segment lower-or-digit,
/// none empty.
fn Is_Segmented(name: &str, separator: char, segment_ok: fn(&str) -> bool) -> bool
{
    return name.split(separator).all(|segment| return !segment.is_empty() && segment_ok(segment));
}

fn Segment_Is_Lower(segment: &str) -> bool
{
    return !segment.is_empty()
        && segment.chars().all(|character| return character.is_ascii_lowercase() || character.is_ascii_digit());
}

fn Segment_Is_Screaming(segment: &str) -> bool
{
    return segment.chars().all(|character| return character.is_ascii_uppercase() || character.is_ascii_digit());
}

/// `^[A-Z][A-Za-z0-9]*(_[A-Z0-9][A-Za-z0-9]*)*$`: the FIRST segment opens with a letter
/// specifically (`case_validation.go`'s own comment: a segment may open with a capital OR
/// a digit for every OTHER position, but the name as a whole still starts with a letter);
/// every segment after it opens with an uppercase letter or a digit, and its remainder is
/// alphanumeric — `Region_200`, `Solve_Lp_2d` and `Region_800x600` all conform.
fn Is_Upper_Snake(name: &str) -> bool
{
    let mut segments = name.split('_');
    let Some(first) = segments.next()
    else
    {
        return false;
    };

    if !Segment_Opens_Upper_Letter(first)
    {
        return false;
    }

    return segments.all(Segment_Opens_Upper_Or_Digit);
}

/// `^[a-z0-9]+(_[A-Za-z0-9]+)*$`: the first segment lower-or-digit only, every later
/// segment alphanumeric with no case constraint of its own — `compute_Total`, the shape
/// "lowercase only the leading word, keep the rest as written" needs.
fn Is_Mixed_Snake(name: &str) -> bool
{
    let mut segments = name.split('_');
    let Some(first) = segments.next()
    else
    {
        return false;
    };

    if !Segment_Is_Lower(first)
    {
        return false;
    }

    return segments.all(|segment| return !segment.is_empty() && segment.chars().all(|character| return character.is_ascii_alphanumeric()));
}

fn Segment_Opens_Upper_Letter(segment: &str) -> bool
{
    let mut characters = segment.chars();
    let Some(first) = characters.next()
    else
    {
        return false;
    };

    return first.is_ascii_uppercase() && characters.all(|character| return character.is_ascii_alphanumeric());
}

fn Segment_Opens_Upper_Or_Digit(segment: &str) -> bool
{
    let mut characters = segment.chars();
    let Some(first) = characters.next()
    else
    {
        return false;
    };

    let first_ok = first.is_ascii_uppercase() || first.is_ascii_digit();
    return first_ok && characters.all(|character| return character.is_ascii_alphanumeric());
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// One row per `case.go` worked example — the exact name each doc comment gives.
    fn Worked_Examples() -> Vec<(Case, &'static str)>
    {
        return vec![
            (Case::UpperCamel, "ComputeTotal"),
            (Case::LowerCamel, "computeTotal"),
            (Case::UnderscoreCamel, "_computeTotal"),
            (Case::LowerSnake, "compute_total"),
            (Case::UpperSnake, "Compute_Total"),
            (Case::ScreamingSnake, "COMPUTE_TOTAL"),
            (Case::MixedSnake, "compute_Total"),
            (Case::LowerKebab, "compute-total"),
        ];
    }

    #[test]
    fn Test_Conforms_Should_Accept_Its_Own_Worked_Example()
    {
        for (case, name) in Worked_Examples()
        {
            assert!(case.Conforms(name), "{case:?} should accept its own worked example {name:?}");
        }
    }

    /// Three pairs among the eight are not mutually exclusive by `case_validation.go`'s
    /// own regexes, and this is a real property of those shapes rather than a defect in
    /// this port. `upper-snake`'s trailing `(_[A-Z0-9][A-Za-z0-9]*)*` group matches zero
    /// times, so a name with no underscore at all reduces the pattern to exactly
    /// `^[A-Z][A-Za-z0-9]*$` — `upper-camel`'s own shape — which is why `ComputeTotal`
    /// (upper-camel's own example) is also a valid upper-snake name. Separately,
    /// `upper-snake`'s `[A-Za-z0-9]*` after each segment's leading letter does not forbid
    /// the rest of the segment from also being uppercase, so `COMPUTE_TOTAL`
    /// (screaming-snake's own example) is simultaneously a valid upper-snake name. The
    /// same zero-case-constraint reasoning makes `compute_total` (lower-snake's own
    /// example) a valid mixed-snake name. Every other pair of the eight worked examples is
    /// genuinely exclusive.
    #[test]
    fn Test_Conforms_Should_Reject_Every_Other_Cases_Worked_Example_Except_The_Three_Known_Overlaps()
    {
        let examples = Worked_Examples();
        let known_overlaps = [
            (Case::UpperSnake, Case::ScreamingSnake),
            (Case::UpperSnake, Case::UpperCamel),
            (Case::MixedSnake, Case::LowerSnake),
        ];

        for (case, _) in &examples
        {
            for (other_case, other_name) in &examples
            {
                if other_case == case || known_overlaps.contains(&(*case, *other_case))
                {
                    continue;
                }
                assert!(
                    !case.Conforms(other_name),
                    "{case:?} should reject {other_name:?}, {other_case:?}'s own worked example"
                );
            }
        }
    }

    #[test]
    fn Test_Upper_Snake_Should_Also_Accept_Screaming_Snakes_Own_Example()
    {
        assert!(Case::UpperSnake.Conforms("COMPUTE_TOTAL"));
    }

    #[test]
    fn Test_Upper_Snake_Should_Also_Accept_Upper_Camels_Own_Example()
    {
        assert!(Case::UpperSnake.Conforms("ComputeTotal"));
    }

    #[test]
    fn Test_Mixed_Snake_Should_Also_Accept_Lower_Snakes_Own_Example()
    {
        assert!(Case::MixedSnake.Conforms("compute_total"));
    }

    #[test]
    fn Test_Upper_Snake_Should_Accept_A_Digit_Led_Segment_After_The_First()
    {
        assert!(Case::UpperSnake.Conforms("Region_200"));
        assert!(Case::UpperSnake.Conforms("Solve_Lp_2d"));
        assert!(Case::UpperSnake.Conforms("Region_800x600"));
    }

    #[test]
    fn Test_Upper_Snake_Should_Reject_A_Digit_Led_First_Segment()
    {
        assert!(!Case::UpperSnake.Conforms("200_Region"));
    }

    #[test]
    fn Test_Upper_Snake_Should_Reject_A_Lower_Led_Segment()
    {
        assert!(!Case::UpperSnake.Conforms("Compute_total"));
    }

    #[test]
    fn Test_Any_Should_Accept_Every_Name()
    {
        for (_, name) in Worked_Examples()
        {
            assert!(Case::Any.Conforms(name));
        }
        assert!(Case::Any.Conforms("literally anything"));
    }

    #[test]
    fn Test_Conforms_Should_Reject_An_Empty_Name_For_Every_Case_But_Any()
    {
        assert!(!Case::UpperCamel.Conforms(""));
        assert!(!Case::LowerSnake.Conforms(""));
        assert!(!Case::UpperSnake.Conforms(""));
        assert!(!Case::ScreamingSnake.Conforms(""));
        assert!(!Case::MixedSnake.Conforms(""));
        assert!(Case::Any.Conforms(""));
    }

    #[test]
    fn Test_Label_Should_Round_Trip_Through_From_Label_For_Every_Variant()
    {
        let variants = [
            Case::UpperCamel,
            Case::LowerCamel,
            Case::UnderscoreCamel,
            Case::LowerSnake,
            Case::UpperSnake,
            Case::ScreamingSnake,
            Case::MixedSnake,
            Case::LowerKebab,
            Case::Any,
        ];

        for variant in variants
        {
            assert_eq!(Case::From_Label(variant.Label()), Some(variant), "{variant:?} did not round-trip");
        }
    }

    #[test]
    fn Test_From_Label_Should_Reject_A_Spelling_Outside_The_Closed_Set()
    {
        assert_eq!(Case::From_Label("kebab-case"), None);
        assert_eq!(Case::From_Label("PascalCase"), None);
        assert_eq!(Case::From_Label(""), None);
    }
}

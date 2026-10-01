//! Vague, grab-bag words that name no concrete responsibility, ported from code-standards'
//! `check-naming-clarity` (`kernel/config/words/{config.go,defaults.go}`'s
//! `Vague_Word`/`default_Vague_Words`).
//!
//! A structural twin of [`super::abbreviations`], reading the same payload the same way:
//! the same trait-impl exemption (a trait fixed a member's name, not this repository), the
//! same use-binding exemption (an import names something declared elsewhere), the same
//! struct-field judgment, the same word-splitter. The two rules differ only in which
//! vocabulary a word is checked against and how the check itself reads: a banned/vowelless
//! judgment there, a flat membership test here.
//!
//! Unlike the approved/banned vocabulary, `vague`/`vague_exempt` is a genuine
//! `OD-RULES-011` dimension — code-standards' own worked example (`Info` is vague for a
//! class but the only correct name for a severity enum's middle member) is a real
//! per-repository disagreement, not a fixed fact — so this rule extends the existing
//! `nomos.cap.words.policy` capability rather than adding a fifth capability crate.
//!
//! # Two narrowings, both already accepted by `abbreviations.rs` for the identical reason
//!
//! The real Go tool's `naming: allow <reason>` marker escape hatch (same-line-or-above,
//! reason required) is not ported at all: [`nomos_cap_syntax::PayloadItem`] carries no
//! line number for any declared item, so there is no position to search near. This is the
//! same payload limitation `abbreviations.rs`'s own module doc does not mention only
//! because that rule never had a marker to begin with — this is the first rule in this
//! crate to actually hit the gap rather than simply not need it.
//!
//! The real tool also judges local `let` bindings ("Local bindings are judged too: a
//! `let data = ...` is a vague name whether or not it escapes the function") — this port
//! cannot, because the syntax payload carries no function-body-local declarations at all.
//! Only top-level and type-level declarations, and struct fields, are judged — exactly
//! what [`super::abbreviations::Check_Abbreviations`] already judges.

use crate::SourceFile;
use crate::checks::finding_shape::{Finding_Shape, Member_Finding};
use nomos_analysis::{FactReader, InputDigest};
use nomos_cap_syntax::{IMPLEMENTATION, Impl_Serves_A_Trait, PayloadItem, Struct_Fields, SyntaxPayload};
use nomos_contracts::{Finding, RuleId};

/// This rule's own identifier, matching the code-standards rule id.
pub const NAMING_CLARITY: &str = "naming-clarity";

const STRUCT: &str = "Struct";
const USE_BINDING: &str = "Use";

/// code-standards' `default_Vague_Words` — ported verbatim from `defaults.go`.
const DEFAULT_VAGUE_WORDS: &[&str] = &[
    "data", "do", "helper", "helpers", "manager", "managers", "object", "objects", "process", "processor", "service",
    "services", "stuff", "thing", "things", "util", "utils", "utility", "handler", "wrapper",
];

/// Judges declared item names and named struct fields against the vague-word vocabulary, a
/// repository's own `nomos.cap.words.policy` additions and exemptions extending and
/// narrowing it.
#[must_use]
pub fn Check_Naming_Clarity(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let VagueWordLists { additions, exempt } = Resolve_Vague_Lists(facts);
    let mut findings = Vec::new();

    for source in sources
    {
        match super::reading::Payload_Of(source, facts)
        {
            Ok(payload) =>
            {
                let violations = Violations_In(&payload, &source.path, &additions, &exempt);
                findings.extend(violations);
            }
            Err(finding) => findings.push(Unread_As_This_Rule(finding)),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// A repository's own vague-word additions and exemptions, named rather than returned as a
/// tuple: both fields are `Vec<String>`, and a caller that received them positionally could
/// swap `additions` and `exempt` without the compiler ever objecting.
struct VagueWordLists
{
    additions: Vec<String>,
    exempt: Vec<String>,
}

/// Resolves a repository's own vague-word additions and exemptions — both empty on any
/// `Require` failure, per `OD-CAPABILITY-004`/`OD-RULES-011`'s settled optional-read
/// pattern: this capability is optional, and its absence must never surface as a `Finding`
/// or this capability's own `Applicability`.
fn Resolve_Vague_Lists(facts: &mut dyn FactReader) -> VagueWordLists
{
    let subject = nomos_model::Subject_Of_Path("");
    let Ok(fact) =
        facts.Require(&nomos_cap_words_policy::Capability(), &subject, InputDigest::Of(&[]), &Words_Policy_Requirement())
    else
    {
        return VagueWordLists { additions: Vec::new(), exempt: Vec::new() };
    };

    let Ok(payload) = nomos_cap_words_policy::Parse_Payload(&fact.payload.bytes)
    else
    {
        return VagueWordLists { additions: Vec::new(), exempt: Vec::new() };
    };

    return VagueWordLists { additions: payload.vague_additions, exempt: payload.vague_exempt };
}

/// This crate's own floor for `nomos.cap.words.policy` — the same floor
/// `abbreviations.rs`'s own `Words_Policy_Requirement` states, duplicated per this crate's
/// per-file convention rather than shared, since a second rule reading the same capability
/// is exactly the shape every other `OD-RULES-011` rule already reads it through.
fn Words_Policy_Requirement() -> nomos_capability::Requirement
{
    return nomos_capability::Requirement::New(nomos_cap_words_policy::Capability(), nomos_cap_words_policy::CONTRACT_VERSION, nomos_cap_words_policy::Ceiling());
}

fn Violations_In(payload: &SyntaxPayload, path: &str, additions: &[String], exempt: &[String]) -> Vec<Finding>
{
    let mut findings = Vec::new();
    let mut enclosing_trait_impl: Option<String> = None;

    for item in &payload.items
    {
        let violations = Item_Violations_In(item, path, WordLists { additions, exempt }, &mut enclosing_trait_impl);
        findings.extend(violations);
    }

    return findings;
}

/// One item's own violations: whether it is exempt as an unowned trait-member name or a
/// `use` binding, then its own name and (for a struct) its fields, against the vocabulary.
/// `enclosing_trait_impl` is carried and updated in place, the same "an `impl` block resets
/// it" rule [`Enclosing_Trait_Impl`] states, so each item in call order sees the block its
/// predecessor left behind.
/// The vague-word vocabulary's two lists, grouped so [`Item_Violations_In`] stays under
/// the parameter-count ceiling.
struct WordLists<'a>
{
    additions: &'a [String],
    exempt: &'a [String],
}

fn Item_Violations_In(item: &PayloadItem, path: &str, words: WordLists<'_>, enclosing_trait_impl: &mut Option<String>) -> Vec<Finding>
{
    *enclosing_trait_impl = Enclosing_Trait_Impl(item, enclosing_trait_impl.take());
    if Is_Exempt(item, enclosing_trait_impl.as_deref())
    {
        return Vec::new();
    }

    return Own_And_Field_Violations(item, path, words.additions, words.exempt);
}

/// The qualified name of the trait-serving `impl` block whose members `item` and everything
/// after it may belong to, carried forward from `previous` when `item` is not itself an
/// `impl` block. Ported identically from `abbreviations.rs`'s own function of the same
/// name and the same reasoning: an `impl` block resets the carry unconditionally, including
/// an inherent one, so two blocks for the same type do not share an exemption.
fn Enclosing_Trait_Impl(item: &PayloadItem, previous: Option<String>) -> Option<String>
{
    if item.kind != IMPLEMENTATION
    {
        return previous;
    }

    // Read through the typed reader rather than compared against `TRAIT`. An `impl` block's
    // own shape carries its generic type parameters behind that label since
    // `OD-CAPABILITY-014`, so an equality test against the bare constant answers `false` for
    // `impl<T> Display for T` -- a trait impl whose members this would then stop exempting.
    if Impl_Serves_A_Trait(&item.shape) == Some(true)
    {
        return Some(item.qualified_name.clone());
    }

    return None;
}

/// Whether `item`'s own name is not this rule's to judge: a member of the trait `impl`
/// block at `enclosing_trait_impl` (a name the trait fixed, not the author), or a `use`
/// binding (a name chosen wherever the binding's target was declared).
fn Is_Exempt(item: &PayloadItem, enclosing_trait_impl: Option<&str>) -> bool
{
    if enclosing_trait_impl.is_some_and(|block| return Is_Member_Of(item, block))
    {
        return true;
    }

    return item.kind == USE_BINDING;
}

fn Is_Member_Of(item: &PayloadItem, block: &str) -> bool
{
    return item.qualified_name.starts_with(&format!("{block}::"));
}

/// `item`'s own name, and (for a struct) its fields, against the vocabulary.
fn Own_And_Field_Violations(item: &PayloadItem, path: &str, additions: &[String], exempt: &[String]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    if let Some(word) = First_Vague_Word(item.Own_Name(), additions, exempt)
    {
        let finding = Violation_Finding(path, item, item.Own_Name(), word);
        findings.push(finding);
    }

    if item.kind == STRUCT
    {
        let field_violations = Field_Violations_In(path, item, additions, exempt);
        findings.extend(field_violations);
    }

    return findings;
}

fn Field_Violations_In(path: &str, item: &PayloadItem, additions: &[String], exempt: &[String]) -> Vec<Finding>
{
    let Some(fields) = Struct_Fields(&item.shape) else { return Vec::new() };

    return fields
        .iter()
        .filter_map(|(name, _type_name)| {
            let word = First_Vague_Word(name, additions, exempt)?;
            return Some(Violation_Finding(path, item, name, word));
        })
        .collect();
}

fn Unread_As_This_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(NAMING_CLARITY);
    finding.summary = finding
        .summary
        .replace("this file's naming could not be judged", "this file's naming clarity could not be judged");
    return finding;
}

/// The first vague word inside `name`, if any — code-standards' own `Vague_Word` reports
/// only the first match too.
fn First_Vague_Word(name: &str, additions: &[String], exempt: &[String]) -> Option<String>
{
    return Split_Identifier_Words(name).into_iter().find(|word| return Is_Vague(word, additions, exempt));
}

fn Is_Vague(word: &str, additions: &[String], exempt: &[String]) -> bool
{
    if exempt.iter().any(|entry| return entry == word)
    {
        return false;
    }

    return DEFAULT_VAGUE_WORDS.contains(&word) || additions.iter().any(|entry| return entry == word);
}

/// Splits `name` into its lowercase word segments — duplicated verbatim from
/// `abbreviations.rs`'s own function of the same name, per this crate's established
/// per-file duplication convention for a small helper two rules both need.
fn Split_Identifier_Words(name: &str) -> Vec<String>
{
    let separated = Insert_Case_Boundaries(name);

    return separated
        .split(|character: char| return character == '_' || character == '-' || character == ' ')
        .filter(|field| return !field.is_empty() && !Is_All_Digits(field))
        .map(str::to_lowercase)
        .collect();
}

fn Insert_Case_Boundaries(name: &str) -> String
{
    let mut result = String::new();
    let mut previous: Option<char> = None;

    for current in name.chars()
    {
        if previous.is_some_and(|previous_character| return Is_New_Word_Start(previous_character, current))
        {
            result.push(' ');
        }
        result.push(current);
        previous = Some(current);
    }

    return result;
}

fn Is_New_Word_Start(previous: char, current: char) -> bool
{
    return current.is_ascii_uppercase() && (previous.is_ascii_lowercase() || previous.is_ascii_digit());
}

fn Is_All_Digits(field: &str) -> bool
{
    return !field.is_empty() && field.chars().all(|character| return character.is_ascii_digit());
}

/// One name carrying a word this rule reads as vague.
fn Violation_Finding(path: &str, item: &PayloadItem, name: &str, word: String) -> Finding
{
    return Member_Finding(
        Finding_Shape { rule: NAMING_CLARITY, path, summary: format!("`{name}` contains vague word `{word}`; use a concrete responsibility name") },
        item,
        name,
    );
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Split_Identifier_Words_Should_Split_On_Case_Boundaries()
    {
        assert_eq!(Split_Identifier_Words("dataManager"), vec!["data", "manager"]);
    }

    #[test]
    fn Test_Is_Vague_Should_Flag_A_Default_Vague_Word()
    {
        assert!(Is_Vague("helper", &[], &[]));
    }

    #[test]
    fn Test_Is_Vague_Should_Accept_An_Ordinary_Word()
    {
        assert!(!Is_Vague("total", &[], &[]));
    }

    #[test]
    fn Test_Is_Vague_Should_Flag_A_Repository_Added_Vague_Word()
    {
        assert!(Is_Vague("registry", &["registry".to_owned()], &[]));
        assert!(!Is_Vague("registry", &[], &[]), "not vague without the repository's own addition");
    }

    #[test]
    fn Test_Is_Vague_Should_Accept_A_Repository_Exempted_Default_Word()
    {
        assert!(!Is_Vague("handler", &[], &["handler".to_owned()]));
    }

    #[test]
    fn Test_Violations_In_Should_Report_A_Vague_Item_Name()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPrivate\tDataHelper\t.\t+fn/0\n");

        let findings = Violations_In(&payload, "src/lib.rs", &[], &[]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "DataHelper");
    }

    #[test]
    fn Test_Violations_In_Should_Report_A_Vague_Struct_Field()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\nitem\t0\tStruct\tPublic\tOrder\t.\t+fields\\nhandler\\tString\\ntotal\\tf64\n",
        );

        let findings = Violations_In(&payload, "src/lib.rs", &[], &[]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "handler");
    }

    #[test]
    fn Test_Violations_In_Should_Not_Judge_A_Name_A_Trait_Fixed()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\nitem\t0\tImplementation\tNotApplicable\tThing\t.\t+trait\n\
             item\t1\tFunction\tPrivate\tThing::process\t.\t+fn/2\n",
        );

        let findings = Violations_In(&payload, "src/lib.rs", &[], &[]);

        assert_eq!(findings.len(), 1, "the trait impl's own name is still judged, its member is not: {findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "Thing");
    }

    #[test]
    fn Test_Violations_In_Should_Not_Judge_A_Use_Binding()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tUse\tPrivate\tDataHelper\t.\t.\n");

        let findings = Violations_In(&payload, "src/lib.rs", &[], &[]);

        assert!(findings.is_empty(), "an import names something declared elsewhere: {findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Accept_Ordinary_Names()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tFunction\tPublic\tParseOrder\t.\t+fn/0\n\
             item\t1\tStruct\tPublic\tOrder\t.\t+fields\\ntotal_count\\tusize\n",
        );

        let findings = Violations_In(&payload, "src/lib.rs", &[], &[]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// `OD-CAPABILITY-014` put a variable-length body behind an `impl` block's own
    /// trait-or-inherent label, so this carry stopped being an equality test against
    /// [`nomos_cap_syntax::TRAIT`]. `impl<T: Display> Display for T` is the shape that
    /// proves it: read the bare constant and its members lose an exemption the trait fixed.
    #[test]
    fn Test_Enclosing_Trait_Impl_Should_Carry_A_Generic_Trait_Impl()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tImplementation\tNotApplicable\tT\t.\t+trait\\ngenerics\\nT\n\
             item\t1\tImplementation\tNotApplicable\tTable\t.\t+inherent\\ngenerics\\nT\n",
        );

        let generic_trait_impl = payload.items.first().expect("the payload fixture declares two items");
        let generic_inherent_impl = payload.items.get(1).expect("the payload fixture declares two items");

        assert_eq!(Enclosing_Trait_Impl(generic_trait_impl, None), Some("T".to_owned()));
        assert_eq!(Enclosing_Trait_Impl(generic_inherent_impl, None), None);
    }

    fn Payload_From_Text(text: &str) -> SyntaxPayload
    {
        return nomos_cap_syntax::Parse_Payload(text.as_bytes()).expect("this fixture payload is well formed");
    }
}

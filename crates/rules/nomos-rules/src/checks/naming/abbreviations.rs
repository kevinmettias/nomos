//! Abbreviated words, ported from code-standards' `kernel/config/words/lists.go` and its
//! own `defaults.go`.
//!
//! A word is judged, not a whole name: [`Split_Identifier_Words`] breaks a declared
//! identifier at case boundaries and separators (`_`/`-`) the identical way `text.
//! Split_Identifier_Words` does, and each resulting word is checked in isolation against a
//! fixed, faithfully-ported default vocabulary — banned truncations, approved shortenings —
//! before the vowel rule (`is_Vowelless`) runs as the catch-all. A word carrying a digit is
//! judged by its letter core with the digits stripped, so `utf8`/`sha256` pass while
//! `gp400`/`md5` are still caught. `Use_Dictionary` (layer C, a broader real-word check) is
//! off by default upstream and not ported: it needs a dictionary this crate does not carry.
//!
//! The default vocabulary itself is not a repository's to configure — code-standards
//! ships it, every repository starts from it — but `nomos.cap.words.policy` carries a
//! repository's own *additions* to the approved half, the same optional-capability shape
//! [`super::Resolve_Case`] and `checks::structure::Resolve_Limit` already establish:
//! `facts.Require` failing for any reason means no additions, never a `Finding`.

use crate::SourceFile;
use nomos_analysis::{FactReader, InputDigest};
use nomos_cap_syntax::{IMPLEMENTATION, Impl_Serves_A_Trait, PayloadItem, Struct_Fields, SyntaxPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

/// This rule's own identifier, matching the code-standards rule id.
pub const ABBREVIATIONS: &str = "abbreviations";

const STRUCT: &str = "Struct";

/// The payload's `kind` for a `use` binding.
///
/// Named locally rather than imported for the same reason [`STRUCT`] is: `nomos-cap-syntax`
/// publishes an *open* kind vocabulary and exports a constant only for the labels its own
/// API needs, so a rule that cares about a third one states the literal it is matching.
///
/// A `use` binding is a reference to a declaration made somewhere else. Its name was chosen
/// wherever that declaration lives, and that is where this rule judges it -- reporting it
/// again at every import asks an author to rename something they do not own, and reports the
/// same name once per file that imports it. It was 144 of this rule's 327 findings against
/// this workspace, all of them `PathBuf`.
const USE_BINDING: &str = "Use";

/// An `extern crate` declaration names the crate being linked, not a name chosen at this
/// site -- the identical reason [`USE_BINDING`] is exempt, one level up: the real name was
/// fixed wherever that crate itself was published. `P68-ABBREVIATIONS-DOES-NOT-EXEMPT-
/// EXTERN-CRATE` measured this directly: `extern crate alloc;` reported `alloc` as a chosen
/// abbreviation, invisible from this workspace's own tree (edition 2021, no `extern crate`
/// statements) but a real, still-common idiom elsewhere.
const EXTERN_CRATE: &str = "ExternCrate";
const MINIMUM_JUDGED_WORD_LENGTH: usize = 2;
const VOWELS: &str = "aeiouy";

/// code-standards' `default_Approved_Words` — common acronyms, broadly recognized
/// programming shortenings, exact domain terms and proper nouns. Ported verbatim from
/// `defaults.go`.
const DEFAULT_APPROVED_WORDS: &[&str] = &[
    // universal acronyms
    "id", "ok", "url", "uri", "http", "https", "html", "css", "xml", "json", "api", "io", "os", "db", "ui", "ux",
    "cpu", "gpu", "ram", "usb", "pdf", "png", "jpg", "jpeg", "gif", "svg", "sql", "csv", "uuid", "guid", "2d", "3d",
    "ascii", "utf", "rgb", "rgba", "dns", "tcp", "udp", "ip", "ssl", "tls", "cli",
    // broadly-recognized programming shortenings
    "config", "info", "sync", "async", "await", "auth", "init", "app", "spec", "max", "min", "dto", "crud", "jwt",
    "mvc", "mvvm", "orm",
    // file-format, query-language and protocol acronyms
    "xlsx", "md", "jql", "mcp", "dlq", "ddl", "enum", "mutex", "token", "lexer", "parser", "ast", "node", "kind",
    "lerp", "aabb", "lod", "fps", "nunit", "xunit", "mstest", "serilog", "linq", "wpf", "xaml", "grpc",
    // Rust language vocabulary: exact keywords and core-library type/trait names
    "impl", "fn", "struct", "trait", "mut", "vec", "iter", "slice", "tuple", "variant", "generic", "deref",
    "lifetime", "arc", "cell", "closure", "expr", "decl", "stmt", "attr", "opcode",
    // Rust primitive numeric types
    "f32", "f64",
];

/// code-standards' `default_Banned_Words` — known truncations that must be spelled out.
/// Ported verbatim from `defaults.go`, including `impl`'s presence here: it is also in
/// [`DEFAULT_APPROVED_WORDS`], which `Abbreviation_Reason` checks first, so the banned
/// entry never fires — a real upstream redundancy, not a port error.
const DEFAULT_BANNED_WORDS: &[&str] = &[
    "rel", "abs", "idx", "prev", "cur", "curr", "dup", "elem", "param", "params", "opt", "opts", "calc", "eval",
    "addr", "sep", "delim", "req", "resp", "num", "dir", "env", "temp", "val", "vals", "len", "obj", "dest", "pos",
    "coord", "coords", "attrib", "attribs", "def", "gen", "alloc", "dealloc", "recv", "sig", "reg", "seg", "ret",
    "util", "utils", "misc", "aux", "exe", "msg", "buf", "doc", "docs", "repo", "impl",
];

/// Judges declared item names and named struct fields against the approved/banned/vowel
/// vocabulary, a repository's own `nomos.cap.words.policy` additions extending the
/// approved half.
#[must_use]
pub fn Check_Abbreviations(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let additions = Resolve_Approved_Additions(facts);
    let mut findings = Vec::new();

    for source in sources
    {
        match super::reading::Payload_Of(source, facts)
        {
            Ok(payload) =>
            {
                let violations = Violations_In(&payload, &source.path, &additions);
                findings.extend(violations);
            }
            Err(finding) => findings.push(Unread_As_This_Rule(finding)),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Resolves a repository's own additional approved words — empty on any `Require`
/// failure, per `OD-CAPABILITY-004`/`OD-RULES-011`'s settled optional-read pattern: this
/// capability is optional, and its absence must never surface as a `Finding` or this
/// capability's own `Applicability`.
fn Resolve_Approved_Additions(facts: &mut dyn FactReader) -> Vec<String>
{
    let subject = nomos_model::Subject_Of_Path("");
    let Ok(fact) =
        facts.Require(&nomos_cap_words_policy::Capability(), &subject, InputDigest::Of(&[]), &Words_Policy_Requirement())
    else
    {
        return Vec::new();
    };

    let Ok(payload) = nomos_cap_words_policy::Parse_Payload(&fact.payload.bytes) else { return Vec::new() };

    return payload.approved_additions;
}

/// This crate's own floor for `nomos.cap.words.policy` — stated at the capability's own
/// ceiling since there is only one real provider today and no weaker answer this crate
/// could honestly still act on. Mirrors `checks::naming::Naming_Policy_Requirement` and
/// `checks::structure::Limits_Policy_Requirement` exactly, for the fourth sibling
/// capability.
fn Words_Policy_Requirement() -> nomos_capability::Requirement
{
    return nomos_capability::Requirement::New(
        nomos_cap_words_policy::Capability(),
        nomos_cap_words_policy::CONTRACT_VERSION,
        nomos_cap_words_policy::Ceiling(),
    );
}

fn Violations_In(payload: &SyntaxPayload, path: &str, additions: &[String]) -> Vec<Finding>
{
    let mut findings = Vec::new();
    let mut enclosing_trait_impl: Option<String> = None;

    for item in &payload.items
    {
        let violations = Item_Violations_In(item, path, additions, &mut enclosing_trait_impl);
        findings.extend(violations);
    }

    return findings;
}

/// One item's own violations: whether it is exempt as an unowned trait-member name or a
/// `use` binding, then its own name and (for a struct) its fields, against the vocabulary.
/// `enclosing_trait_impl` is carried and updated in place, the same "an `impl` block resets
/// it" rule [`Enclosing_Trait_Impl`] states, so each item in call order sees the block its
/// predecessor left behind.
fn Item_Violations_In(
    item: &PayloadItem,
    path: &str,
    additions: &[String],
    enclosing_trait_impl: &mut Option<String>,
) -> Vec<Finding>
{
    *enclosing_trait_impl = Enclosing_Trait_Impl(item, enclosing_trait_impl.take());
    if Is_Exempt(item, enclosing_trait_impl.as_deref())
    {
        return Vec::new();
    }

    return Own_And_Field_Violations(item, path, additions);
}

/// The qualified name of the trait-serving `impl` block whose members `item` and everything
/// after it may belong to, carried forward from `previous` when `item` is not itself an
/// `impl` block.
///
/// An `impl` block resets the carry unconditionally, including an inherent one: two blocks
/// for the same type are indistinguishable by name, so `impl Display for Table` followed by
/// `impl Table` has to stop exempting at the second block or the inherent block's own
/// methods would inherit the first's exemption.
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
/// block at `enclosing_trait_impl` (a name the trait fixed, not the author), a `use`
/// binding, or an `extern crate` declaration (both a name chosen wherever the thing bound
/// or linked was actually declared, not here).
fn Is_Exempt(item: &PayloadItem, enclosing_trait_impl: Option<&str>) -> bool
{
    if enclosing_trait_impl.is_some_and(|block| return Is_Member_Of(item, block))
    {
        return true;
    }

    return item.kind == USE_BINDING || item.kind == EXTERN_CRATE;
}

/// Whether `item` is declared inside the `impl` block at `block`.
///
/// Nesting is the only signal there is: the provider qualifies a member by the scope it
/// pushed for the block, so `impl Display for Table`'s `fmt` arrives as `Table::fmt`. A
/// later sibling at the same level (`Other`) does not carry the prefix and so falls out of
/// the exemption without anything having to pop it.
fn Is_Member_Of(item: &PayloadItem, block: &str) -> bool
{
    return item.qualified_name.starts_with(&format!("{block}::"));
}

/// `item`'s own name, and (for a struct) its fields, against the vocabulary.
fn Own_And_Field_Violations(item: &PayloadItem, path: &str, additions: &[String]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    if let Some((word, reason)) = First_Abbreviation(item.Own_Name(), additions)
    {
        let finding = Violation_Finding(path, item, item.Own_Name(), (&word, reason));
        findings.push(finding);
    }

    if item.kind == STRUCT
    {
        let field_violations = Field_Violations_In(path, item, additions);
        findings.extend(field_violations);
    }

    return findings;
}

fn Field_Violations_In(path: &str, item: &PayloadItem, additions: &[String]) -> Vec<Finding>
{
    let Some(fields) = Struct_Fields(&item.shape) else { return Vec::new() };

    return fields
        .iter()
        .filter_map(|(name, _type_name)| {
            let (word, reason) = First_Abbreviation(name, additions)?;
            return Some(Violation_Finding(path, item, name, (&word, reason)));
        })
        .collect();
}

fn Unread_As_This_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(ABBREVIATIONS);
    finding.summary = finding
        .summary
        .replace("this file's naming could not be judged", "this file's abbreviations could not be judged");
    return finding;
}

/// The first abbreviated word inside `name`, and why — code-standards' own `Name_
/// Abbreviations` reports every one; a single `Finding` per subject only needs the first,
/// the same "report the first vague word" precedent `naming-clarity`'s own `Vague_Word`
/// sets upstream.
fn First_Abbreviation(name: &str, additions: &[String]) -> Option<(String, &'static str)>
{
    for word in Split_Identifier_Words(name)
    {
        if let Some(reason) = Abbreviation_Reason(&word, additions)
        {
            return Some((word, reason));
        }
    }

    return None;
}

/// Why `word` (already lowercased by [`Split_Identifier_Words`]) is an abbreviation, or
/// `None` if it is not. Ported from `abbreviation_Reason`: the denylist first, because a
/// listed word deserves the clearer message, then the vowel rule, which catches the
/// truncations nobody thought to list.
fn Abbreviation_Reason(word: &str, additions: &[String]) -> Option<&'static str>
{
    if Is_Too_Short_To_Judge(word) || Is_Approved(word, additions)
    {
        return None;
    }
    if DEFAULT_BANNED_WORDS.contains(&word)
    {
        return Some("known abbreviation; spell it out");
    }
    if Contains_Digit(word)
    {
        return Digit_Bearing_Abbreviation_Reason(word, additions);
    }
    if Is_Vowelless(word)
    {
        return Some("no vowels — an abbreviation; spell it out");
    }

    return None;
}

/// The reason a digit-bearing word is an abbreviation, judged on its letter core with the
/// digits stripped: `utf8` is weighed as `utf`, `gp400` as `gp`.
fn Digit_Bearing_Abbreviation_Reason(word: &str, additions: &[String]) -> Option<&'static str>
{
    let core = Letters_Only(word);
    let core_is_not_a_flaggable_abbreviation = Is_Too_Short_To_Judge(&core) || Is_Approved(&core, additions) || !Is_Vowelless(&core);
    if core_is_not_a_flaggable_abbreviation
    {
        return None;
    }
    return Some("no vowels — an abbreviation; spell it out");
}

/// Whether `word` is too short to judge for abbreviation at all — named so the length
/// chain reads as a question, and so it can still sit inside a short-circuited `||` chain
/// as a single call rather than an unconditionally-evaluated pipeline.
fn Is_Too_Short_To_Judge(word: &str) -> bool
{
    return word.chars().count() < MINIMUM_JUDGED_WORD_LENGTH;
}

fn Is_Approved(word: &str, additions: &[String]) -> bool
{
    return DEFAULT_APPROVED_WORDS.contains(&word) || additions.iter().any(|addition| return addition == word);
}

fn Is_Vowelless(word: &str) -> bool
{
    return !word.chars().any(|character| return VOWELS.contains(character));
}

fn Contains_Digit(word: &str) -> bool
{
    return word.chars().any(|character| return character.is_ascii_digit());
}

/// `word` with its digits removed — the "core" a digit-bearing identifier is judged on, so
/// `utf8` is weighed as `utf` and `gp400` as `gp`.
fn Letters_Only(word: &str) -> String
{
    return word.chars().filter(|character| return character.is_alphabetic()).collect();
}

/// Splits `name` into its lowercase word segments — ported from `Split_Identifier_Words`:
/// breaks on case boundaries (a lower/digit followed by an upper), on underscores and
/// hyphens, and drops digit-only runs so `Vector2` yields just `vector`. An acronym run
/// (`HTTP`) stays one segment, which the vocabulary checks treat as a single word.
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

fn Violation_Finding(path: &str, item: &PayloadItem, name: &str, abbreviation: (&str, &str)) -> Finding
{
    use nomos_model::Content_Digest;

    let (word, reason) = abbreviation;
    let qualified = format!("{path}::{}::{name}", item.qualified_name);

    return Finding {
        rule: RuleId::New(ABBREVIATIONS),
        subject: SubjectId::From_Digest(Content_Digest(qualified.as_bytes())),
        subject_name: name.to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("`{name}` contains the word `{word}`, {reason}"),
        locations: vec![path.to_owned()],
    };
}

#[cfg(test)]
mod tests;

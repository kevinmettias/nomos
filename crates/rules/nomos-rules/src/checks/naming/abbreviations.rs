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
use nomos_cap_syntax::{PayloadItem, Struct_Fields, SyntaxPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

/// This rule's own identifier, matching the code-standards rule id.
pub const ABBREVIATIONS: &str = "abbreviations";

const STRUCT: &str = "Struct";
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
            Ok(payload) => findings.extend(Violations_In(&payload, &source.path, &additions)),
            Err(finding) => findings.push(Unread_As_This_Rule(finding)),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Violations_In(payload: &SyntaxPayload, path: &str, additions: &[String]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for item in &payload.items
    {
        if let Some((word, reason)) = First_Abbreviation(item.Own_Name(), additions)
        {
            findings.push(Violation_Finding(path, item, item.Own_Name(), &word, reason));
        }

        if item.kind == STRUCT
        {
            findings.extend(Field_Violations_In(path, item, additions));
        }
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
            return Some(Violation_Finding(path, item, name, &word, reason));
        })
        .collect();
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
    if word.chars().count() < MINIMUM_JUDGED_WORD_LENGTH || Is_Approved(word, additions)
    {
        return None;
    }
    if DEFAULT_BANNED_WORDS.contains(&word)
    {
        return Some("known abbreviation; spell it out");
    }
    if Contains_Digit(word)
    {
        let core = Letters_Only(word);
        if core.chars().count() < MINIMUM_JUDGED_WORD_LENGTH || Is_Approved(&core, additions) || !Is_Vowelless(&core)
        {
            return None;
        }
        return Some("no vowels — an abbreviation; spell it out");
    }
    if Is_Vowelless(word)
    {
        return Some("no vowels — an abbreviation; spell it out");
    }

    return None;
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

fn Violation_Finding(path: &str, item: &PayloadItem, name: &str, word: &str, reason: &str) -> Finding
{
    use nomos_model::Content_Digest;

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

fn Unread_As_This_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(ABBREVIATIONS);
    finding.summary = finding
        .summary
        .replace("this file's naming could not be judged", "this file's abbreviations could not be judged");
    return finding;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Split_Identifier_Words_Should_Split_On_Case_Boundaries()
    {
        assert_eq!(Split_Identifier_Words("computeTotal"), vec!["compute", "total"]);
        assert_eq!(Split_Identifier_Words("ComputeTotal"), vec!["compute", "total"]);
    }

    #[test]
    fn Test_Split_Identifier_Words_Should_Split_On_Underscores_And_Hyphens()
    {
        assert_eq!(Split_Identifier_Words("compute_total"), vec!["compute", "total"]);
        assert_eq!(Split_Identifier_Words("compute-total"), vec!["compute", "total"]);
    }

    #[test]
    fn Test_Split_Identifier_Words_Should_Drop_A_Digit_Only_Run()
    {
        // Not `Vector2` — despite the upstream doc comment's claim, `insert_Case_Boundaries`
        // never separates a letter run from a digit run that follows it directly (only an
        // upper-case letter following a lower/digit gets a boundary), so `Vector2` stays one
        // field, `vector2`, all the way through. A digit-only run is only its own field, and
        // so only dropped, when something else already separates it: an explicit `_`/`-`, or
        // a case boundary on its OTHER side.
        assert_eq!(Split_Identifier_Words("Vector2"), vec!["vector2"]);
        assert_eq!(Split_Identifier_Words("item_2"), vec!["item"]);
    }

    #[test]
    fn Test_Split_Identifier_Words_Should_Keep_An_Acronym_Run_As_One_Segment()
    {
        assert_eq!(Split_Identifier_Words("HTTP"), vec!["http"]);
    }

    #[test]
    fn Test_Abbreviation_Reason_Should_Flag_A_Vowelless_Word()
    {
        assert_eq!(Abbreviation_Reason("ctx", &[]), Some("no vowels — an abbreviation; spell it out"));
    }

    #[test]
    fn Test_Abbreviation_Reason_Should_Flag_A_Banned_Word()
    {
        assert_eq!(Abbreviation_Reason("msg", &[]), Some("known abbreviation; spell it out"));
    }

    #[test]
    fn Test_Abbreviation_Reason_Should_Accept_A_Default_Approved_Word()
    {
        assert_eq!(Abbreviation_Reason("json", &[]), None);
    }

    #[test]
    fn Test_Abbreviation_Reason_Should_Accept_A_Word_Approved_By_A_Repository_Addition()
    {
        assert_eq!(Abbreviation_Reason("kwb", &["kwb".to_owned()]), None);
    }

    #[test]
    fn Test_Abbreviation_Reason_Should_Accept_A_Digit_Bearing_Word_Whose_Letter_Core_Has_A_Vowel()
    {
        assert_eq!(Abbreviation_Reason("utf8", &[]), None, "utf is approved, so utf8 passes");
        assert_eq!(Abbreviation_Reason("sha256", &[]), None, "sha has a vowel");
    }

    #[test]
    fn Test_Abbreviation_Reason_Should_Flag_A_Digit_Bearing_Word_Whose_Letter_Core_Is_Vowelless()
    {
        assert_eq!(
            Abbreviation_Reason("gp400", &[]),
            Some("no vowels — an abbreviation; spell it out"),
            "gp has no vowel even with the digits stripped"
        );
    }

    /// `md` is itself in the default approved list (`.md`, the Markdown extension), so
    /// `md5` is exempt by the same "approved core" reasoning that clears `utf8`/`sha256` —
    /// not by the vowel rule, which would flag `md` (no vowel) on its own. Upstream's own
    /// doc comment on `abbreviation_Reason` names `md5` as an example the vowel rule
    /// catches, which no longer holds once `md` was added to `default_Approved_Words` — a
    /// stale-doc drift the same shape as this session's other doc-vs-implementation finds,
    /// caught here by tracing the real data rather than the comment.
    #[test]
    fn Test_Abbreviation_Reason_Should_Accept_Md5_Because_Its_Core_Is_Approved()
    {
        assert_eq!(Abbreviation_Reason("md5", &[]), None);
    }

    #[test]
    fn Test_Abbreviation_Reason_Should_Accept_An_Ordinary_Voweled_Word()
    {
        assert_eq!(Abbreviation_Reason("total", &[]), None);
    }

    #[test]
    fn Test_Abbreviation_Reason_Should_Ignore_A_Single_Character_Word()
    {
        assert_eq!(Abbreviation_Reason("x", &[]), None);
    }

    #[test]
    fn Test_Violations_In_Should_Report_A_Vowelless_Item_Name()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPrivate\tCtx\t.\t+fn/0\n");

        let findings = Violations_In(&payload, "src/lib.rs", &[]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "Ctx");
    }

    #[test]
    fn Test_Violations_In_Should_Report_A_Banned_Struct_Field()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\nitem\t0\tStruct\tPublic\tConfig\t.\t+fields\\nmsg\\tString\\nbody\\tString\n",
        );

        let findings = Violations_In(&payload, "src/lib.rs", &[]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "msg");
    }

    #[test]
    fn Test_Violations_In_Should_Accept_Approved_And_Ordinary_Names()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tFunction\tPublic\tParseJson\t.\t+fn/0\n\
             item\t1\tStruct\tPublic\tConfig\t.\t+fields\\ntotal_count\\tusize\n",
        );

        let findings = Violations_In(&payload, "src/lib.rs", &[]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Payload_From_Text(text: &str) -> SyntaxPayload
    {
        return nomos_cap_syntax::Parse_Payload(text.as_bytes()).expect("this fixture payload is well formed");
    }
}

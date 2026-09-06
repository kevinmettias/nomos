//! The Rust facade-surface family from code-standards.
//!
//! Three published rule documents, one tool. code-standards' own `check-facade-surface`
//! enforces `facade-chooses-flattening-or-namespace`, `facade-aliases-name-the-contract`
//! and `facade-consumers-use-the-facade-path` from one binary because all three read the
//! same two statement shapes out of Rust source text -- `pub mod <child>;` and
//! `pub use <child>::<item>;` -- and disagree only about what they conclude from them.
//! They are split into three functions here for the same reason the atomic-ordering family
//! is: this crate's unit of export is the rule, not the tool that happened to ship it.
//!
//! # Why this is not a capability
//!
//! `OD-RULES-011` routes a rule's *parameters* through a repository's own declaration, and
//! none of these three has one. There is no threshold, no vocabulary and no convention a
//! repository would plausibly state differently -- a facade either publishes a child twice
//! or it does not. So this is a leaf module, the same answer the marker-comment family
//! reached, and not a fifth `nomos.cap.*.policy` contract.
//!
//! # What each rule decides
//!
//! [`Check_A_Facade_Publishes_A_Child_One_Way`] reports a file that both declares
//! `pub mod child;` and re-exports through `pub use child::...`, which makes
//! `parent::child::Thing` and `parent::Thing` both public and leaves callers to divide
//! between them. Restricted visibility counts: the rule document says so explicitly, and a
//! `pub(crate)` facade is still a boundary.
//!
//! [`Check_A_Renamed_Facade_Re_Export_Names_The_Contract`] reports
//! `pub use child::Internal as Public;` carrying no `facade-alias: allow` reason. The
//! alias is a public name, so the rule asks for the contract it states rather than
//! forbidding it. The marker is read the way `concurrency_text` and `error_text` already
//! read theirs -- leading a `//` comment on the statement's own line, or on a contiguous
//! run of comment, attribute and blank lines immediately above it -- rather than by
//! reimplementing code-standards' own marker package a third time.
//!
//! [`Check_A_Consumer_Imports_Through_The_Facade`] is the only rule here that reads more
//! than one file: it first collects every `pub use <child>::<item>;` a source under a
//! `src` directory publishes, deriving that source's own module path from its location,
//! and then reports any plain `use` statement that names the child path the facade was
//! supposed to hide.
//!
//! # Two deliberate narrowings, both matching this crate rather than the Go tool
//!
//! Every judgment is line-local over comment-stripped text, so a `use` statement broken
//! across lines is not decided. code-standards' own regex nominally spans lines, but its
//! braced comparisons are written against a single-line spelling and do not survive the
//! newline either, so this is a narrowing in form more than in effect -- and it is the
//! convention every other text rule in this crate already follows.
//!
//! A path segment list of exactly two (`child::item`) is what makes a re-export a facade
//! export, exactly as the Go implementation requires. A braced re-export
//! (`pub use child::{One, Two};`) publishes a surface this port does not model, and is
//! left undecided rather than guessed at.
//!
//! # Measured against this workspace before it was written
//!
//! Zero double-publications; twelve consumer findings over ten import lines reaching around
//! a crate-root facade, two of those lines bypassing two facades at once; and fifty-nine
//! unexplained aliases -- the last because `pub use id::Id as EntityId;` is this
//! workspace's own settled idiom for a one-type module. That is a real disagreement
//! between two standards and not a defect in either, which is why this item lands the
//! rules and leaves composition into a real run to a later increment that can weigh it.
//!
//! One import line can be reported more than once on purpose: a finding names the facade it
//! reached around, so two facades publishing the same child and item are two different
//! things to say about one line rather than one thing said twice. code-standards' own tool
//! reports them the same way.
//!
//! None of the three can match this file: it declares no `pub mod`, re-exports nothing,
//! and its own `use` lines name no `crate`-rooted child path. The self-exemption
//! `rust_text`, `security_text`, `concurrency_text` and `error_text` each carry is
//! therefore absent here on purpose rather than by oversight.

use super::code_prefix::Code_Prefix;
use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// The code-standards double-publication rule id.
pub const FACADE_CHOOSES_FLATTENING_OR_NAMESPACE: &str = "facade-chooses-flattening-or-namespace";
/// The code-standards renamed-re-export rule id.
pub const FACADE_ALIASES_NAME_THE_CONTRACT: &str = "facade-aliases-name-the-contract";
/// The code-standards consumer-import rule id.
pub const FACADE_CONSUMERS_USE_THE_FACADE_PATH: &str = "facade-consumers-use-the-facade-path";

/// The literal marker a justified facade alias must lead its reason comment with.
const FACADE_ALIAS_MARKER: &str = "facade-alias: allow";

/// The directory whose contents are a crate's module tree.
const SOURCE_DIRECTORY: &str = "src";

/// A `pub use <child>::<item>;` a facade publishes, with both paths the rule compares.
struct FacadeExport
{
    /// The publishing module's own crate-relative path, `::`-joined; empty at a crate root.
    facade: String,
    /// The child module the item really lives in.
    child: String,
    /// The item's own name.
    item: String,
    /// The path callers are meant to use.
    canonical: String,
    /// The path the facade was hiding.
    bypass: String,
}

/// Reports a Rust file that exposes one child module both as a public module and through a
/// public re-export, so the same item is reachable by two public paths.
#[must_use]
pub fn Check_A_Facade_Publishes_A_Child_One_Way(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE)
        {
            findings.extend(Double_Publication_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Double_Publication_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let code_lines = Code_Lines(&source.text);
    let published: Vec<&str> = code_lines.iter().filter_map(|line| return Public_Module_Name(line)).collect();

    let mut findings = Vec::new();
    for (index, line) in code_lines.iter().enumerate()
    {
        let finding = Double_Publication_Finding_For_Line(source, line, index, &published);
        if let Some(finding) = finding
        {
            findings.push(finding);
        }
    }

    return findings;
}

/// The module name a `pub mod <name>;` line declares. An inline `pub mod name { ... }` block
/// declares no child file and is not one.
fn Public_Module_Name(code: &str) -> Option<&str>
{
    let after_visibility = After_Public_Visibility(code)?;
    let after_keyword = Keyword_Body(after_visibility, "mod")?;
    let (name, after_name) = Leading_Identifier(after_keyword)?;

    if !after_name.trim_start().starts_with(';')
    {
        return None;
    }

    return Some(name);
}

fn Double_Publication_Finding_For_Line(source: &SourceFile, line: &str, index: usize, published: &[&str]) -> Option<Finding>
{
    let child = Public_Use_Head(line)?;
    if !published.contains(&child)
    {
        return None;
    }

    let line_number = Line_Number(index);
    return Some(Finding_At(
        source,
        FACADE_CHOOSES_FLATTENING_OR_NAMESPACE,
        line_number,
        &format!(
            "publishes `{child}` both as a public module and through a public re-export; a facade either \
             publishes the child namespace with `pub mod` or keeps it private and lifts selected items with \
             `pub use`, never both"
        ),
    ));
}

/// The child module a `pub use <child>::...` line re-exports through.
fn Public_Use_Head(code: &str) -> Option<&str>
{
    let after_visibility = After_Public_Visibility(code)?;
    let after_keyword = Keyword_Body(after_visibility, "use")?;
    let rest = after_keyword.strip_prefix("self::").unwrap_or(after_keyword);
    let (head, after_head) = Leading_Identifier(rest)?;

    if !after_head.starts_with("::")
    {
        return None;
    }

    return Some(head);
}

/// Reports a renamed public re-export carrying no adjacent `facade-alias: allow` reason --
/// the alias is a public name, so it owes the contract it states.
#[must_use]
pub fn Check_A_Renamed_Facade_Re_Export_Names_The_Contract(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE)
        {
            findings.extend(Unexplained_Alias_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Unexplained_Alias_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let code_lines = Code_Lines(&source.text);
    let raw_lines: Vec<&str> = source.text.lines().collect();

    let mut findings = Vec::new();
    for (index, line) in code_lines.iter().enumerate()
    {
        let finding = Unexplained_Alias_Finding_For_Line(source, line, index, &raw_lines);
        if let Some(finding) = finding
        {
            findings.push(finding);
        }
    }

    return findings;
}

fn Unexplained_Alias_Finding_For_Line(source: &SourceFile, line: &str, index: usize, raw_lines: &[&str]) -> Option<Finding>
{
    let re_export = Re_Export(line)?;
    let alias = re_export.alias?;
    if Has_Marker_Reason(raw_lines, index)
    {
        return None;
    }

    let line_number = Line_Number(index);
    return Some(Finding_At(
        source,
        FACADE_ALIASES_NAME_THE_CONTRACT,
        line_number,
        &format!(
            "renames the facade re-export to `{alias}` and carries no adjacent `{FACADE_ALIAS_MARKER}` reason; an \
             alias is a public name, so either rename the item at its declaration or state the contract this \
             boundary name gives it"
        ),
    ));
}

/// The statement's own line, or a contiguous run of blank, comment and attribute lines
/// walking upward from it, carries the literal `facade-alias: allow` marker with a non-empty
/// reason -- the same shape `concurrency_text` and `error_text` read their own markers with.
fn Has_Marker_Reason(lines: &[&str], index: usize) -> bool
{
    if lines.get(index).is_some_and(|line| return Marker_Reason_In(line).is_some_and(|reason| return !reason.is_empty()))
    {
        return true;
    }

    return Marker_Reason_Found_Above(lines, index);
}

/// Walks upward from `index` (exclusive) over a contiguous run of blank/comment/attribute
/// lines, stopping at the first line that is not skippable -- returning whether a marker
/// reason was found with a non-empty reason before that happened.
fn Marker_Reason_Found_Above(lines: &[&str], index: usize) -> bool
{
    let mut cursor = index;
    while cursor > 0
    {
        cursor = cursor.saturating_sub(1);
        let Some(line) = lines.get(cursor)
        else
        {
            break;
        };

        if let Some(reason) = Marker_Reason_In(line)
        {
            return !reason.is_empty();
        }

        if !Is_Skippable_Above(line)
        {
            break;
        }
    }

    return false;
}

fn Is_Skippable_Above(line: &str) -> bool
{
    let trimmed = line.trim();

    return trimmed.is_empty()
        || trimmed.starts_with("//")
        || trimmed.starts_with("/*")
        || trimmed.starts_with('*')
        || trimmed.starts_with("#[")
        || trimmed.starts_with("#![");
}

/// Reports an import that names a child path a facade in `sources` already re-exports,
/// reaching around the surface that facade published.
#[must_use]
pub fn Check_A_Consumer_Imports_Through_The_Facade(sources: &[SourceFile]) -> Vec<Finding>
{
    let exports = Facade_Exports(sources);
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE)
        {
            let bypass_findings = Bypass_Findings_In(source, &exports);
            findings.extend(bypass_findings);
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Every `pub use <child>::<item>;` published by a source that sits inside a crate's module
/// tree. A source outside a `src` directory has no module path to root a facade at, so it
/// publishes nothing this rule can compare an import against.
fn Facade_Exports(sources: &[SourceFile]) -> Vec<FacadeExport>
{
    let mut exports = Vec::new();

    for source in sources
    {
        let source_exports = Facade_Exports_In(source);
        exports.extend(source_exports);
    }

    return exports;
}

/// Every `pub use <child>::<item>;` one source publishes, or nothing when it is not Rust or
/// sits outside a crate's module tree.
fn Facade_Exports_In(source: &SourceFile) -> Vec<FacadeExport>
{
    let mut exports = Vec::new();

    if !source.Is_Written_In(RUST_LANGUAGE)
    {
        return exports;
    }
    let Some(facade_path) = Module_Path_For_Source(&source.path)
    else
    {
        return exports;
    };

    for line in Code_Lines(&source.text)
    {
        if let Some(export) = Facade_Export_For_Line(&line, &facade_path)
        {
            exports.push(export);
        }
    }

    return exports;
}

/// The crate-relative module path of a source inside a crate's source directory: empty at a
/// crate root (`lib.rs`, `main.rs`, or a `bin` entry point), otherwise its directories and
/// file stem with any trailing `mod` segment dropped, since `a/mod.rs` *is* module `a`.
/// `None` when the path is not a Rust file under a source directory at all.
fn Module_Path_For_Source(path: &str) -> Option<Vec<String>>
{
    let normalized = path.replace('\\', "/");
    let relative = Relative_To_Source_Directory(&normalized)?;
    let stem = relative.strip_suffix(".rs")?;

    let is_crate_root = stem == "lib" || stem == "main" || stem.starts_with("bin/");
    if is_crate_root
    {
        return Some(Vec::new());
    }

    let mut parts: Vec<String> = stem.split('/').map(|part| return part.strip_prefix("r#").unwrap_or(part).to_owned()).collect();
    if parts.last().is_some_and(|last| return last == "mod")
    {
        parts.pop();
    }

    return Some(parts);
}

fn Relative_To_Source_Directory(normalized: &str) -> Option<String>
{
    let leading = format!("{SOURCE_DIRECTORY}/");
    if let Some(rest) = normalized.strip_prefix(leading.as_str())
    {
        return Some(rest.to_owned());
    }

    let nested = format!("/{SOURCE_DIRECTORY}/");
    let start = normalized.find(nested.as_str())?;

    return normalized.get(start.saturating_add(nested.len())..).map(str::to_owned);
}

fn Facade_Export_For_Line(line: &str, facade_path: &[String]) -> Option<FacadeExport>
{
    let re_export = Re_Export(line)?;
    let [child, item] = re_export.path.as_slice()
    else
    {
        return None;
    };

    return Some(FacadeExport {
        facade: facade_path.join("::"),
        child: (*child).to_owned(),
        item: (*item).to_owned(),
        canonical: Crate_Path(facade_path, &[item]),
        bypass: Crate_Path(facade_path, &[child, item]),
    });
}

/// The crate root, the facade's own module path, then `tail`, all `::`-joined.
fn Crate_Path(facade_path: &[String], tail: &[&str]) -> String
{
    let mut parts: Vec<&str> = vec!["crate"];
    parts.extend(facade_path.iter().map(String::as_str));
    parts.extend_from_slice(tail);

    return parts.join("::");
}

fn Bypass_Findings_In(source: &SourceFile, exports: &[FacadeExport]) -> Vec<Finding>
{
    let code_lines = Code_Lines(&source.text);

    let mut findings = Vec::new();
    for (index, line) in code_lines.iter().enumerate()
    {
        let line_findings = Bypass_Findings_For_Line(source, line, index, exports);
        findings.extend(line_findings);
    }

    return findings;
}

fn Bypass_Findings_For_Line(source: &SourceFile, line: &str, index: usize, exports: &[FacadeExport]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    let Some(import) = Consumer_Import_Text(line)
    else
    {
        return findings;
    };

    for export in exports
    {
        if Is_Import_Bypassing_Facade(import, export)
        {
            let finding = Bypass_Finding(source, index, export);
            findings.push(finding);
        }
    }

    return findings;
}

/// The import text of a plain `use ...;` statement -- what a consumer binds to. A `pub use`
/// line is a facade publishing its own surface, not a consumer reaching for one, so it is
/// deliberately not matched here.
fn Consumer_Import_Text(code: &str) -> Option<&str>
{
    let after_keyword = Keyword_Body(code.trim_start(), "use")?;
    let (import, _terminator) = after_keyword.split_once(';')?;

    return Some(import.trim());
}

/// The four single-line spellings of an import that names a facade's hidden child path: the
/// bare path, the path renamed, the path inside a list, and the path opening a brace group.
/// The last subsumes code-standards' own two further variants, which differ from it only in
/// what follows the item name.
fn Is_Import_Bypassing_Facade(import: &str, export: &FacadeExport) -> bool
{
    let suffix = format!("::{}", export.item);
    let braced = format!("{}::{{{}", export.bypass.strip_suffix(&suffix).unwrap_or(&export.bypass), export.item);

    return import == export.bypass
        || import.contains(&format!("{} as ", export.bypass))
        || import.contains(&format!("{},", export.bypass))
        || import.contains(&format!("{}}}", export.bypass))
        || import.contains(&braced);
}

fn Bypass_Finding(source: &SourceFile, index: usize, export: &FacadeExport) -> Finding
{
    let line_number = Line_Number(index);

    return Finding_At(
        source,
        FACADE_CONSUMERS_USE_THE_FACADE_PATH,
        line_number,
        &format!(
            "reaches around the `{}` facade through `{}`; import `{}` so the binding is to the published \
             surface rather than to the file the item currently sits in",
            Facade_Label(&export.facade),
            export.child,
            export.canonical
        ),
    );
}

/// What to call a facade in a finding: a crate root publishes at the crate itself, and has
/// no module path of its own to name.
fn Facade_Label(facade: &str) -> &str
{
    if facade.is_empty()
    {
        return "crate";
    }

    return facade;
}

/// Every line of `text` with any `//` comment removed, so a commented-out declaration is
/// never read as a real one. The line count is preserved, so an index is still a line.
fn Code_Lines(text: &str) -> Vec<String>
{
    return text.lines().map(Code_Prefix).collect();
}

fn Line_Number(index: usize) -> usize
{
    return index.saturating_add(1);
}

/// A parsed single-item public re-export.
struct ReExport<'a>
{
    /// The path segments, raw-identifier prefixes removed.
    path: Vec<&'a str>,
    /// The alias, when the statement renames what it publishes.
    alias: Option<&'a str>,
}

/// The path and optional alias of a single-item public re-export. A brace group publishes a
/// surface this port does not model and is not one.
fn Re_Export(code: &str) -> Option<ReExport<'_>>
{
    let after_visibility = After_Public_Visibility(code)?;
    let after_keyword = Keyword_Body(after_visibility, "use")?;
    let rest = after_keyword.strip_prefix("self::").unwrap_or(after_keyword);

    let (path, rest) = Path_Segments(rest)?;
    let trailing = rest.trim_start();

    if let Some(after_as) = Keyword_Body(trailing, "as")
    {
        return Aliased_Re_Export(path, after_as);
    }

    if !trailing.starts_with(';')
    {
        return None;
    }

    return Some(ReExport { path, alias: None });
}

/// The re-export `path` renames to whatever leads `after_as` -- the text right after the
/// `as` keyword -- or `None` if that alias does not close on this line.
fn Aliased_Re_Export<'a>(path: Vec<&'a str>, after_as: &'a str) -> Option<ReExport<'a>>
{
    let (alias, after_alias) = Leading_Identifier(after_as)?;
    if !after_alias.trim_start().starts_with(';')
    {
        return None;
    }

    return Some(ReExport { path, alias: Some(alias) });
}

/// The `::`-separated leading path segments of `text`, and whatever follows the last one --
/// a raw-identifier prefix removed from each segment, the same as [`Leading_Identifier`].
fn Path_Segments(text: &str) -> Option<(Vec<&str>, &str)>
{
    let mut path = Vec::new();
    let mut rest = text;

    return loop
    {
        let (segment, after_segment) = Leading_Identifier(rest)?;
        path.push(segment);

        let Some(after_colons) = after_segment.strip_prefix("::")
        else
        {
            break Some((path, after_segment));
        };
        rest = after_colons;
    };
}

/// `code` with its leading whitespace and its `pub` or restricted-`pub` visibility removed.
/// Restricted visibility counts: `facade-chooses-flattening-or-namespace` says a facade is a
/// boundary even when the boundary is only inside the crate.
fn After_Public_Visibility(code: &str) -> Option<&str>
{
    let trimmed = code.trim_start();
    let after_pub = trimmed.strip_prefix("pub")?;

    let after_restriction = if let Some(inside) = after_pub.strip_prefix('(')
    {
        let close = inside.find(')')?;
        inside.get(close.saturating_add(1)..)?
    }
    else
    {
        after_pub
    };

    if !after_restriction.starts_with(char::is_whitespace)
    {
        return None;
    }

    return Some(after_restriction.trim_start());
}

/// What follows `keyword` in `text`, requiring whitespace after it so `used` is never read
/// as `use`.
fn Keyword_Body<'a>(text: &'a str, keyword: &str) -> Option<&'a str>
{
    let after = text.strip_prefix(keyword)?;

    if !after.starts_with(char::is_whitespace)
    {
        return None;
    }

    return Some(after.trim_start());
}

/// `text`'s leading Rust identifier -- any raw-identifier prefix removed, since `r#match`
/// and `match` name one module -- and whatever follows it.
fn Leading_Identifier(text: &str) -> Option<(&str, &str)>
{
    let body = text.strip_prefix("r#").unwrap_or(text);
    let end = Leading_Identifier_Byte_Length(body);

    if end == 0
    {
        return None;
    }

    return Some((body.get(..end)?, body.get(end..)?));
}

/// The byte length of the leading Rust identifier characters in `body`: ASCII-alphabetic or
/// `_` first, then ASCII-alphanumeric or `_`.
fn Leading_Identifier_Byte_Length(body: &str) -> usize
{
    let mut end = 0usize;

    for (offset, character) in body.char_indices()
    {
        let position = if offset == 0 { IdentifierPosition::First } else { IdentifierPosition::Rest };
        if !Is_Acceptable_Identifier_Character(character, position)
        {
            break;
        }
        end = offset.saturating_add(character.len_utf8());
    }

    return end;
}

/// Where a character sits in an identifier — the first character allows a narrower set
/// (no digits) than the rest, so a caller cannot silently pass the wrong test the way a
/// bare `bool` invites.
enum IdentifierPosition
{
    First,
    Rest,
}

fn Is_Acceptable_Identifier_Character(character: char, position: IdentifierPosition) -> bool
{
    return match position
    {
        IdentifierPosition::First => character.is_ascii_alphabetic() || character == '_',
        IdentifierPosition::Rest => character.is_ascii_alphanumeric() || character == '_',
    };
}

/// The marker must lead a comment, never merely appear somewhere on the line, so prose or a
/// string literal spelling its words cannot silence a real finding.
fn Marker_Reason_In(line: &str) -> Option<&str>
{
    let comment = line.split_once("//").map(|(_before, after)| return after)?;
    let after_marker = comment.trim_start().strip_prefix(FACADE_ALIAS_MARKER)?;
    let reason = after_marker.trim_start().strip_prefix(':').unwrap_or(after_marker);

    return Some(reason.trim());
}

fn Finding_At(source: &SourceFile, rule: &str, line_number: usize, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(rule),
        subject: source.subject,
        subject_name: format!("{}:{line_number}", source.path),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("{} line {line_number} {because}", source.path),
        locations: vec![format!("{}:{line_number}", source.path)],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_A_Facade_Publishes_A_Child_One_Way_Should_Report_A_Child_Published_Twice()
    {
        let source = Source("src/pipeline.rs", "pub mod pass_outcome;\npub use pass_outcome::PassOutcome;\n");

        let findings = Check_A_Facade_Publishes_A_Child_One_Way(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let reported = findings.first().expect("asserted len 1 above");
        assert_eq!(reported.rule, RuleId::New(FACADE_CHOOSES_FLATTENING_OR_NAMESPACE));
        assert_eq!(reported.subject_name, "src/pipeline.rs:2");
    }

    #[test]
    fn Test_Check_A_Facade_Publishes_A_Child_One_Way_Should_Accept_A_Private_Child_Lifted_Onto_The_Parent()
    {
        let source = Source("src/pipeline.rs", "mod pass_outcome;\npub use pass_outcome::PassOutcome;\n");

        let findings = Check_A_Facade_Publishes_A_Child_One_Way(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Facade_Publishes_A_Child_One_Way_Should_Accept_A_Published_Namespace_Alone()
    {
        let source = Source("src/pipeline.rs", "pub mod pass_outcome;\n");

        let findings = Check_A_Facade_Publishes_A_Child_One_Way(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Facade_Publishes_A_Child_One_Way_Should_Judge_Restricted_Visibility_Too()
    {
        let source = Source("src/pipeline.rs", "pub(crate) mod pass_outcome;\npub(crate) use pass_outcome::PassOutcome;\n");

        let findings = Check_A_Facade_Publishes_A_Child_One_Way(&[source]);

        assert_eq!(findings.len(), 1, "a facade is a boundary even inside the crate: {findings:?}");
    }

    #[test]
    fn Test_Check_A_Facade_Publishes_A_Child_One_Way_Should_Ignore_A_Commented_Out_Declaration()
    {
        let source = Source("src/pipeline.rs", "// pub mod pass_outcome;\npub use pass_outcome::PassOutcome;\n");

        let findings = Check_A_Facade_Publishes_A_Child_One_Way(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Facade_Publishes_A_Child_One_Way_Should_Not_Judge_A_File_In_Another_Language()
    {
        let source = Source("src/pipeline.go", "pub mod pass_outcome;\npub use pass_outcome::PassOutcome;\n");

        let findings = Check_A_Facade_Publishes_A_Child_One_Way(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Renamed_Facade_Re_Export_Names_The_Contract_Should_Report_An_Unexplained_Alias()
    {
        let source = Source("src/pipeline.rs", "pub use pass_outcome::Internal as PassOutcome;\n");

        let findings = Check_A_Renamed_Facade_Re_Export_Names_The_Contract(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").rule, RuleId::New(FACADE_ALIASES_NAME_THE_CONTRACT));
    }

    #[test]
    fn Test_Check_A_Renamed_Facade_Re_Export_Names_The_Contract_Should_Accept_A_Reason_Above_It()
    {
        let source = Source(
            "src/pipeline.rs",
            "// facade-alias: allow: PassOutcome is the surface vocabulary this crate publishes\npub use pass_outcome::Internal as PassOutcome;\n",
        );

        let findings = Check_A_Renamed_Facade_Re_Export_Names_The_Contract(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Renamed_Facade_Re_Export_Names_The_Contract_Should_Accept_A_Reason_Beside_It()
    {
        let source = Source(
            "src/pipeline.rs",
            "pub use pass_outcome::Internal as PassOutcome; // facade-alias: allow: the migration name callers still bind to\n",
        );

        let findings = Check_A_Renamed_Facade_Re_Export_Names_The_Contract(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Renamed_Facade_Re_Export_Names_The_Contract_Should_Refuse_A_Marker_With_No_Reason()
    {
        let source = Source("src/pipeline.rs", "// facade-alias: allow:\npub use pass_outcome::Internal as PassOutcome;\n");

        let findings = Check_A_Renamed_Facade_Re_Export_Names_The_Contract(&[source]);

        assert_eq!(findings.len(), 1, "a marker states a reason or it states nothing: {findings:?}");
    }

    #[test]
    fn Test_Check_A_Renamed_Facade_Re_Export_Names_The_Contract_Should_Accept_A_Re_Export_That_Renames_Nothing()
    {
        let source = Source("src/pipeline.rs", "pub use pass_outcome::PassOutcome;\n");

        let findings = Check_A_Renamed_Facade_Re_Export_Names_The_Contract(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Consumer_Imports_Through_The_Facade_Should_Report_An_Import_Around_The_Facade()
    {
        let facade = Source("src/pipeline.rs", "pub use pass_outcome::PassOutcome;\n");
        let consumer = Source("src/runner.rs", "use crate::pipeline::pass_outcome::PassOutcome;\n");

        let findings = Check_A_Consumer_Imports_Through_The_Facade(&[facade, consumer]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let reported = findings.first().expect("asserted len 1 above");
        assert_eq!(reported.rule, RuleId::New(FACADE_CONSUMERS_USE_THE_FACADE_PATH));
        assert!(reported.summary.contains("crate::pipeline::PassOutcome"), "names the canonical path: {reported:?}");
    }

    #[test]
    fn Test_Check_A_Consumer_Imports_Through_The_Facade_Should_Accept_The_Facade_Path()
    {
        let facade = Source("src/pipeline.rs", "pub use pass_outcome::PassOutcome;\n");
        let consumer = Source("src/runner.rs", "use crate::pipeline::PassOutcome;\n");

        let findings = Check_A_Consumer_Imports_Through_The_Facade(&[facade, consumer]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Consumer_Imports_Through_The_Facade_Should_Report_A_Braced_Import_Around_The_Facade()
    {
        let facade = Source("src/pipeline.rs", "pub use pass_outcome::PassOutcome;\n");
        let consumer = Source("src/runner.rs", "use crate::pipeline::pass_outcome::{PassOutcome, Other};\n");

        let findings = Check_A_Consumer_Imports_Through_The_Facade(&[facade, consumer]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Consumer_Imports_Through_The_Facade_Should_Root_A_Crate_Root_Facade_At_The_Crate()
    {
        let facade = Source("src/lib.rs", "pub use pass_outcome::PassOutcome;\n");
        let consumer = Source("src/runner.rs", "use crate::pass_outcome::PassOutcome;\n");

        let findings = Check_A_Consumer_Imports_Through_The_Facade(&[facade, consumer]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(
            findings.first().expect("asserted len 1 above").summary.contains("crate::PassOutcome"),
            "a crate root publishes at the crate itself: {findings:?}"
        );
    }

    #[test]
    fn Test_Check_A_Consumer_Imports_Through_The_Facade_Should_Read_A_Mod_File_As_Its_Own_Directory()
    {
        let facade = Source("src/pipeline/mod.rs", "pub use pass_outcome::PassOutcome;\n");
        let consumer = Source("src/runner.rs", "use crate::pipeline::pass_outcome::PassOutcome;\n");

        let findings = Check_A_Consumer_Imports_Through_The_Facade(&[facade, consumer]);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Check_A_Consumer_Imports_Through_The_Facade_Should_Publish_Nothing_From_Outside_A_Source_Directory()
    {
        let facade = Source("tests/harness.rs", "pub use pass_outcome::PassOutcome;\n");
        let consumer = Source("src/runner.rs", "use crate::pass_outcome::PassOutcome;\n");

        let findings = Check_A_Consumer_Imports_Through_The_Facade(&[facade, consumer]);

        assert!(findings.is_empty(), "a file outside the module tree roots no facade: {findings:?}");
    }

    #[test]
    fn Test_Check_A_Consumer_Imports_Through_The_Facade_Should_Not_Judge_The_Facades_Own_Re_Export()
    {
        let facade = Source("src/pipeline.rs", "mod pass_outcome;\npub use pass_outcome::PassOutcome;\n");

        let findings = Check_A_Consumer_Imports_Through_The_Facade(&[facade]);

        assert!(findings.is_empty(), "a facade publishing its own surface is not a consumer: {findings:?}");
    }

    #[test]
    fn Test_Re_Export_Should_Read_A_Raw_Identifier_As_The_Module_It_Names()
    {
        let parsed = Re_Export("pub use r#match::Matcher;").expect("a raw identifier is still a path segment");

        assert_eq!(parsed.path, vec!["match", "Matcher"]);
        assert_eq!(parsed.alias, None);
    }

    #[test]
    fn Test_Re_Export_Should_Leave_A_Brace_Group_Undecided()
    {
        assert!(Re_Export("pub use pass_outcome::{One, Two};").is_none());
    }

    fn Source(path: &str, text: &str) -> SourceFile
    {
        let mut source = SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
        source.language = crate::Recognized_Language_In_Tests(path);
        return source;
    }
}

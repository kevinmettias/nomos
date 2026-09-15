//! An import that reaches around a facade instead of through the path it published.
//!
//! Split out of [`super`], which states the family's shared reasoning. This is the only rule
//! of the three that reads more than one file: it collects every facade export in the source
//! set first, then judges each plain `use` statement against them.

use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::Finding;

use super::text::{Keyword_Body, Re_Export};
use super::{Code_Lines, FacadeExport, Finding_At, Line_Number, FACADE_CONSUMERS_USE_THE_FACADE_PATH, SOURCE_DIRECTORY};

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

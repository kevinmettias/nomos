//! A rule names only the contract half of a crate that bundles a provider with its contract.
//!
//! `OD-CAPABILITY-017` decided that bundling does not defeat the `Rules` boundary, because what
//! that boundary forbids is a rule **obtaining its own answer** rather than a rule sharing an
//! rlib with provider code. That decision rests on three clauses, and the first is load-bearing:
//! the rule consumes the contract half only.
//!
//! When that record was written the clause was true by measurement — `nomos-rules` called no
//! provider-side symbol of either bundled crate. A measurement is a fact about one afternoon.
//! Nothing stopped a later edit from calling `Fetch_Review_Comment` or `Materialize_Workspace`
//! directly: the crates are linked and the symbols are public, so the compiler would accept it
//! and no test would notice. The record named that debt; this module is it.
//!
//! # Where the halves come from
//!
//! Not from a list invented here. Each bundled crate's `lib.rs` re-exports every public symbol
//! through a `pub use` naming the module it comes from, so which half a symbol belongs to is
//! something the crate already states. [`Exports_By_Module`] reads those lines, and the forbidden
//! set is every symbol re-exported from a module that produces a fact.
//!
//! What is unavoidably a list is the classification of modules into the two halves, because
//! "produces a fact" is a judgment about what code does rather than something spelled in the
//! source. `HALVES` carries it, and it is checked in both directions so it cannot go stale in
//! either: a named module the crate no longer declares fails, and a module the crate gains that
//! is classified as neither half fails. A new provider module therefore cannot slip through
//! unjudged — it fails until somebody says which half it is.
//!
//! # Why the population is one crate and not two
//!
//! It was two. `OD-ROADMAP-005`'s fifth decision split `nomos.cap.review.finding`'s contract
//! out of `nomos-connector-coderabbit` into `nomos-cap-review-finding`, so that crate is not
//! bundled any more -- it declares no contract, it is Provider zone, and `Permits` forbids
//! `Rules` from naming it at all. Its entry is gone rather than kept: a row naming a crate
//! that is not bundled is a classification outliving its subject, which is the staleness
//! [`Test_Every_Module_Of_A_Bundled_Crate_Should_Be_Classified`] exists to refuse.
//!
//! What is left is `nomos-cap-requirement-trace`, still bundled, still named by
//! `nomos-rules`, so this guard still has a live subject and is not the
//! `OD-COMPLETENESS-001` shape -- a guard quantifying over an empty universe, passing
//! because there was nothing to judge. It is not retired, because retiring it would leave
//! `OD-CAPABILITY-017`'s first clause unenforced for the crate that still depends on it, and
//! the day a second bundled contract arrives it would have to be written again from the
//! record. Both tests below assert the population is non-empty rather than trusting that
//! somebody would notice, which is the assertion the list did not carry while it had two
//! members and could not go empty by one edit.
//!
//! # What this cannot do
//!
//! It reads source text, so it sees what `nomos-rules` *names*, not what it links. That is the
//! right target: `OD-CAPABILITY-017` already established that linking is not the property, and
//! naming is what a rule does when it starts obtaining its own answer.

use crate::bands::Repository_Root;
use std::collections::BTreeSet;
use std::iter::Peekable;
use std::str::Chars;

/// A bundled crate, and how its own modules divide into the two halves.
///
/// Fields are the crate's directory under `crates/`, the path `nomos-rules` spells it by, the
/// modules that produce a fact, and the modules that do not. The two lists together must be
/// exactly the modules the crate declares — checked in both directions by
/// [`Test_Every_Module_Of_A_Bundled_Crate_Should_Be_Classified`].
struct Bundled
{
    directory: &'static str,
    rust_path: &'static str,
    fact_producing: &'static [&'static str],
    contract: &'static [&'static str],
}

/// The bundled capability-contract crates `nomos-rules` is permitted to name.
///
/// `OD-CAPABILITY-002` licenses the bundling while a capability has exactly one provider;
/// `OD-CAPABILITY-015` puts such a crate in the Capability Contract zone because it declares a
/// contract. Neither record is restated here. One member, since `OD-ROADMAP-005`'s fifth
/// decision unbundled the other -- the module doc above says why that leaves a guard with a
/// subject rather than a guard with nothing to judge.
const HALVES: &[Bundled] = &[
    Bundled {
        directory: "crates/capabilities/nomos-cap-requirement-trace",
        rust_path: "nomos_cap_requirement_trace",
        // `provider` walks the tree and materializes the fact. `assessment`, `predicates` and
        // `registry` are parsing and judgment over content already in hand, which a rule may
        // hold: they produce no fact and reach no port.
        fact_producing: &["provider"],
        contract: &[
            "assessment",
            "contract",
            "guarantee",
            "payload",
            "predicates",
            "registry",
            "requirement_trace_fact_production",
        ],
    },
];

/// The crate root `nomos-rules` judges from.
const RULES_SOURCE: &str = "crates/rules/nomos-rules/src";

/// Every `mod x;` a `lib.rs` declares.
fn Declared_Modules(lib: &str) -> BTreeSet<String>
{
    let mut found = BTreeSet::new();

    for line in lib.lines()
    {
        let trimmed = line.trim();
        let body = trimmed.strip_prefix("pub mod ").or_else(|| trimmed.strip_prefix("mod "));

        if let Some(rest) = body
        {
            if let Some(name) = rest.strip_suffix(';')
            {
                found.insert(name.trim().to_string());
            }
        }
    }

    return found;
}

/// Every symbol a `lib.rs` re-exports, paired with the module it names.
///
/// Handles `pub use module::{A, B};`, `pub use module::A;` and `pub use module::inner::{A};` —
/// the last attributing to `module`, since that is the half the crate placed it in.
fn Exports_By_Module(lib: &str) -> Vec<(String, String)>
{
    let mut found = Vec::new();

    for line in lib.lines()
    {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("pub use ")
        else
        {
            continue;
        };

        let Some(rest) = rest.strip_suffix(';')
        else
        {
            continue;
        };

        let Some((head, tail)) = rest.split_once("::")
        else
        {
            continue;
        };

        let module = head.trim().to_string();

        for symbol in Symbols_In(tail)
        {
            found.push((module.clone(), symbol));
        }
    }

    return found;
}

/// Every symbol the tail of one `pub use module::…;` line names — the braces of
/// `{A, B}` split apart, or the single trailing name of `inner::A`.
///
/// Extracted from [`Exports_By_Module`]'s own loop rather than left inline: the brace
/// branch put a filter inside a loop inside a branch inside a loop, which `nesting-depth`
/// reports at four levels, and this is the second remedy that rule's own finding names.
/// Empty names are dropped here rather than at the call site for the same reason.
fn Symbols_In(tail: &str) -> Vec<String>
{
    let Some(open) = tail.find('{')
    else
    {
        let symbol = tail.rsplit("::").next().unwrap_or(tail).trim();

        return Named(symbol);
    };

    let Some(inner) = tail.get(open.saturating_add(1)..)
    else
    {
        return Vec::new();
    };

    let inner = inner.strip_suffix('}').unwrap_or(inner);

    return inner
        .split(',')
        .map(str::trim)
        .filter(|symbol| return !symbol.is_empty())
        .map(str::to_owned)
        .collect();
}

/// `symbol` as the one-element list a caller extends its own with, or an empty one when
/// the name is blank — the single-name half of [`Symbols_In`]'s own answer.
fn Named(symbol: &str) -> Vec<String>
{
    if symbol.is_empty()
    {
        return Vec::new();
    }

    return vec![symbol.to_owned()];
}

/// Every symbol `source` names through `prefix`, from `prefix::Symbol` and `use prefix::{A, B}`.
fn Symbols_Named_Through(source: &str, prefix: &str) -> BTreeSet<String>
{
    let needle = format!("{prefix}::");
    let mut found = BTreeSet::new();
    let mut rest = source;

    while let Some(offset) = rest.find(&needle)
    {
        let Some(after) = rest.get(offset.saturating_add(needle.len())..)
        else
        {
            break;
        };

        found.extend(Symbols_After(after));

        rest = after;
    }

    return found;
}

/// Every symbol named immediately after a `prefix::`, whether the text that follows opens a
/// `{A, B}` group or is a single bare name.
///
/// Extracted from [`Symbols_Named_Through`]'s own loop rather than left inline: three
/// `if let`s, a loop and a filter one inside another put the innermost line six levels deep,
/// which `nesting-depth` reports, and this is the second remedy that rule's own finding
/// names. Each failed match answers an empty list here, which is what the nested form said by
/// falling out of every branch.
fn Symbols_After(after: &str) -> Vec<String>
{
    let Some(tail) = after.strip_prefix('{')
    else
    {
        let symbol: String = after.chars().take_while(|c| return c.is_alphanumeric() || *c == '_').collect();

        return Named(&symbol);
    };

    let Some(close) = tail.find('}')
    else
    {
        return Vec::new();
    };

    let Some(inner) = tail.get(..close)
    else
    {
        return Vec::new();
    };

    return inner
        .split(',')
        .map(str::trim)
        .filter(|symbol| return !symbol.is_empty())
        .map(str::to_owned)
        .collect();
}

/// `source` with comments removed, leaving string and character literals intact.
///
/// Necessary, and found necessary the hard way: the first version of this module read raw text
/// and reported `Discover_Workspace` and `Materialize_Workspace` as called by `nomos-rules`.
/// They are named in `checks/requirement_trace.rs`'s own documentation, in prose explaining who
/// produces the fact the rule consumes — which is the opposite of calling them. A scanner that
/// cannot tell code from prose reports a rule for describing the boundary it respects.
///
/// Literals are tracked rather than skipped because the dangerous direction is the other one: a
/// `//` inside a string would otherwise blind the scanner to the rest of that line, and a real
/// call sitting there would go unseen.
fn Code_Only(source: &str) -> String
{
    let mut out = String::with_capacity(source.len());
    let mut characters = source.chars().peekable();
    let mut depth: usize = 0;

    while let Some(character) = characters.next()
    {
        if depth > 0
        {
            depth = Nesting_After(character, &mut characters, depth);
            continue;
        }

        match character
        {
            '/' if characters.peek() == Some(&'/') => Skip_Line_Comment(&mut characters, &mut out),
            '/' if characters.peek() == Some(&'*') =>
            {
                let _ = characters.next();
                depth = depth.saturating_add(1);
            }
            '"' =>
            {
                out.push(character);
                Copy_Literal(&mut characters, &mut out, '"');
            }
            '\'' =>
            {
                out.push(character);
                Copy_Literal(&mut characters, &mut out, '\'');
            }
            _ => out.push(character),
        }
    }

    return out;
}

/// The block-comment nesting depth after reading `character`, while already inside one.
///
/// Extracted from [`Code_Only`]'s own loop rather than left inline, the same remedy
/// `nesting-depth` names for the two helpers below it: the open and close tests sat inside the
/// depth branch inside the character loop, and every level here is one a reader had to hold to
/// know whether the next line ran.
fn Nesting_After(character: char, characters: &mut Peekable<Chars<'_>>, depth: usize) -> usize
{
    if character == '*' && characters.peek() == Some(&'/')
    {
        let _ = characters.next();

        return depth.saturating_sub(1);
    }

    if character == '/' && characters.peek() == Some(&'*')
    {
        let _ = characters.next();

        return depth.saturating_add(1);
    }

    return depth;
}

/// Consumes the rest of a line comment, keeping only the newline that ends it so line numbers
/// downstream still line up.
fn Skip_Line_Comment(characters: &mut Peekable<Chars<'_>>, out: &mut String)
{
    for next in characters.by_ref()
    {
        if next == '\n'
        {
            out.push('\n');
            break;
        }
    }
}

/// Copies a string or character literal through unchanged, `terminator` being the quote that
/// ends it.
///
/// One function for both, where the loop was written twice: the two differed only in which
/// quote closes them and in the character literal also ending at a newline, which is the
/// single-quote clause below. Copying rather than skipping is what this module's own doc
/// calls the dangerous direction -- a `//` inside a string must not blind the scanner to the
/// rest of that line.
fn Copy_Literal(characters: &mut Peekable<Chars<'_>>, out: &mut String, terminator: char)
{
    while let Some(next) = characters.next()
    {
        out.push(next);

        if next == '\\'
        {
            Copy_Escaped(characters, out);
            continue;
        }

        if next == terminator || (terminator == '\'' && next == '\n')
        {
            break;
        }
    }
}

/// Copies the character an escape introduces, if the literal has one left to give.
fn Copy_Escaped(characters: &mut Peekable<Chars<'_>>, out: &mut String)
{
    if let Some(escaped) = characters.next()
    {
        out.push(escaped);
    }
}

/// The fact-producing symbols of `crate_` that `source` names in code.
///
/// Comments are stripped first: naming a provider function in prose that explains who produces a
/// fact is not calling it, and reporting it would punish a rule for documenting the boundary.
fn Forbidden_Named(source: &str, crate_: &Bundled, lib: &str) -> BTreeSet<String>
{
    let source = &Code_Only(source);

    let forbidden: BTreeSet<String> = Exports_By_Module(lib)
        .into_iter()
        .filter(|(module, _)| crate_.fact_producing.contains(&module.as_str()))
        .map(|(_, symbol)| symbol)
        .collect();

    return Symbols_Named_Through(source, crate_.rust_path)
        .into_iter()
        .filter(|symbol| forbidden.contains(symbol))
        .collect();
}

/// Every `.rs` file under a directory, as one concatenated string.
fn Source_Under(root: &std::path::Path) -> String
{
    let mut text = String::new();
    let mut pending = vec![root.to_path_buf()];

    while let Some(directory) = pending.pop()
    {
        let Ok(entries) = std::fs::read_dir(&directory)
        else
        {
            continue;
        };

        for entry in entries.flatten()
        {
            let path = entry.path();

            if path.is_dir()
            {
                pending.push(path);
                continue;
            }

            // A guard clause rather than a nested `if`: the extension test and the read one
            // inside the other put this at four levels of control flow, which `nesting-depth`
            // reports, and flattening with an early `continue` is the first remedy it names.
            if !path.extension().is_some_and(|extension| return extension == "rs")
            {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(&path)
            {
                text.push_str(&content);
                text.push('\n');
            }
        }
    }

    return text;
}

/// `lib.rs` of a bundled crate.
fn Lib_Of(crate_: &Bundled) -> String
{
    let path = Repository_Root().join(crate_.directory).join("src/lib.rs");

    return std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
}

/// `HALVES` names at least one crate, and `nomos-rules` still depends on it.
///
/// Both assertions above iterate over `HALVES`, so an empty list passes them having judged
/// nothing -- `OD-COMPLETENESS-001`'s own subject, and the failure mode this list acquired
/// the day it went from two members to one. The dependency half matters as much as the
/// count: a row for a crate `nomos-rules` no longer names would keep the list non-empty
/// while the thing it guards had already stopped happening.
fn Assert_A_Bundled_Crate_Is_Still_Named()
{
    assert!(
        !HALVES.is_empty(),
        "HALVES is empty, so every assertion in this module quantifies over nothing and          passes having judged nothing. If the last bundled contract really was split out,          this module is retired deliberately and with a record saying so -- not left          standing as a green test over an empty universe."
    );

    let manifest = std::fs::read_to_string(Repository_Root().join(RULES_SOURCE).join("../Cargo.toml"))
        .expect("nomos-rules has a manifest");

    for crate_ in HALVES
    {
        let name = crate_.directory.rsplit('/').next().unwrap_or(crate_.directory);

        assert!(
            manifest.contains(name),
            "{name} is classified here as a bundled crate nomos-rules is permitted to name,              and nomos-rules does not depend on it. Then nothing in this module is guarding              anything about it: drop the row, or the dependency was removed and the row              outlived its subject."
        );
    }
}

/// Both halves together are exactly the modules the crate declares.
///
/// The both-directions check. A named module the crate dropped fails, and a module the crate
/// gained that nobody classified fails — so a new provider module cannot arrive unjudged and be
/// silently treated as contract half.
#[test]
fn Test_Every_Module_Of_A_Bundled_Crate_Should_Be_Classified()
{
    Assert_A_Bundled_Crate_Is_Still_Named();

    for crate_ in HALVES
    {
        let declared = Declared_Modules(&Lib_Of(crate_));

        let classified: BTreeSet<String> = crate_
            .fact_producing
            .iter()
            .chain(crate_.contract.iter())
            .map(|name| (*name).to_string())
            .collect();

        let unclassified: Vec<&String> = declared.difference(&classified).collect();
        let vanished: Vec<&String> = classified.difference(&declared).collect();

        assert!(
            unclassified.is_empty(),
            "{} declares modules no half claims: {unclassified:?}.\n\
             A module nobody classified would be treated as contract half by default, which is \
             how a new provider arrives without anybody deciding it may. Say which half it is.",
            crate_.directory
        );

        assert!(
            vanished.is_empty(),
            "{} no longer declares modules this classification names: {vanished:?}.\n\
             Drop them. A classification that outlives its subject is a list going stale in the \
             direction nothing would notice.",
            crate_.directory
        );
    }
}

/// `nomos-rules` names no fact-producing symbol of a bundled crate.
///
/// `OD-CAPABILITY-017`'s first clause, made checkable.
#[test]
fn Test_Rules_Should_Name_Only_The_Contract_Half_Of_A_Bundled_Crate()
{
    Assert_A_Bundled_Crate_Is_Still_Named();

    let source = Source_Under(&Repository_Root().join(RULES_SOURCE));

    for crate_ in HALVES
    {
        let named = Forbidden_Named(&source, crate_, &Lib_Of(crate_));

        assert!(
            named.is_empty(),
            "nomos-rules names fact-producing symbols of {}: {named:?}.\n\n\
             OD-CAPABILITY-017 permits a rule to name this crate only because the rule consumes \
             its contract half -- capability identity, contract version, ceiling, schema and \
             payload parsing -- and calls nothing that produces a fact. A rule that fetches, \
             translates or materializes its own evidence is obtaining its own answer, which is \
             what the Rules boundary forbids, and the bundling permission does not survive it.\n\n\
             Either take the fact through the capability registry the way the other rules do, or \
             the contract and the provider have to be split into separate crates.",
            crate_.directory
        );
    }
}

/// The judging rejects what it must, and accepts what it must.
///
/// The negative control. The one bundled crate satisfies the rule today, so the assertion
/// above is otherwise only known to pass on a tree that already agrees with it -- which is
/// indistinguishable from an assertion that cannot fail. `OD-GATE-028` is why this is not
/// optional.
///
/// Written against whichever crate `HALVES` holds first rather than a crate named here, so
/// that the control cannot survive the list it is a control for. It used to spell
/// `nomos_connector_coderabbit`, and when `OD-ROADMAP-005`'s fifth decision unbundled that
/// crate the literals had to move with the row; the `rust_path` below is read from the row
/// instead, so the next such move breaks nothing silently.
#[test]
fn Test_A_Rule_Naming_A_Fact_Producing_Symbol_Should_Be_Rejected()
{
    let Some(bundled) = HALVES.first()
    else
    {
        panic!("HALVES is empty, so this control proves nothing about a guard nothing runs");
    };

    let lib = Lib_Of(bundled);
    let path = bundled.rust_path;
    let produces = Some_Fact_Producing_Symbol(bundled, &lib);
    let reads = Some_Contract_Symbol(bundled, &lib);

    let obtains_its_own_answer = format!("let fact = {path}::{produces}(root, context, filesystem);");
    let reads_the_contract = format!("let held = {path}::{reads}();");
    let braced = format!("use {path}::{{{reads}, {produces}}};");

    assert!(
        !Forbidden_Named(&obtains_its_own_answer, bundled, &lib).is_empty(),
        "a rule calling {produces} was accepted. This assertion cannot fail, and a guard          that cannot fail is worse than no guard."
    );

    assert!(
        !Forbidden_Named(&braced, bundled, &lib).is_empty(),
        "a braced import of {produces} was accepted; the scanner misses the spelling a real          `use` statement would take."
    );

    assert!(
        Forbidden_Named(&reads_the_contract, bundled, &lib).is_empty(),
        "reading {reads} was rejected. The guard is wrong in the direction that would forbid          what OD-CAPABILITY-017 explicitly permits."
    );

    let describes_it_in_prose =
        format!("//! The fact arrives from `{path}::{produces}`,
//! which this rule never calls.
let x = 1;");

    assert!(
        Forbidden_Named(&describes_it_in_prose, bundled, &lib).is_empty(),
        "a doc comment naming a provider function was reported as a call. This is the false          positive the first version of this module actually produced against          checks/requirement_trace.rs, and it punishes a rule for documenting the boundary it          respects."
    );

    let url_in_a_string =
        format!("let u = \"https://example.invalid/x\"; let c = {path}::{produces}(root, context, filesystem);");

    assert!(
        !Forbidden_Named(&url_in_a_string, bundled, &lib).is_empty(),
        "a `//` inside a string literal blinded the scanner to a real call later on the same          line. That is the dangerous direction: the guard would go quiet rather than loud."
    );
}

/// One symbol the crate re-exports from a fact-producing module, whichever comes first.
fn Some_Fact_Producing_Symbol(crate_: &Bundled, lib: &str) -> String
{
    return Some_Symbol_From(crate_.fact_producing, lib)
        .unwrap_or_else(|| panic!("{} re-exports no fact-producing symbol, so the control has nothing to reject", crate_.directory));
}

/// One symbol the crate re-exports from a contract-half module, whichever comes first.
fn Some_Contract_Symbol(crate_: &Bundled, lib: &str) -> String
{
    return Some_Symbol_From(crate_.contract, lib)
        .unwrap_or_else(|| panic!("{} re-exports no contract-half symbol, so the control has nothing to accept", crate_.directory));
}

/// The first symbol re-exported from any of `modules`.
fn Some_Symbol_From(modules: &[&str], lib: &str) -> Option<String>
{
    return Exports_By_Module(lib)
        .into_iter()
        .find(|(module, _)| return modules.contains(&module.as_str()))
        .map(|(_, symbol)| return symbol);
}

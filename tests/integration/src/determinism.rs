//! The channel a parent and a child process compare one domain's digest over.
//!
//! # Why this is a harness and not four hand-written tests
//!
//! A declaration is a testable claim, and the verification it owes follows mechanically
//! from the triple it declared — `nomos_contracts::determinism` says so in prose, and
//! saying it in prose is what left four domains declaring nothing for as long as they
//! did. So the mapping from declaration to obligation is code, split across this module's
//! siblings, and a domain gets the checks its own declaration implies rather than the
//! checks its author remembered:
//!
//! | Declared | Obligation discharged, and where |
//! |---|---|
//! | any triple | the triple is internally coherent — [`crate::Verify`] |
//! | `STRENGTH: StateTemporal` | repeated production is byte-identical, order included — [`crate::Verify`] |
//! | `STRENGTH: State` | repeated production agrees as a set, order not promised — [`crate::Verify`] |
//! | `STRENGTH: None` | nothing — and `TRACE` must then be `NotApplicable` |
//! | `SCOPE` above `SingleRun` | a second process reaches the same bytes — [`crate::Cross_Environment_Owed`] |
//! | `SCOPE` at `CrossPlatform` or above | the bytes match a digest committed to the tree |
//!
//! Raising a declaration therefore raises what is checked, with no second edit. Lowering
//! one is the sanctioned way to make a failing domain honest, and it is visible in a diff
//! as a weakened promise rather than as a deleted test.
//!
//! What is left in this file is the part that is neither the verification nor the
//! obligation: the line a child prints and the parent reads, which is the only piece both
//! processes have to agree on.
//!
//! # Why the fixtures are in this repository
//!
//! `nomos-lang-rust/tests/corpus.rs` already asserts reading determinism over 7,500 real
//! files, and `nomos-workspace/tests/portable.rs` asserts snapshot identity over a hundred
//! ingestion orders of that corpus. Both are stronger instruments than anything here and
//! both are corpus-gated, so on every machine without `NOMOS_RUST_CORPUS` they report `ok`
//! having read nothing — which is every machine the gate runs on. `docs/records/OD-GATE-001`
//! measures that hole at 68 assertions.
//!
//! A declaration whose only proof is inside that hole is a declaration CI has never seen
//! tested. So these fixtures are small, sufficient and here.

/// The environment variable by which a parent hands a child the domain to report on.
///
/// Assembled rather than written, for the reason `tests/contract/src/gates.rs` assembles
/// the corpus variable names: a file that contains the spelling of a variable it also
/// scans for is a file that finds itself.
#[must_use]
pub fn Child_Variable() -> String
{
    return concat!("NOMOS_", "DETERMINISM_", "CHILD").to_owned();
}

/// The line a child process prints so its parent can read one domain's digest.
#[must_use]
pub fn Report_Line(domain: &str, digest: &str) -> String
{
    return format!("nomos-determinism\t{domain}\t{digest}");
}

/// Reads a digest for one domain out of a child process's output.
#[must_use]
pub fn Digest_In(output: &str, domain: &str) -> Option<String>
{
    for line in output.lines()
    {
        let mut fields = line.trim().split('\t');
        if fields.next() != Some("nomos-determinism")
        {
            continue;
        }
        if fields.next() != Some(domain)
        {
            continue;
        }
        if let Some(digest) = fields.next()
        {
            return Some(digest.to_owned());
        }
    }

    return None;
}

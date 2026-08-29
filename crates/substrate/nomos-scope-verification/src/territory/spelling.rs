//! Reducing a path spelling to the subject it names.

use super::{Content_Digest, SubjectId};

/// A record stem up to and including its ordinal component.
///
/// An identifier begins with a word. `2026-08-09-notes.md` reaches its first all-digit
/// component at `08` and would otherwise reduce to `2026-08`, which two unrelated notes
/// from the same month would then share. Requiring a letter first refuses the whole name
/// rather than the first component of it.
///
/// A stem that never reaches an all-digit component is not an allocation at all, which is
/// why running out of components answers with nothing rather than with the whole stem.
pub(super) fn Up_To_The_Ordinal(stem: &str) -> Option<String>
{
    let mut identifier = String::new();

    for (position, component) in stem.split('-').enumerate()
    {
        if position == 0 && !Has_A_Letter(component)
        {
            return None;
        }

        if position > 0
        {
            identifier.push('-');
        }
        identifier.push_str(component);

        if Is_Ordinal(component, position)
        {
            return Some(identifier);
        }
    }

    return None;
}

/// Whether a component carries at least one letter.
fn Has_A_Letter(component: &str) -> bool
{
    return component.bytes().any(|byte| return byte.is_ascii_alphabetic());
}

/// Whether a component is the ordinal that ends an identifier.
///
/// Never the first: a name that opens with digits is a date or a serial rather than an
/// allocation, and `2026` is not an identifier `2026-08-notes.md` extends.
pub(super) fn Is_Ordinal(component: &str, position: usize) -> bool
{
    return position > 0
        && !component.is_empty()
        && component.bytes().all(|byte| return byte.is_ascii_digit());
}

/// The identity of the subject a path denotes, for the purpose of holding it.
///
/// Deliberately not [`nomos_model::Subject_Of_Path`], which is what a *fact* is about.
/// The two agree on every path but one shape: this one folds a record filename onto the
/// identifier it carries, so an item that reserved `docs/records/OD-LEDGER-006` excludes
/// the holder of the file it became. See [`Normalize_Path`] for that rule and
/// `OD-MODEL-001` for why the difference is kept rather than settled either way.
#[must_use]
pub(crate) fn Subject_Of(path: &str) -> SubjectId
{
    return SubjectId::From_Digest(Content_Digest(Normalize_Path(path).as_bytes()));
}

/// Reduces an authored path to the text its identity is computed from.
///
/// Separators are unified, `./` prefixes and repeated or trailing separators are
/// dropped, the result is lowercased, and a record filename is reduced to the record
/// identifier it carries.
///
/// # Why the spelling half is not written here
///
/// Everything up to the record fold is [`nomos_model::Normalize_Path`], and it is called
/// rather than repeated. It used to be repeated — three times, here and in two fact
/// producers — and `OD-MODEL-001` records the decision to converge them. What a path
/// spells is the same question whether the answer addresses a claim or a fact, so the
/// reasoning for unifying separators and folding case now lives with the rule, in the
/// kernel.
///
/// What survives here is the half that is genuinely the ledger's, and the composition runs
/// in this direction — ledger over kernel, never a flag passed down — because the kernel
/// must not learn what a decision record is. A fact *about* a record file has to stay a
/// fact about that file.
///
/// # Why a record filename folds onto its identifier
///
/// `OD-LEDGER-001`'s authoring rule tells an item to reserve the record it will write *by
/// identifier*: `docs/records/OD-<AREA>-<NNN>`, not the directory records live in and not
/// a pattern. The reason it says identifier is good and has not changed — the slug on the
/// end of the filename is the writing, and the writing is not knowable when the item is
/// authored. So the identifier is the only name two items can both write down in advance,
/// which makes it the coordination token whether or not it is a path.
///
/// It is not a path. Nothing is ever at `docs/records/OD-LEDGER-006`; the file is
/// `docs/records/OD-LEDGER-006-a-reason-attached-to-a-transition-does-not-survive-it.md`.
/// Compared as paths the two are *siblings* — neither is a prefix of the other at a
/// separator — so before this rule an item reserving the identifier excluded nobody from
/// the file, and an item that named the filename outright excluded nobody who had
/// reserved the identifier. Two items could hold one record between them and both be told
/// the ledger was disjoint.
///
/// That is the same failure as `src/Main.rs` against `src/main.rs`, arriving by a
/// different route, and it has the same answer: **two spellings of one thing are one
/// subject.** A record has two spellings — the identifier it was allocated and the file it
/// became — and folding the second onto the first is what makes them one. Containment is
/// deliberately *not* where this lives. The identifier does not contain the record; it is
/// the record, and saying so here leaves [`Is_Overlapping`] the pure path relation it
/// has always been, and fixes [`Subject_Of`] — and so [`Territory::As_Subject_Set`] — at
/// the same time. A rule written into containment would have left the identity form still
/// answering that the two are unrelated.
///
/// # Why it is scoped to `docs/records`
///
/// Because the general version of it destroys the ledger. "A name contains the names that
/// extend it with a hyphen" would make `crates/nomos-spec` contain
/// `crates/nomos-spec-model`, and the whole repository would serialize — the exact case
/// [`Is_Overlapping`] appends a separator to avoid, and
/// `Test_A_Sibling_With_A_Shared_Prefix_Should_Not_Be_Contained` is the guard on it.
///
/// So the rule is confined to the one directory where the identifier-to-filename relation
/// is a stated fact rather than an inference from the shape of a name: every canonical
/// record's registration under `crates/spec/nomos-spec-store/records/<ID>.record` writes
/// that mapping down as a `path:` line, and `OD-SPEC-006` makes `docs/records` the
/// authoring substrate and nothing else. Outside it, `tests/fixtures/case-001` and
/// `tests/fixtures/case-001-expected.md` stay two subjects, because there the resemblance
/// really is a coincidence.
///
/// Inside it the residual risk is over-folding, not under-folding: a file in
/// `docs/records` whose name happens to fit the grammar without being a record would fold
/// onto a name it does not mean, and two items would serialize over a record neither is
/// writing. That is the same trade case folding already makes and it falls the same way —
/// over-folding costs throughput, under-folding costs an edit.
#[must_use]
pub fn Normalize_Path(path: &str) -> String
{
    let folded = nomos_model::Normalize_Path(path);

    // Applied last, and to the folded text rather than to the authored spelling, so that
    // `Docs\Records\OD-LEDGER-006-x.md` reaches the same identifier as
    // `docs/records/od-ledger-006-x.md`. A rule that read the identifier off what the
    // author typed would have reintroduced, one level up, the hole the folding closes.
    if let Some(identifier) = Record_Identifier_Form(&folded)
    {
        return identifier;
    }

    return folded;
}

/// The directory this repository authors its decision records in, normalized.
///
/// Named here rather than passed in, because the rule below is about this directory and no
/// other — see [`Normalize_Path`] for why it has to be.
const RECORD_DIRECTORY: &str = "docs/records";

/// A record filename reduced to the identifier it carries, or [`None`] if it carries none.
///
/// Takes an already-folded path, because the identifier is read off the text and reading
/// it off two spellings would produce two identifiers.
///
/// # The grammar
///
/// Every record in this repository is named `<IDENTIFIER>-<slug>.md`, and every identifier
/// is one or more hyphen-separated words followed by an ordinal: `d-129`, `od-ledger-006`,
/// `arc-specdb-001`. The ordinal is what *ends* an identifier, so the first all-digit
/// component is its last component and everything after it is prose. Reading the ordinal
/// as the first digit group rather than the last is what keeps
/// `d-130-no-xvpe-dependency-before-phase-5.md` from being read as ending at the `5`.
///
/// Two things are deliberately not accepted. A name that does not *begin* with a word is
/// not an identifier at all — `2026-08-09-notes.md` reaches its first all-digit component
/// at `08` and would otherwise reduce to `docs/records/2026-08`, and a date is not an
/// allocation. And a component of digits must match a reservation's ordinal exactly rather
/// than prefix it, which falls out of comparing whole components: nothing makes
/// `od-ledger-001` the identifier of `od-ledger-0011-something.md`, because `0011` is one
/// component and it is not `001`.
pub(super) fn Record_Identifier_Form(folded: &str) -> Option<String>
{
    let name = folded.strip_prefix(RECORD_DIRECTORY)?.strip_prefix('/')?;

    // Records are flat. Anything deeper is some other thing that happens to live under the
    // directory, and guessing at its shape is how a rule scoped to one convention escapes
    // the scope it was given. `docs/records-archive/...` is refused by the same two steps
    // above: stripping the directory leaves `-archive/...`, which has no leading separator.
    if name.contains('/')
    {
        return None;
    }

    let stem = name.strip_suffix(".md")?;
    let identifier = Up_To_The_Ordinal(stem)?;

    return Some(format!("{RECORD_DIRECTORY}/{identifier}"));
}

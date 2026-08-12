//! Band 0 is described in one place, and the two files that restated it now route to it.
//!
//! Three files described `nomos-contracts` and they did not agree. `README.md`'s band table
//! said protocol truth, the crate root said the only authoritative statement of Nomos
//! *protocol* semantics, and `Cargo.toml`'s members comment dropped the word protocol and
//! claimed the only authoritative statement of Nomos semantics. The widest of the three sat
//! in a workspace manifest comment that nothing parses, so the disagreement `AGENTS.md`
//! resolves in favour of the mechanical authority had no mechanical authority to resolve it
//! against.
//!
//! `OD-CONTRACTS-001` decides the criterion and is the only place it is stated. This holds
//! the other half of that decision: the ownership sentence may appear in exactly one of the
//! three files, and every one of them must name the record.
//!
//! It is a phrase check and it is deliberately narrow. No test can tell whether a *new* type
//! belongs in band 0 — the criterion is for a person reviewing a change. What this can do is
//! stop the wide sentence being restated somewhere nothing reads back, which is how the
//! defect arrived in the first place.

use crate::common::Repository_Root;

/// The claim that was made three times and chosen once.
const OWNERSHIP: &str = "authoritative statement of Nomos";

/// The record that now carries it.
const RECORD: &str = "OD-CONTRACTS-001";

/// The one file entitled to make the claim: where an author adding a module is reading.
const HOME: &str = "crates/contracts/nomos-contracts/src/lib.rs";

/// The three files that describe band 0 at all.
const DESCRIBED: &[&str] = &["Cargo.toml", "README.md", HOME];

#[test]
fn Test_Band_Zero_Should_Be_Described_In_One_Place()
{
    let (claiming, silent_about_the_record) = Read_The_Three();

    assert_eq!(
        claiming,
        vec![HOME],
        "band 0 is claimed in {claiming:?} rather than in {HOME} alone. `{OWNERSHIP}` belongs \
         there and nowhere else: a second copy is a wider rule than the one that was decided, \
         sitting where no test reads it back. {RECORD} records why."
    );
    assert!(
        silent_about_the_record.is_empty(),
        "{silent_about_the_record:?} describe band 0 without naming {RECORD}. A file that \
         describes the band and does not route to the criterion is the restatement this \
         check exists to stop."
    );
}

/// Which of the three claim the ownership sentence, and which do not name the record.
///
/// Both answers come from one read of each file, because they are two readings of the same
/// text rather than two questions asked separately.
fn Read_The_Three() -> (Vec<&'static str>, Vec<&'static str>)
{
    let root = Repository_Root();
    let mut claiming = Vec::new();
    let mut silent = Vec::new();
    for relative in DESCRIBED
    {
        let path = root.join(relative);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        if text.contains(OWNERSHIP)
        {
            claiming.push(*relative);
        }
        if !text.contains(RECORD)
        {
            silent.push(*relative);
        }
    }

    return (claiming, silent);
}

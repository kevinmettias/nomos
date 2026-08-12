use crate::{GATES, VARIABLES};
use nomos_contract_tests::{Corpus_Gates, CORPUS_VARIABLES};
use std::collections::{BTreeMap, BTreeSet};

/// The gated count is the one figure the source can settle, so it does.
///
/// Everything else in the table was already derived and compared: the file set, the
/// variables, the test counts, the headline. `gated` was not. It was asserted to be no
/// larger than its file's test count, which `gated: 1` satisfies in every row — and it is
/// the column [`GATED_TOTAL`] sums, OD-GATE-001 cites and the gate workflow prints. The
/// most-load-bearing number here was the least checked.
///
/// The derivation resolves a test to the corpora it reaches through the helpers it calls.
/// It has to: almost no gated test names a variable itself, so counting the tests in a file
/// that mention one would find nearly none of them.
///
/// This does not replace the declaration. A derived count would let a fifteenth gated
/// assertion join the headline without anybody deciding it should, which is the silence
/// this file exists to break — one level further in.
#[test]
fn Test_Every_Declared_Gate_Count_Should_Be_The_One_In_The_Source()
{
    let derived = Gated_Tests_Per_File();

    assert!(
        !derived.is_empty(),
        "the scanner found no corpus-gated test anywhere in the workspace.\n\
         Every comparison below would then pass over an empty set, reporting that the \
         table is correct because nothing contradicted it — which is the shape of defect \
         this whole file is about."
    );

    let wrong = Rows_Disagreeing_With(&derived);

    assert!(
        wrong.is_empty(),
        "the table and the source disagree about what is gated: {wrong:#?}.\n\
         Set GATES to what the source now holds, deliberately. A test that inherits its \
         file's silence should be a decision somebody made rather than a consequence of \
         where it was written."
    );
}

/// Both directions in one list: a row the source contradicts, and a file the source gates
/// that no row names.
pub(crate) fn Rows_Disagreeing_With(derived: &BTreeMap<String, usize>) -> Vec<String>
{
    let declared: BTreeSet<&str> = GATES.iter().map(|gate| return gate.path).collect();
    let mut wrong = Vec::new();
    for gate in GATES
    {
        let found = derived.get(gate.path).copied().unwrap_or(0);
        if found != gate.gated
        {
            wrong.push(format!("{}: declares {} gated, source has {found}", gate.path, gate.gated));
        }
    }
    for (path, found) in derived
    {
        if !declared.contains(path.as_str())
        {
            wrong.push(format!("{path}: not in GATES, source has {found} gated"));
        }
    }

    return wrong;
}

/// The two spellings of the same three variables must not drift apart.
///
/// This file assembles the names with `concat!` so that it does not contain the strings it
/// searches for; the scanner spells them out, and excludes this crate from its own walk.
/// Two defences against the same problem, and therefore two lists — so the fact that they
/// are one fact is worth asserting. If they drift, one of them quietly stops seeing a
/// corpus and reports a smaller hole.
#[test]
fn Test_The_Scanner_And_This_Table_Should_Name_The_Same_Variables()
{
    let here: BTreeSet<&str> = VARIABLES.iter().copied().collect();
    let there: BTreeSet<&str> = CORPUS_VARIABLES.iter().copied().collect();

    assert_eq!(
        here, there,
        "this table and the scanner name different sets of corpus variables. One of them \
         has stopped looking for a corpus the other still counts, and whichever it is now \
         reports a hole smaller than the one that exists"
    );
}

/// How many gated tests the source holds, per file, repo-relative with forward slashes.
pub(crate) fn Gated_Tests_Per_File() -> BTreeMap<String, usize>
{
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();

    for gate in Corpus_Gates()
    {
        let entry = counts.entry(gate.file).or_default();
        *entry = entry.saturating_add(1);
    }

    return counts;
}

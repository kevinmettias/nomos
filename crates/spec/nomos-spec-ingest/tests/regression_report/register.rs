//! Whether the checked-in register is coherent on its own terms.
//!
//! No archive is read here. These are the questions that can be asked of the register alone
//! — every clause answered once, every family carrying one fate entry, every quoted figure
//! present in the counts register — and they are the half of this suite that runs on a
//! machine with no corpus. A register that contradicted itself would make the archive half
//! agree with the wrong numbers.

use crate::rows::{Count, Counts, Entry, Register, Restored_Family_For_Label};

const PLAN_HEADLINE: &[&str] = &[
    "282 table_row",
    "6 code_block",
    "8 roadmap_milestone",
    "8 scenario",
    "54 service",
    "N glossary_term",
    "all appendix-D schema",
    "all appendix-H section",
    "132 sections replaced by filler",
    "64 records (records/ + spec-governance/records/)",
];

#[test]
fn Test_Every_Headline_Clause_Should_Be_Answered_Once()
{
    let entries = Register();

    for clause in PLAN_HEADLINE
    {
        let answering: Vec<&str> = entries
            .iter()
            .filter(|entry| return entry.clause == *clause)
            .map(|entry| return entry.id.as_str())
            .collect();
        assert_eq!(answering.len(), 1, "{clause} is answered by {answering:?}");
    }
}

#[test]
fn Test_Every_Restored_Family_Should_Carry_A_Fate()
{
    let entries = Register();
    let named: Vec<&str> = entries.iter().filter_map(|entry| return entry.family.as_deref()).collect();

    Assert_Every_Restored_Family_Is_Named_Exactly_Once(&named);
    Assert_Every_Named_Family_Resolves(&named);
    Assert_Fates_Are_Stated_Only_By_A_Family_Entry(&entries);
}

/// Every restored family the ingest knows about is carried by exactly one entry.
fn Assert_Every_Restored_Family_Is_Named_Exactly_Once(named: &[&str])
{
    use nomos_spec_ingest::Restored;

    for family in Restored::All()
    {
        let carrying = named.iter().filter(|label| return **label == family.Label()).count();
        assert_eq!(carrying, 1, "{} carries {carrying} fate entries", family.Label());
    }
}

/// Every family label the register names is one the ingest still recognizes.
fn Assert_Every_Named_Family_Resolves(named: &[&str])
{
    for label in named
    {
        Restored_Family_For_Label(label);
    }
}

/// A family entry states fates, and only a family entry does.
fn Assert_Fates_Are_Stated_Only_By_A_Family_Entry(entries: &[Entry])
{
    for entry in entries
    {
        assert_eq!(
            entry.family.is_some(),
            entry.fates.is_some(),
            "{}: a family entry states fates and only a family entry does",
            entry.id
        );
    }
}

#[test]
fn Test_A_Family_Should_Sum_To_The_Size_The_Counts_Register_Measured()
{
    let counts = Counts();

    for entry in Register()
    {
        let Some(fates) = &entry.fates
        else
        {
            continue;
        };
        let measured = Measured_Size(&entry, &counts);

        assert_eq!(
            fates.Total(),
            measured,
            "{}: the fates cover {} members and the register measured {measured}",
            entry.id,
            fates.Total()
        );
    }
}

#[test]
fn Test_Every_Entry_Should_Say_What_The_Clause_Resolved_To()
{
    use std::collections::BTreeSet;

    let mut seen: BTreeSet<&str> = BTreeSet::new();

    for entry in &Register()
    {
        assert!(seen.insert(entry.id.as_str()), "{} is registered twice", entry.id);
        assert!(!entry.resolution.trim().is_empty(), "{}: resolves to nothing", entry.id);
        assert!(!entry.clause.trim().is_empty(), "{}: answers no clause", entry.id);
        assert!(
            ["disappeared", "changed in place", "appeared", "context"].contains(&entry.row.as_str()),
            "{}: {} is not a row of the headline",
            entry.id,
            entry.row
        );
    }
}

#[test]
fn Test_Every_Quoted_Count_Should_Exist_In_The_Counts_Register()
{
    let counts = Counts();

    for entry in Register()
    {
        let Some(quoted) = entry.quoted
        else
        {
            continue;
        };
        assert!(
            counts.iter().any(|count| return count.id == quoted),
            "{}: quotes {quoted}, which is not in the counts register",
            entry.id
        );
    }
}

/// The size the counts register measured for the family an entry quotes.
pub(crate) fn Measured_Size(entry: &Entry, counts: &[Count]) -> u32
{
    let quoted = entry
        .quoted
        .as_deref()
        // Only entries that state fates reach here, and fates are four numbers that have to
        // add up to a size somebody measured. An entry stating them without naming a counts
        // entry has no total to be checked against, so its four numbers answer to nothing.
        .unwrap_or_else(|| panic!("{}: states fates and quotes no measured size", entry.id));

    return counts
        .iter()
        .find(|count| return count.id == quoted)
        // The quoted id is the link between the two checked-in registers.
        // `Test_Every_Quoted_Count_Should_Exist_In_The_Counts_Register` asserts the link
        // holds; reaching this from a caller means the two registers were edited apart, and
        // a default size would let the fates be compared against a total nobody measured.
        .unwrap_or_else(|| panic!("{}: {quoted} is not in the counts register", entry.id))
        .measured;
}

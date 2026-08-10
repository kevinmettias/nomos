//! Which corpus requirements this build has been assessed against, and whether the
//! assessments still resolve.
//!
//! `OD-TRACE-001` measured the hole this closes. The v14 corpus carries 363 requirements at
//! `authority: canonical-normative-record`, fourteen are named anywhere in this repository,
//! and — the finding that made it a defect rather than a fact — **met and unmet have the
//! same shape from outside.** An identifier that appears nowhere, beside code that may or
//! may not honour it, reads identically whether somebody checked or nobody did. Six hand
//! comparisons cost a full manual audit each and three came back met, which is the worst
//! ratio a hand check can have: most of the work bought no change.
//!
//! An assessment is therefore **a declared entry committed to this repository**, compared
//! against the workspace by this file, and never derived from the corpus at check time.
//! `OD-GATE-001` is what forces that: a test that cannot find its corpus returns early and
//! prints `ok`, CI has none of the three corpora, and a traceability surface computed from
//! the corpus would be green on every pull request by being unable to look — green about
//! the *entire* corpus, which is a far larger lie than the sixty-eight tests that hole
//! already covers. `OD-TRACE-002` records why no corpus comparison is added here at all,
//! which is a stronger statement than "not yet".
//!
//! # What the guard can and cannot see
//!
//! It sees that a named site has vanished. It cannot see that the code at that site drifted
//! out of satisfying the requirement while the path stayed valid. Semantic drift is meaning
//! and this workspace has no type for it, so a stale `Met` is wrong in one requirement —
//! against a status quo that is unreadable in all 363. `OD-TRACE-001` takes that trade
//! explicitly and this file does not re-argue it.
//!
//! # Unassessed is a state, not a gap
//!
//! Absence of an entry means nobody looked, and that is the honest description of most of
//! the corpus today. So the registry is **a floor rather than a count**, the shape
//! `OD-SPEC-007` already chose for governing records: entries may be added freely, and
//! [`FEWEST_ASSESSMENTS`] is what a deletion has to walk past.

use nomos_contract_tests::Workspace;
use std::collections::BTreeSet;
use std::path::Path;

/// Where the committed entries live, relative to the workspace root.
const REGISTRY: &str = "tests/contract/requirements";

/// The extension one entry carries.
const EXTENSION: &str = "assessment";

/// How few assessments this repository may hold.
///
/// A floor, not a count, and `OD-SPEC-007` settled the trade for the identical case one
/// directory over. Adding an assessment costs no edit here, which is the whole point —
/// two people assessing two requirements must not collide on a third file. Removing one
/// costs lowering this, which is the deliberate step that keeps an entry from being quietly
/// deleted to make a divergence disappear.
///
/// Its guarantee is exact only while the count sits on it. Once the registry has grown
/// above, a deletion inside the slack is caught by nothing here — the same slack
/// `OD-SPEC-007` accepted, for the same reason: an upper bound would reintroduce the shared
/// edit on an unpredictable schedule.
const FEWEST_ASSESSMENTS: usize = 4;

/// What an assessment says about a requirement.
///
/// Four verdicts in `OD-TRACE-001` and three of them here. The fourth, `Unassessed`, is
/// held by the *absence* of an entry and is refused as a written word by [`Parse`] — a file
/// saying `Unassessed` would be somebody looking and recording that they had not, which is
/// the one thing this registry must not be able to express.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verdict
{
    /// The site satisfies the requirement, and the entry names where.
    Met,
    /// The site is derived from the requirement and departs from it deliberately. The
    /// entry names the governing record carrying the reason; a divergence with no record
    /// is not a verdict, it is the state `OD-TRACE-001` exists to end.
    Diverges,
    /// The requirement is read as not reaching this build, with a record saying why a
    /// corpus requirement does not bind the thing built to enforce it.
    NotBinding,
}

impl Verdict
{
    /// The word an entry is written with.
    const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Met => "Met",
            Self::Diverges => "Diverges",
            Self::NotBinding => "NotBinding",
        };
    }

    /// Whether a verdict of this kind is owed a governing record.
    ///
    /// Both non-`Met` verdicts are. Departing from a normative requirement and declaring
    /// one out of scope are the same act from the corpus's side — somebody deciding this
    /// build will not do what the requirement says — and the reason is what makes either
    /// reviewable.
    const fn Owes_A_Record(self) -> bool
    {
        return matches!(self, Self::Diverges | Self::NotBinding);
    }
}

/// A place in the workspace a verdict is about.
///
/// A path alone would nearly never fire: files are renamed far less often than the symbols
/// inside them. The symbol is what makes the entry decay visibly when the thing it was
/// about is renamed out from under it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Site
{
    /// Repo-relative, forward slashes.
    path: String,
    /// Text that must occur in that file.
    symbol: String,
}

/// One committed assessment.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Assessment
{
    /// The corpus requirement identifier. The file stem, never a key inside the file.
    requirement: String,
    /// What the assessment says.
    verdict: Verdict,
    /// The governing record carrying the reasoning, when one is named.
    record: Option<String>,
    /// Every place the verdict is about. Never empty.
    sites: Vec<Site>,
}

// ---------------------------------------------------------------------------
// The three things OD-TRACE-001 said the guard asserts.
// ---------------------------------------------------------------------------

/// Every entry names a site that exists in the workspace.
#[test]
fn Test_Every_Assessment_Should_Name_A_Site_That_Exists()
{
    let root = Workspace::Workspace_Root();
    let assessments = Committed(&root);

    assert!(
        !assessments.is_empty(),
        "no assessment was read from {REGISTRY}, so every comparison in this file would \
         pass over an empty set — reporting that the registry is sound because nothing \
         contradicted it, which is the shape of defect OD-TRACE-001 is about"
    );

    let missing = Unresolved_Sites(&root, &assessments);

    assert!(
        missing.is_empty(),
        "these assessments name a site that is not there: {missing:#?}.\n\
         Either the code moved, in which case update the entry, or the thing the verdict \
         was about is gone, in which case the verdict is about nothing and the entry has \
         to be re-authored against what replaced it."
    );
}

/// Every entry that names a record names one that exists and is registered.
///
/// Wider than `OD-TRACE-001` requires, which is only that a `Diverges` entry does. A `Met`
/// entry may omit a record; naming one that does not resolve is a different thing, and a
/// dangling citation is worth catching wherever it appears. The narrower obligation — that
/// a divergence names a record *at all* — is
/// [`Test_Every_Divergence_Should_Name_A_Governing_Record`].
///
/// "Registered" is `OD-SPEC-007`'s sense: a document under `docs/records` is not governing
/// until a file under `crates/spec/nomos-spec-store/records/` says so. An entry citing an
/// unregistered document would be citing prose.
#[test]
fn Test_Every_Named_Record_Should_Exist_And_Be_Registered()
{
    let root = Workspace::Workspace_Root();
    let assessments = Committed(&root);

    let cited = assessments
        .iter()
        .filter(|assessment| return assessment.record.is_some())
        .count();
    assert!(
        cited > 0,
        "no committed assessment names a record, so this comparison read nothing"
    );

    let unresolved = Unresolved_Records(&root, &assessments);

    assert!(
        unresolved.is_empty(),
        "these assessments name a record that does not resolve: {unresolved:#?}.\n\
         A record is registered by its own file under crates/spec/nomos-spec-store/records/ \
         — OD-SPEC-007 — and writing the document alone does not make it governing."
    );
}

/// A divergence with no record is not a verdict.
///
/// **This assertion is vacuous today and that is stated rather than hidden.** Every
/// committed entry is `Met`, so the loop below runs over an empty set. The two divergences
/// `OD-TRACE-001` hand-audited — `WORK-LEDGER-005` dropping `StaleProbeArtifact` and
/// `WORK-LEDGER-001` dropping `priority` — cannot be entered yet precisely because neither
/// has a record saying why, which is `P10-LEDGER-CORPUS`'s subject and not this file's.
///
/// What keeps the vacuity from being a hole is
/// [`Test_A_Divergence_With_No_Record_Should_Be_Refused`], which runs this same function
/// over a synthetic entry. It calls `Divergences_With_No_Record`, not a copy of it, so the
/// control exercises the guard rather than something written beside it.
#[test]
fn Test_Every_Divergence_Should_Name_A_Governing_Record()
{
    let root = Workspace::Workspace_Root();
    let assessments = Committed(&root);
    let unreasoned = Divergences_With_No_Record(&assessments);

    assert!(
        unreasoned.is_empty(),
        "these assessments depart from a requirement and say nothing about why: \
         {unreasoned:#?}.\n\
         A divergence with no record is the state OD-TRACE-001 exists to end. Write the \
         record, register it, and name it here."
    );
}

/// The assessed set only grows.
#[test]
fn Test_The_Assessed_Set_Should_Not_Shrink()
{
    let root = Workspace::Workspace_Root();
    let assessments = Committed(&root);

    assert!(
        assessments.len() >= FEWEST_ASSESSMENTS,
        "{} requirements are assessed and FEWEST_ASSESSMENTS says at least \
         {FEWEST_ASSESSMENTS}.\n\
         An entry cannot be quietly deleted to make a divergence disappear. If a removal \
         is deliberate, lower the floor in the same commit and say why in the message.",
        assessments.len()
    );

    let distinct: BTreeSet<&str> = assessments
        .iter()
        .map(|assessment| return assessment.requirement.as_str())
        .collect();
    assert_eq!(
        distinct.len(),
        assessments.len(),
        "two entries assess one requirement, so the floor counts a requirement twice"
    );
}

/// `CHK-003`'s verdict stops living in prose.
///
/// The entry this whole item exists to make readable. `OD-CONTRACTS-002` argued that
/// `CHK-003` binds this build, decided the seventh reporting category, and then had to say
/// **Met** in its own prose because the registry had no home yet — its territory was two
/// source files, two surface snapshots and one record, and territory cannot be widened
/// mid-claim. This is the entry that replaces that sentence.
#[test]
fn Test_CHK_003_Should_Be_Assessed_Met_By_The_Record_That_Decided_It()
{
    let root = Workspace::Workspace_Root();
    let assessments = Committed(&root);

    let entry = assessments
        .iter()
        .find(|assessment| return assessment.requirement == "CHK-003")
        .expect("CHK-003 must be assessed; OD-CONTRACTS-002 says Met in prose until it is");

    assert_eq!(entry.verdict, Verdict::Met);
    assert_eq!(entry.record.as_deref(), Some("OD-CONTRACTS-002"));

    for file in [
        "crates/contracts/nomos-contracts/src/applicability.rs",
        "crates/contracts/nomos-contracts/src/evidence.rs",
    ]
    {
        assert!(
            entry.sites.iter().any(|site| return site.path == file),
            "CHK-003's entry must name a site in {file}; the seventh reporting category is \
             a variant in one and the evidence class it must not be confused with is in \
             the other"
        );
    }
}

// ---------------------------------------------------------------------------
// Controls. Each runs the real predicate over a constructed input.
// ---------------------------------------------------------------------------

/// The negative control for [`Test_Every_Assessment_Should_Name_A_Site_That_Exists`].
///
/// A guard that reported every site as present would pass that test and catch nothing.
#[test]
fn Test_An_Entry_Naming_A_Vanished_Site_Should_Be_Reported()
{
    let root = Workspace::Workspace_Root();

    let gone_file = Assessment {
        requirement: "CHK-003".to_owned(),
        verdict: Verdict::Met,
        record: None,
        sites: vec![Site {
            path: "crates/contracts/nomos-contracts/src/no_such_file.rs".to_owned(),
            symbol: "Applicability".to_owned(),
        }],
    };
    let gone_symbol = Assessment {
        requirement: "EVID-001".to_owned(),
        verdict: Verdict::Met,
        record: None,
        sites: vec![Site {
            path: "crates/contracts/nomos-contracts/src/evidence.rs".to_owned(),
            symbol: "A_Name_This_Workspace_Does_Not_Use".to_owned(),
        }],
    };

    assert_eq!(Unresolved_Sites(&root, &[gone_file]).len(), 1);
    assert_eq!(
        Unresolved_Sites(&root, &[gone_symbol]).len(),
        1,
        "a renamed symbol in a file that still exists is the case a path-only check misses, \
         and it is the common one"
    );
}

/// The negative control for [`Test_Every_Named_Record_Should_Exist_And_Be_Registered`],
/// in both halves.
///
/// A document with no registration is `OD-SPEC-005`'s defect — six governing records were
/// written as files and never reached the store — so citing one has to be caught
/// separately from citing nothing at all.
#[test]
fn Test_A_Record_That_Does_Not_Resolve_Should_Be_Reported()
{
    let root = Workspace::Workspace_Root();

    let invented = Assessment {
        requirement: "CAP-002".to_owned(),
        verdict: Verdict::Met,
        record: Some("OD-NOTHING-999".to_owned()),
        sites: vec![Site {
            path: "crates/contracts/nomos-contracts/src/guarantee.rs".to_owned(),
            symbol: "FactVariant".to_owned(),
        }],
    };

    assert_eq!(Unresolved_Records(&root, &[invented]).len(), 1);

    let real = Assessment {
        requirement: "CAP-002".to_owned(),
        verdict: Verdict::Met,
        record: Some("OD-TRACE-001".to_owned()),
        sites: Vec::new(),
    };
    assert!(
        Unresolved_Records(&root, &[real]).is_empty(),
        "a registered record must resolve, or the control above proves only that the \
         function always reports something"
    );
}

/// The negative control for [`Test_Every_Divergence_Should_Name_A_Governing_Record`],
/// which is vacuous over the committed set.
#[test]
fn Test_A_Divergence_With_No_Record_Should_Be_Refused()
{
    let unreasoned = Assessment {
        requirement: "WORK-LEDGER-005".to_owned(),
        verdict: Verdict::Diverges,
        record: None,
        sites: vec![Site {
            path: "crates/substrate/nomos-ledger/src/item.rs".to_owned(),
            symbol: "Blocker".to_owned(),
        }],
    };

    assert_eq!(
        Divergences_With_No_Record(std::slice::from_ref(&unreasoned)).len(),
        1
    );

    let reasoned = Assessment {
        record: Some("OD-TRACE-001".to_owned()),
        ..unreasoned.clone()
    };
    assert!(Divergences_With_No_Record(&[reasoned]).is_empty());

    let met = Assessment {
        verdict: Verdict::Met,
        ..unreasoned
    };
    assert!(
        Divergences_With_No_Record(&[met]).is_empty(),
        "a Met entry owes no record, and reporting one would make the obligation \
         unstatable rather than merely strict"
    );
}

// ---------------------------------------------------------------------------
// The reader refuses rather than skips.
// ---------------------------------------------------------------------------

/// Every refusal, asserted as an `Err` rather than as a short `Ok`.
///
/// A skipped entry is an assessment that quietly stops being one — `OD-SPEC-005`'s defect
/// in this registry's clothes — so the reader has no lenient path at all.
#[test]
fn Test_The_Reader_Should_Refuse_Every_Malformed_Entry()
{
    let sound = "verdict: Met\nsite: README.md#Nomos\n";
    assert!(
        Parse("CHK-003", sound).is_ok(),
        "the well-formed case must parse, or every refusal below proves only that the \
         reader refuses everything"
    );

    for (stem, text, because) in [
        ("CHK-003", "site: README.md#Nomos\n", "no verdict"),
        ("CHK-003", "verdict: Met\n", "no site"),
        (
            "CHK-003",
            "verdict: Unassessed\nsite: README.md#Nomos\n",
            "Unassessed is held by the absence of an entry and must not be writable",
        ),
        (
            "CHK-003",
            "verdict: Satisfied\nsite: README.md#Nomos\n",
            "a verdict outside the three",
        ),
        (
            "CHK-003",
            "verdict: Met\nverdict: Diverges\nsite: README.md#Nomos\n",
            "two verdicts",
        ),
        (
            "CHK-003",
            "verdict: Met\nrecord: OD-TRACE-001\nrecord: OD-SPEC-007\nsite: README.md#Nomos\n",
            "two records",
        ),
        (
            "CHK-003",
            "verdict: Met\nsite: README.md\n",
            "a site with no symbol",
        ),
        (
            "CHK-003",
            "verdict: Met\nsite: #Nomos\n",
            "a site with no path",
        ),
        (
            "CHK-003",
            "verdict: Met\nsite: /absolute.rs#Nomos\n",
            "a site that is not repo-relative",
        ),
        (
            "CHK-003",
            "verdict: Met\nsite: ../outside.rs#Nomos\n",
            "a site reaching outside the workspace",
        ),
        (
            "CHK-003",
            "verdict: Met\nid: CHK-003\nsite: README.md#Nomos\n",
            "an unknown key — an id: line would be a second place for the identity to be \
             wrong, which is the rule OD-SPEC-007 set for a record registration",
        ),
        (
            "CHK-003",
            "verdict Met\nsite: README.md#Nomos\n",
            "a line that is not a key",
        ),
        (
            "chk-3",
            "verdict: Met\nsite: README.md#Nomos\n",
            "a stem that is not a requirement identifier",
        ),
        (
            "CHK-003",
            "verdict: Diverges\nsite: README.md#Nomos\n",
            "a divergence with no record, refused at read time as well as compared",
        ),
    ]
    {
        assert!(
            Parse(stem, text).is_err(),
            "the reader accepted an entry with {because}"
        );
    }
}

/// The identifier is the stem, and nothing inside the file may restate it.
#[test]
fn Test_A_Requirement_Identifier_Should_Be_A_Family_And_A_Number()
{
    for accepted in ["CHK-003", "EVID-001", "CAP-002", "WORK-LEDGER-005", "US-CHK-001"]
    {
        assert!(Is_Requirement_Id(accepted), "{accepted} is a requirement id");
    }

    for refused in ["CHK-3", "CHK-0003", "chk-003", "003", "CHK-", "-003", "CHK-00A"]
    {
        assert!(!Is_Requirement_Id(refused), "{refused} is not a requirement id");
    }
}

/// The reader enumerates the directory it is given, not one it knows about.
///
/// `OD-SPEC-007` made the same check structural for record registrations, for the reason it
/// states: a reader that reaches for a fixed path cannot be handed a constructed input, and
/// a guard whose input cannot be constructed is a guard nobody can write a control for.
#[test]
fn Test_The_Reader_Should_Enumerate_The_Directory_It_Is_Given()
{
    let root = Workspace::Workspace_Root();
    // Non-empty rather than at the floor. How many entries there are is
    // `Test_The_Assessed_Set_Should_Not_Shrink`'s fact, and asserting it here as well would
    // give one deletion two red tests — measured, when removing a single entry reddened
    // both. This one owes only that the reader read the directory it was handed.
    let real = Entries(&root.join(REGISTRY)).expect("the committed registry must read");
    assert!(!real.is_empty(), "the committed registry read as empty");

    let empty = Entries(&root.join("tests/contract/surface"))
        .expect("a directory holding no entry is empty rather than an error");
    assert!(
        empty.is_empty(),
        "the reader answered about the registry while being handed another directory, so \
         no control can ever construct an input for it"
    );
}

// ---------------------------------------------------------------------------
// The predicates. Each is called by an assertion and by a control.
// ---------------------------------------------------------------------------

/// Every site that is not where its entry says it is.
fn Unresolved_Sites(root: &Path, assessments: &[Assessment]) -> Vec<String>
{
    let mut missing = Vec::new();

    for assessment in assessments
    {
        for site in &assessment.sites
        {
            let path = root.join(&site.path);
            let Ok(text) = std::fs::read_to_string(&path)
            else
            {
                missing.push(format!(
                    "{}: {} is not a file in this workspace",
                    assessment.requirement, site.path
                ));
                continue;
            };

            if !text.contains(&site.symbol)
            {
                missing.push(format!(
                    "{}: {} no longer occurs in {}",
                    assessment.requirement, site.symbol, site.path
                ));
            }
        }
    }

    return missing;
}

/// Every named record that is not a registered governing record.
fn Unresolved_Records(root: &Path, assessments: &[Assessment]) -> Vec<String>
{
    let mut unresolved = Vec::new();

    for assessment in assessments
    {
        let Some(record) = assessment.record.as_deref()
        else
        {
            continue;
        };

        if !Registration_Exists(root, record)
        {
            unresolved.push(format!(
                "{}: {record} has no registration under \
                 crates/spec/nomos-spec-store/records/",
                assessment.requirement
            ));
            continue;
        }

        if !Document_Exists(root, record)
        {
            unresolved.push(format!(
                "{}: {record} is registered and its document is not under docs/records/",
                assessment.requirement
            ));
        }
    }

    return unresolved;
}

/// Every entry that departs from a requirement without saying why.
fn Divergences_With_No_Record(assessments: &[Assessment]) -> Vec<String>
{
    return assessments
        .iter()
        .filter(|assessment| {
            return assessment.verdict.Owes_A_Record() && assessment.record.is_none();
        })
        .map(|assessment| {
            return format!(
                "{}: {} with no governing record",
                assessment.requirement,
                assessment.verdict.Label()
            );
        })
        .collect();
}

/// Whether a record identifier has a registration file.
fn Registration_Exists(root: &Path, record: &str) -> bool
{
    return root
        .join("crates/spec/nomos-spec-store/records")
        .join(format!("{record}.record"))
        .is_file();
}

/// Whether a record identifier has a document under `docs/records`.
///
/// By stem prefix, because the slug is not derivable from the identifier — the same reason
/// `OD-SPEC-007` puts the path inside the registration rather than computing it.
fn Document_Exists(root: &Path, record: &str) -> bool
{
    let prefix = format!("{record}-");
    let Ok(entries) = std::fs::read_dir(root.join("docs/records"))
    else
    {
        return false;
    };

    return entries.flatten().any(|entry| {
        let path = entry.path();
        if path
            .extension()
            .is_none_or(|extension| return extension != "md")
        {
            return false;
        }

        return path
            .file_name()
            .and_then(|name| return name.to_str())
            .is_some_and(|name| return name.starts_with(&prefix));
    });
}

// ---------------------------------------------------------------------------
// Reading the registry.
// ---------------------------------------------------------------------------

/// Every committed assessment, or a panic naming the entry that would not read.
fn Committed(root: &Path) -> Vec<Assessment>
{
    return Entries(&root.join(REGISTRY))
        .unwrap_or_else(|refusal| panic!("the committed registry must read: {refusal}"));
}

/// Every assessment in a directory, sorted by requirement.
///
/// Takes the directory rather than finding it, so a control can hand it one.
fn Entries(directory: &Path) -> Result<Vec<Assessment>, String>
{
    let mut found = Vec::new();

    let listing = std::fs::read_dir(directory)
        .map_err(|error| return format!("{} cannot be read: {error}", directory.display()))?;

    for entry in listing.flatten()
    {
        let path = entry.path();
        if !path
            .extension()
            .is_some_and(|extension| return extension == EXTENSION)
        {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|stem| return stem.to_str())
            .ok_or_else(|| return format!("{} has no readable stem", path.display()))?;

        let text = std::fs::read_to_string(&path)
            .map_err(|error| return format!("{} cannot be read: {error}", path.display()))?;

        found.push(Parse(stem, &text).map_err(|refusal| {
            return format!("{}: {refusal}", path.display());
        })?);
    }

    found.sort_by(|left, right| return left.requirement.cmp(&right.requirement));
    return Ok(found);
}

/// One entry, or the reason it is not one.
///
/// Refuses rather than skips on every malformed shape. A lenient reader here would turn a
/// typo into an assessment that silently stopped being counted, and the floor would then
/// measure a set nobody chose.
fn Parse(stem: &str, text: &str) -> Result<Assessment, String>
{
    if !Is_Requirement_Id(stem)
    {
        return Err(format!(
            "{stem} is not a requirement identifier; a file here is named for the \
             requirement it assesses"
        ));
    }

    let mut verdict: Option<Verdict> = None;
    let mut record: Option<String> = None;
    let mut sites: Vec<Site> = Vec::new();

    for line in text.lines()
    {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#')
        {
            continue;
        }

        let Some((key, value)) = trimmed.split_once(':')
        else
        {
            return Err(format!("`{trimmed}` is not a `key: value` line"));
        };
        let value = value.trim();

        match key.trim()
        {
            "verdict" =>
            {
                if verdict.is_some()
                {
                    return Err("two verdict lines; an entry says one thing".to_owned());
                }
                verdict = Some(Read_Verdict(value)?);
            }
            "record" =>
            {
                if record.is_some()
                {
                    return Err(
                        "two record lines; one reason, in one record, or the entry does \
                         not say which"
                            .to_owned(),
                    );
                }
                if value.is_empty()
                {
                    return Err("an empty record line".to_owned());
                }
                record = Some(value.to_owned());
            }
            "site" => sites.push(Read_Site(value)?),
            other => return Err(format!("unknown key `{other}`")),
        }
    }

    let verdict = verdict.ok_or_else(|| {
        return "no verdict; an entry with none is not an assessment".to_owned();
    })?;

    if sites.is_empty()
    {
        return Err(
            "no site; a verdict that names no place in the workspace cannot go stale and \
             cannot be checked"
                .to_owned(),
        );
    }

    if verdict.Owes_A_Record() && record.is_none()
    {
        return Err(format!(
            "{} with no record; OD-TRACE-001 makes that not a verdict but the state it \
             exists to end",
            verdict.Label()
        ));
    }

    return Ok(Assessment {
        requirement: stem.to_owned(),
        verdict,
        record,
        sites,
    });
}

/// One of the three writable verdicts.
fn Read_Verdict(value: &str) -> Result<Verdict, String>
{
    for verdict in [Verdict::Met, Verdict::Diverges, Verdict::NotBinding]
    {
        if value == verdict.Label()
        {
            return Ok(verdict);
        }
    }

    if value == "Unassessed"
    {
        return Err(
            "Unassessed is held by the absence of an entry, so writing one would record \
             that somebody looked and did not look"
                .to_owned(),
        );
    }

    return Err(format!(
        "`{value}` is not a verdict; OD-TRACE-001 names Met, Diverges and NotBinding, and \
         holds the fourth by absence"
    ));
}

/// A `path#symbol` site.
fn Read_Site(value: &str) -> Result<Site, String>
{
    let Some((path, symbol)) = value.split_once('#')
    else
    {
        return Err(format!(
            "`{value}` is not a site; a site is `path#symbol`, and a path on its own \
             survives every rename that matters"
        ));
    };

    let path = path.trim();
    let symbol = symbol.trim();

    if path.is_empty() || symbol.is_empty()
    {
        return Err(format!("`{value}` has an empty path or symbol"));
    }

    if path.starts_with('/') || path.starts_with('\\') || path.contains("..")
    {
        return Err(format!(
            "`{path}` is not repo-relative; a site outside this workspace is not a site \
             this guard can see vanish"
        ));
    }

    return Ok(Site {
        path: path.to_owned(),
        symbol: symbol.to_owned(),
    });
}

/// Whether a string is a corpus requirement identifier: a family, then three digits.
///
/// The shape every family in the corpus uses — `CHK-003`, `EVID-001`, `WORK-LEDGER-005`.
/// Checked because the stem *is* the identity, so a mistyped one would silently create a
/// requirement the corpus does not have and count it toward the floor.
fn Is_Requirement_Id(stem: &str) -> bool
{
    let Some((family, number)) = stem.rsplit_once('-')
    else
    {
        return false;
    };

    if number.len() != 3 || !number.bytes().all(|byte| return byte.is_ascii_digit())
    {
        return false;
    }

    if family.is_empty()
    {
        return false;
    }

    return family
        .split('-')
        .all(|segment| {
            return !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| return byte.is_ascii_uppercase() || byte.is_ascii_digit());
        });
}

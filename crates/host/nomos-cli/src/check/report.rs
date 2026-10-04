//! Turning what `nomos_check_orchestration` produced — or a walk decision made before it
//! was ever called — into text and an [`ExitCode`].
//!
//! `Coverage` and this rendering stayed here on purpose: `nomos_check_orchestration::
//! CheckOutcome::Judged` carries `findings`, `examined` and `claim` — the judgment itself —
//! and a coverage breakdown is a pure, re-derivable grouping of `findings` for a text
//! reader, not a second fact a caller could need without also wanting to print it.
//! `Examined` and `Claim` moved out because they are exactly that second kind: a judgment a
//! second adapter would otherwise have to re-derive rather than read off `CheckOutcome`.
//!
//! So did `Populations`, which this report renders after the claim and apart from it: the rules
//! that judged an empty population, whose zero findings are not a clean judgment.
//! `OD-ANALYSIS-012` version 2 decided both halves -- the reader is shown them, and the claim
//! does not move because of them.
//!
//! And `UndeclaredValues`, rendered after both: every value a rule read that the repository never
//! declared, with what the rule did with it. `OD-RULES-011` version 3 decided it the same way --
//! named in default output, never in a finding, and the claim does not read it.

use super::{ExitCode, Finding, Path, Write};
use nomos_capability::RegistryError;
use nomos_check_orchestration::{CheckOutcome, Claim, Examined, Populations, UndeclaredValues};
use nomos_contracts::Applicability;
use nomos_rules::{UndeclaredOutcome, UndeclaredValue};

/// How many subjects this run placed in each [`Applicability`] state.
///
/// One counter per variant, filled by a match with no wildcard arm — the same shape
/// [`Applicability`] itself is built with, so a twelfth state added there stops this file
/// compiling rather than landing silently in whichever bucket the match happened to fall
/// through to.
///
/// `OD-COMPLETENESS-004` is why this exists beside `Examined` rather than growing it by
/// one more integer. Two denominators can tell a clean tree from a broken walk; neither one
/// integer nor a third can tell which of eleven reasons a subject was not judged for, and
/// that is the distinction a reader needs to tell an unexamined subject from a clean one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(super) struct Coverage
{
    supported: usize,
    supported_with_fallback: usize,
    partially_supported: usize,
    not_applicable: usize,
    missing_capability: usize,
    provider_unavailable: usize,
    dependency_unavailable: usize,
    configuration_disabled: usize,
    unparseable: usize,
    analysis_failed: usize,
    agent_required: usize,
}

/// How many variants [`Applicability`] has -- and so how many buckets [`Coverage::All`]
/// reports, one per variant.
const APPLICABILITY_VARIANT_COUNT: usize = 11;

impl Coverage
{
    /// Buckets every finding's applicability, once each.
    fn Of(findings: &[Finding]) -> Self
    {
        let mut coverage = Self::default();

        for finding in findings
        {
            coverage.Record(finding.applicability);
        }

        return coverage;
    }

    /// The one place a bucket is chosen. No wildcard arm: an [`Applicability`] variant
    /// arriving without a home here fails the build at this match, beside the field it has
    /// to be added to — the same guard [`Applicability::Label`] already keeps.
    fn Record(&mut self, applicability: Applicability)
    {
        match applicability
        {
            Applicability::Supported => self.supported = self.supported.saturating_add(1),
            Applicability::SupportedWithFallback =>
            {
                self.supported_with_fallback = self.supported_with_fallback.saturating_add(1);
            }
            Applicability::PartiallySupported =>
            {
                self.partially_supported = self.partially_supported.saturating_add(1);
            }
            Applicability::NotApplicable => self.not_applicable = self.not_applicable.saturating_add(1),
            Applicability::MissingCapability =>
            {
                self.missing_capability = self.missing_capability.saturating_add(1);
            }
            Applicability::ProviderUnavailable =>
            {
                self.provider_unavailable = self.provider_unavailable.saturating_add(1);
            }
            Applicability::DependencyUnavailable =>
            {
                self.dependency_unavailable = self.dependency_unavailable.saturating_add(1);
            }
            Applicability::ConfigurationDisabled =>
            {
                self.configuration_disabled = self.configuration_disabled.saturating_add(1);
            }
            Applicability::Unparseable => self.unparseable = self.unparseable.saturating_add(1),
            Applicability::AnalysisFailed => self.analysis_failed = self.analysis_failed.saturating_add(1),
            Applicability::AgentRequired => self.agent_required = self.agent_required.saturating_add(1),
        }
    }

    /// The buckets this run actually put something in.
    ///
    /// Zero buckets are omitted from the rendering rather than printed at zero: eleven lines
    /// on every run, ten of them reading `: 0`, is the kind of output a reader learns to skip
    /// past, which is exactly how a silent omission stayed silent the first time.
    fn Nonzero(&self) -> Vec<(Applicability, usize)>
    {
        return self.All().into_iter().filter(|&(_, count)| return count > 0).collect();
    }

    /// All eleven buckets, labelled, in the order [`Applicability`] declares them.
    ///
    /// Grouped by variant rather than by capability or by language, because this build
    /// registers one capability and one language: either of those groupings would collapse
    /// to a single bucket today and hide exactly the distinction this type exists to show.
    /// Nothing here prevents composing a second grouping later — [`Finding::rule`] is
    /// already a capability's proxy — but a grouping that answers nothing at the current
    /// scale is not the one to ship first. `OD-COMPLETENESS-004` records the choice.
    fn All(&self) -> [(Applicability, usize); APPLICABILITY_VARIANT_COUNT]
    {
        return [
            (Applicability::Supported, self.supported),
            (Applicability::SupportedWithFallback, self.supported_with_fallback),
            (Applicability::PartiallySupported, self.partially_supported),
            (Applicability::NotApplicable, self.not_applicable),
            (Applicability::MissingCapability, self.missing_capability),
            (Applicability::ProviderUnavailable, self.provider_unavailable),
            (Applicability::DependencyUnavailable, self.dependency_unavailable),
            (Applicability::ConfigurationDisabled, self.configuration_disabled),
            (Applicability::Unparseable, self.unparseable),
            (Applicability::AnalysisFailed, self.analysis_failed),
            (Applicability::AgentRequired, self.agent_required),
        ];
    }
}

/// Turns what [`nomos_check_orchestration::Run`] produced — or a walk decision the
/// composition root made before ever calling it — into text and an [`ExitCode`].
pub(super) fn Render_Outcome(root: &Path, outcome: &CheckOutcome, stdout: &mut impl Write, stderr: &mut impl Write) -> ExitCode
{
    return match outcome
    {
        CheckOutcome::Unreadable => Render_Unreadable(root, stderr),
        CheckOutcome::Contradictory(error) => Render_Contradictory(error, stderr),
        CheckOutcome::NoSource => Render_No_Source(root, stderr),
        CheckOutcome::NoFacts { files } => Render_No_Facts(root, *files, stderr),
        CheckOutcome::Judged { findings, examined, claim, populations, undeclared, .. } =>
        {
            let code = Report_Findings(findings, *examined, *claim, stdout);
            Print_Empty_Populations(populations, stdout);
            Print_Undeclared_Values(undeclared, stdout);
            code
        }
    };
}

/// The root does not exist, is not a directory, or its walk could not be ingested.
fn Render_Unreadable(root: &Path, stderr: &mut impl Write) -> ExitCode
{
    let _ignored = writeln!(
        stderr,
        "cannot judge `{}`: not a directory, or its walk could not be ingested as a \
         workspace state",
        root.display()
    );

    return ExitCode::Unreadable;
}

/// This build's own capability registry is self-contradictory -- a defect in the
/// composition, not in the tree being checked.
fn Render_Contradictory(error: &RegistryError, stderr: &mut impl Write) -> ExitCode
{
    let _ignored = writeln!(
        stderr,
        "this build's own composition is contradictory, so no fact it produced would \
         have been offered by anybody: {error}"
    );

    return ExitCode::Unreadable;
}

/// The walk found no source under `root`.
fn Render_No_Source(root: &Path, stderr: &mut impl Write) -> ExitCode
{
    let _ignored = writeln!(
        stderr,
        "no Rust source found under `{}`, so nothing was judged.\n\
         A clean result here would mean only that the walk found nothing.",
        root.display()
    );

    return ExitCode::Vacuous;
}

/// Source was found but no syntax fact was materialized for any of it.
fn Render_No_Facts(root: &Path, files: usize, stderr: &mut impl Write) -> ExitCode
{
    let _ignored = writeln!(
        stderr,
        "{files} file(s) were read under `{}` and no syntax fact was materialized for \
         any of them, so no mirror claim could be resolved.\n\
         A clean result here would mean only that the analysis never ran.",
        root.display()
    );

    return ExitCode::Vacuous;
}

/// Renders the findings and decides the exit code.
fn Report_Findings(findings: &[Finding], examined: Examined, claim: Claim, stdout: &mut impl Write) -> ExitCode
{
    for finding in findings
    {
        let _ignored = writeln!(stdout, "{}", finding.Describe());
    }

    let blocking = findings
        .iter()
        .filter(|finding| return finding.Can_Fail_A_Build())
        .count();
    Print_Counts(findings, examined, claim, stdout);

    if blocking > 0
    {
        return ExitCode::Violations;
    }

    return ExitCode::Ok;
}

/// What was looked at, which is part of the result and not decoration.
///
/// "0 findings" over 4 files and "0 findings" over 400 are different claims, and so are
/// "400 files" and "400 files, 12 of which produced a fact" — a reader who cannot tell
/// them apart cannot tell a clean tree from a broken walk. And "0 findings" over a run that
/// reached a judgment about everything and "0 findings" over a run that could not judge a
/// third of it are a third pair a reader must not be left to conflate, which is what the
/// claim line and the coverage breakdown below say.
fn Print_Counts(findings: &[Finding], examined: Examined, claim: Claim, stdout: &mut impl Write)
{
    let found = findings.len();
    let blocking = findings
        .iter()
        .filter(|finding| return finding.Can_Fail_A_Build())
        .count();
    let _ignored = writeln!(
        stdout,
        "\n{} file(s) examined, {} with a syntax fact, {found} finding(s), {blocking} of \
         which can fail a build",
        examined.files,
        examined.facts
    );

    let _ignored = writeln!(stdout, "claim: {claim}");

    let coverage = Coverage::Of(findings);
    for (applicability, count) in coverage.Nonzero()
    {
        let _ignored = writeln!(stdout, "  {}: {count}", applicability.Label());
    }
}

/// Every selected rule whose population was empty, after the claim and apart from it.
///
/// Such a rule judged nothing: the run handed it no source of the language or kind its norm is
/// about. Its zero findings read exactly like a judgment of real subjects found clean, and this is
/// the one place a reader is told the difference. The claim above is what the findings support
/// and is not changed by it.
///
/// Prints nothing for a run in which every selected rule judged something.
fn Print_Empty_Populations(populations: &Populations, stdout: &mut impl Write)
{
    let empty = populations.Empty();
    if empty.is_empty()
    {
        return;
    }

    let _ignored = writeln!(stdout, "
{} rule(s) judged nothing, because no source was in their population:", empty.len());
    for rule in empty
    {
        let _ignored = writeln!(stdout, "  {rule}");
    }
}

/// Every value a selected rule read that the repository never declared, after the claim and the
/// population and apart from both, one line each: the rule, where the value would be declared, and
/// what the rule did without it.
///
/// A rule that judged against a value this workspace substituted reports exactly what it would
/// report had the repository declared that value, and a rule with no norm to judge by reports
/// nothing; this is the one place a reader learns which zero is which, and which remedy is theirs:
/// a change of code, or a declaration. The claim above is what the findings support and is not
/// changed by it. `OD-RULES-011` version 3 decisions 1 and 3.
///
/// Prints nothing for a run in which every value a rule read was declared.
fn Print_Undeclared_Values(undeclared: &UndeclaredValues, stdout: &mut impl Write)
{
    if undeclared.Is_Empty()
    {
        return;
    }

    let count: usize = undeclared.Named().iter().map(|(_, values)| return values.len()).sum();
    let _ignored = writeln!(stdout, "\n{count} value(s) the rules read were never declared by this repository:");
    for (rule, values) in undeclared.Named()
    {
        for value in values
        {
            let _ignored = writeln!(stdout, "  {rule}: {}", Undeclared_Line(value));
        }
    }
}

/// One undeclared value as a reader acts on it: where to declare it, and what the rule did instead.
fn Undeclared_Line(value: &UndeclaredValue) -> String
{
    let language = value.language.as_ref().map_or_else(String::new, |language| return format!(" for {language}"));
    let outcome = match &value.outcome
    {
        UndeclaredOutcome::JudgedAgainst { value } => format!("judged against {value}"),
        UndeclaredOutcome::JudgedNothing => "judged nothing".to_owned(),
        UndeclaredOutcome::ReportedUndeclared => "reported undeclared".to_owned(),
        UndeclaredOutcome::SpreadBoundNotJudged => "judged the audit and not the spread bound".to_owned(),
    };

    return format!("{} `{}`{language} in {}, {outcome}", value.family, value.key, value.declared_in);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_check_orchestration::{Claim_Of, SupportingFactTrail};
    use nomos_contracts::{Digest128, EvidenceClass, GateCategory, RuleId, SubjectId};

    /// The byte the fixture subject digest repeats to fill `Digest128`'s width. Opaque to
    /// every rendering here: nothing reads it, only its width matters.
    const EXAMPLE_SUBJECT_BYTE: u8 = 3;

    fn Finding_With(applicability: Applicability) -> Finding
    {
        return Finding {
            address: None,
            rule: RuleId::New("completeness-mirror"),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([EXAMPLE_SUBJECT_BYTE; Digest128::BYTE_LENGTH])),
            subject_name: "Example".to_owned(),
            applicability,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Advisory,
            summary: "example".to_owned(),
            locations: vec!["a.rs".to_owned()],
        };
    }

    /// Every [`Applicability`] variant, in no particular order -- the domain
    /// [`Test_Every_Applicability_Should_Land_In_Its_Own_Bucket`] iterates. Named once so a
    /// twelfth variant is a diff to this list, not to the test that reads it.
    const EVERY_APPLICABILITY: [Applicability; APPLICABILITY_VARIANT_COUNT] = [
        Applicability::Supported,
        Applicability::SupportedWithFallback,
        Applicability::PartiallySupported,
        Applicability::NotApplicable,
        Applicability::MissingCapability,
        Applicability::ProviderUnavailable,
        Applicability::DependencyUnavailable,
        Applicability::ConfigurationDisabled,
        Applicability::Unparseable,
        Applicability::AnalysisFailed,
        Applicability::AgentRequired,
    ];

    /// All eleven variants land in a bucket, and no two variants share one.
    #[test]
    fn Test_Every_Applicability_Should_Land_In_Its_Own_Bucket()
    {
        for applicability in EVERY_APPLICABILITY
        {
            let findings = vec![Finding_With(applicability)];
            let nonzero = Coverage::Of(&findings).Nonzero();

            assert_eq!(
                nonzero,
                vec![(applicability, 1)],
                "{applicability} did not land in exactly its own bucket"
            );
        }
    }

    /// The dispatcher itself, not only the outcomes it dispatches to -- a root that cannot
    /// be judged at all renders the `Unreadable` message and exit code, driven through
    /// `Render_Outcome`'s own top-level match rather than only observed indirectly through
    /// `check::Run`.
    #[test]
    fn Test_Render_Outcome_Should_Report_Unreadable_For_A_Root_That_Cannot_Be_Judged()
    {
        let root = Path::new("no-such-directory-for-render-outcome-test");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = Render_Outcome(root, &CheckOutcome::Unreadable, &mut stdout, &mut stderr);

        assert_eq!(code, ExitCode::Unreadable);
        assert!(stdout.is_empty(), "{}", String::from_utf8_lossy(&stdout));
        assert!(
            String::from_utf8_lossy(&stderr).contains("cannot judge"),
            "{}",
            String::from_utf8_lossy(&stderr)
        );
    }

    /// A run that judged nothing at all — the empty case — has no nonzero bucket, and is
    /// [`Claim::Complete`]: no subject was ever placed in a debt state, because no subject
    /// produced a finding of any kind. `Examined` is what tells that apart from a walk that
    /// found nothing.
    #[test]
    fn Test_No_Findings_Should_Be_Complete_With_No_Buckets()
    {
        assert!(Coverage::Of(&[]).Nonzero().is_empty());
        assert_eq!(Claim_Of(&[]), Claim::Complete);
    }

    /// The whole point: a debt or agent-required state must flip the roll-up verdict, and a
    /// deliberate absence must not.
    #[test]
    fn Test_Coverage_Debt_And_Agent_Required_Should_Be_Incomplete_Deliberate_Absences_Should_Not()
    {
        Assert_Claim_For_Each(
            [
                Applicability::MissingCapability,
                Applicability::ProviderUnavailable,
                Applicability::DependencyUnavailable,
                Applicability::Unparseable,
                Applicability::AnalysisFailed,
                Applicability::AgentRequired,
            ],
            Claim::Incomplete,
            "must mark the claim incomplete",
        );

        Assert_Claim_For_Each(
            [Applicability::NotApplicable, Applicability::ConfigurationDisabled],
            Claim::Complete,
            "is a decision, not a gap",
        );

        assert_eq!(Claim_Of(&[Finding_With(Applicability::Supported)]), Claim::Complete);
    }

    /// Every listed applicability, alone, must reduce to `expected` -- the assertion this
    /// test repeats for two different reasons the message text names.
    fn Assert_Claim_For_Each(variants: impl IntoIterator<Item = Applicability>, expected: Claim, why: &str)
    {
        for variant in variants
        {
            assert_eq!(Claim_Of(&[Finding_With(variant)]), expected, "{variant} {why}");
        }
    }

    /// The counts the report is asked to print: every file the walk read, and the files a
    /// syntax fact was materialized for. Named so the fixture and the assertions below that
    /// read the rendered text are two views of one pair of numbers.
    const EXAMINED_FILES: usize = 41;
    const EXAMINED_FACTS: usize = 39;

    /// The counts are part of the result. Without them, a broken walk and a clean tree
    /// render the same line.
    #[test]
    fn Test_The_Report_Should_Say_How_Much_Was_Looked_At()
    {
        let mut stdout = Vec::new();

        let _code = Report_Findings(&[], Examined { files: EXAMINED_FILES, facts: EXAMINED_FACTS }, Claim::Complete, &mut stdout);

        let rendered = String::from_utf8(stdout).expect("output is utf-8");

        assert!(rendered.contains("41 file(s) examined"), "{rendered}");
        assert!(rendered.contains("39 with a syntax fact"), "{rendered}");
    }

    /// The negative control `OD-COMPLETENESS-004` asks for: a subject the run could not
    /// judge must render distinguishably from a subject that was judged clean, in the
    /// rendered text a person actually reads — not only in a field nobody prints.
    #[test]
    fn Test_A_Coverage_Debt_Subject_Must_Not_Render_The_Same_As_A_Clean_Run()
    {
        let clean_examined = Examined { files: 1, facts: 1 };
        let mut clean_stdout = Vec::new();
        let _clean_code = Report_Findings(&[], clean_examined, Claim_Of(&[]), &mut clean_stdout);
        let clean_rendered = String::from_utf8(clean_stdout).expect("Report_Findings writes only str into the buffer");

        let debt = vec![Finding_With(Applicability::DependencyUnavailable)];
        let debt_examined = Examined { files: 1, facts: 0 };
        let mut debt_stdout = Vec::new();
        let _debt_code = Report_Findings(&debt, debt_examined, Claim_Of(&debt), &mut debt_stdout);
        let debt_rendered = String::from_utf8(debt_stdout).expect("Report_Findings writes only str into the buffer");

        assert_ne!(
            clean_rendered, debt_rendered,
            "a run carrying coverage debt rendered identically to a clean run"
        );
        assert!(clean_rendered.contains("claim: complete"), "{clean_rendered}");
        assert!(debt_rendered.contains("claim: incomplete"), "{debt_rendered}");
        assert!(debt_rendered.contains("DependencyUnavailable"), "{debt_rendered}");
        assert!(!clean_rendered.contains("DependencyUnavailable"), "{clean_rendered}");
    }

    fn Judged_Over(populations: Populations) -> String
    {
        let outcome = CheckOutcome::Judged {
            findings: Vec::new(),
            examined: Examined { files: 1, facts: 1 },
            claim: Claim_Of(&[]),
            supporting_facts: SupportingFactTrail::New(),
            populations,
            undeclared: nomos_check_orchestration::UndeclaredValues::New(),
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let _code = Render_Outcome(Path::new("."), &outcome, &mut stdout, &mut stderr);
        return String::from_utf8(stdout).expect("Render_Outcome writes only str into the buffer");
    }

    /// A rule whose population was empty is named after the claim, the rule that judged a real
    /// source is not, and the claim reads what it reads with no population reported at all.
    #[test]
    fn Test_An_Empty_Population_Should_Be_Named_Apart_From_An_Unchanged_Claim()
    {
        let mut populations = Populations::New();
        populations.Note(RuleId::New("go-only-rule"), 0);
        populations.Note(RuleId::New("rust-only-rule"), 3);

        let rendered = Judged_Over(populations);
        let without = Judged_Over(Populations::New());

        assert!(rendered.contains("1 rule(s) judged nothing, because no source was in their population:\n  go-only-rule\n"), "{rendered}");
        assert!(!rendered.contains("rust-only-rule"), "{rendered}");
        assert!(rendered.contains("claim: complete"), "{rendered}");
        assert!(rendered.starts_with(&without), "the report up to and including the claim is unchanged:\n{rendered}\n---\n{without}");
    }

    /// A run in which every selected rule judged something prints nothing new.
    #[test]
    fn Test_A_Run_With_No_Empty_Population_Should_Print_Nothing_New()
    {
        let mut populations = Populations::New();
        populations.Note(RuleId::New("rust-only-rule"), 3);

        assert_eq!(Judged_Over(populations), Judged_Over(Populations::New()));
    }

    fn Judged_With(undeclared: UndeclaredValues) -> String
    {
        let outcome = CheckOutcome::Judged {
            findings: Vec::new(),
            examined: Examined { files: 1, facts: 1 },
            claim: Claim_Of(&[]),
            supporting_facts: SupportingFactTrail::New(),
            populations: Populations::New(),
            undeclared,
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let _code = Render_Outcome(Path::new("."), &outcome, &mut stdout, &mut stderr);
        return String::from_utf8(stdout).expect("Render_Outcome writes only str into the buffer");
    }

    /// Every value a rule read and nobody declared is named after the claim, with where it would be
    /// declared and what the rule did instead, and the report up to the claim does not move.
    #[test]
    fn Test_An_Undeclared_Value_Should_Be_Named_After_An_Unchanged_Claim()
    {
        let mut undeclared = UndeclaredValues::New();
        undeclared.Note(
            RuleId::New("1500-lines"),
            vec![UndeclaredValue {
                family: "limits",
                declared_in: "nomos-limits.json",
                key: "file-size-hard-lines",
                language: None,
                outcome: UndeclaredOutcome::JudgedAgainst { value: "1500".to_owned() },
            }],
        );
        undeclared.Note(
            RuleId::New("go-rule"),
            vec![UndeclaredValue {
                family: "naming",
                declared_in: "standards.json",
                key: "type.exported",
                language: Some("go".to_owned()),
                outcome: UndeclaredOutcome::JudgedAgainst { value: "upper-camel".to_owned() },
            }],
        );

        let rendered = Judged_With(undeclared);
        let without = Judged_With(UndeclaredValues::New());

        assert!(
            rendered.contains(
                "2 value(s) the rules read were never declared by this repository:\n  \
                 1500-lines: limits `file-size-hard-lines` in nomos-limits.json, judged against 1500\n  \
                 go-rule: naming `type.exported` for go in standards.json, judged against upper-camel\n"
            ),
            "{rendered}"
        );
        assert!(rendered.contains("claim: complete"), "{rendered}");
        assert!(rendered.starts_with(&without), "the report up to and including the claim is unchanged:\n{rendered}\n---\n{without}");
    }

    /// A run in which every value its rules read was declared prints nothing new: a rule that names
    /// none is not listed.
    #[test]
    fn Test_A_Run_With_Nothing_Undeclared_Should_Print_Nothing_New()
    {
        let mut undeclared = UndeclaredValues::New();
        undeclared.Note(RuleId::New("1500-lines"), Vec::new());

        assert_eq!(Judged_With(undeclared), Judged_With(UndeclaredValues::New()));
    }
}

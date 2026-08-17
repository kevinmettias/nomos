//! Saying what was found, over how much, what would fail a build, and what the run could
//! not judge.

use super::{Finding, Write, ExitCode};
use nomos_contracts::Applicability;

/// How much of the world this run actually saw.
///
/// Two denominators and not one. "0 findings over 400 files" and "0 findings over 400
/// files none of which produced a fact" are different claims, and the second is a broken
/// run — the prototype reported the first shape for a check that had walked nothing, and
/// the defect was invisible because the report had no place to put the number that would
/// have shown it. That reasoning still holds and is unchanged by `OD-COMPLETENESS-004`:
/// it is a property of the walk, decided before a single subject is judged, and it is
/// orthogonal to what judgment each subject received. [`Coverage`], below, is the type
/// that carries the eleven-way answer to the second question — deliberately not a third
/// field here, for the reason its own doc comment gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Examined
{
    /// Files the walk read.
    pub(super) files: usize,
    /// Files a syntax fact was materialized for.
    pub(super) facts: usize,
}

/// How many subjects this run placed in each [`Applicability`] state.
///
/// One counter per variant, filled by a match with no wildcard arm — the same shape
/// [`Applicability`] itself is built with, so a twelfth state added there stops this file
/// compiling rather than landing silently in whichever bucket the match happened to fall
/// through to.
///
/// `OD-COMPLETENESS-004` is why this exists beside [`Examined`] rather than growing it by
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

    /// All eleven buckets, labelled, in the order [`Applicability`] declares them.
    ///
    /// Grouped by variant rather than by capability or by language, because this build
    /// registers one capability and one language: either of those groupings would collapse
    /// to a single bucket today and hide exactly the distinction this type exists to show.
    /// Nothing here prevents composing a second grouping later — [`Finding::rule`] is
    /// already a capability's proxy — but a grouping that answers nothing at the current
    /// scale is not the one to ship first. `OD-COMPLETENESS-004` records the choice.
    fn All(&self) -> [(Applicability, usize); 11]
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

    /// The buckets this run actually put something in.
    ///
    /// Zero buckets are omitted from the rendering rather than printed at zero: eleven lines
    /// on every run, ten of them reading `: 0`, is the kind of output a reader learns to skip
    /// past, which is exactly how a silent omission stayed silent the first time.
    fn Nonzero(&self) -> Vec<(Applicability, usize)>
    {
        return self.All().into_iter().filter(|&(_, count)| return count > 0).collect();
    }
}

/// Whether this run reached a judgment about everything it touched.
///
/// Not implied by zero blocking findings. [`Claim::Incomplete`] is a fact about the run's
/// reach, not about severity, and `OD-COMPLETENESS-004` records that this build keeps it a
/// fact reported in the text rather than a fact the exit code carries — see the record for
/// why splitting the exit code here is not the remedy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Claim
{
    /// No subject fell into a debt or agent-required state. Deliberate absences —
    /// [`Applicability::NotApplicable`] and [`Applicability::ConfigurationDisabled`] — do
    /// not break this, for the same reason [`Applicability::Is_Coverage_Debt`] excludes
    /// them: a decision is not a gap.
    Complete,
    /// At least one subject fell into [`Applicability::Is_Coverage_Debt`] or
    /// [`Applicability::Requires_Agent`]. The run did not reach a judgment about it, and
    /// that is a different claim from reaching one and finding it clean.
    Incomplete,
}

impl core::fmt::Display for Claim
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(match self
        {
            Self::Complete => "complete",
            Self::Incomplete => "incomplete",
        });
    }
}

/// The roll-up verdict, read off [`Applicability`]'s own predicates rather than
/// re-derived from [`Coverage`]'s counters — one classification, asked once, so a change to
/// what counts as debt cannot drift between the two.
fn Claim_Of(findings: &[Finding]) -> Claim
{
    let unjudged = findings.iter().any(|finding| {
        return finding.applicability.Is_Coverage_Debt() || finding.applicability.Requires_Agent();
    });

    return if unjudged { Claim::Incomplete } else { Claim::Complete };
}

/// Renders the findings and decides the exit code.
pub(super) fn Report(findings: &[Finding], examined: Examined, stdout: &mut impl Write) -> ExitCode
{
    for finding in findings
    {
        let _ignored = writeln!(stdout, "{}", finding.Describe());
    }

    let blocking = findings
        .iter()
        .filter(|finding| return finding.Can_Fail_A_Build())
        .count();
    Counts(findings, blocking, examined, stdout);

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
pub(super) fn Counts(findings: &[Finding], blocking: usize, examined: Examined, stdout: &mut impl Write)
{
    let found = findings.len();
    let _ignored = writeln!(
        stdout,
        "\n{} file(s) examined, {} with a syntax fact, {found} finding(s), {blocking} of \
         which can fail a build",
        examined.files,
        examined.facts
    );

    let claim = Claim_Of(findings);
    let _ignored = writeln!(stdout, "claim: {claim}");

    let coverage = Coverage::Of(findings);
    for (applicability, count) in coverage.Nonzero()
    {
        let _ignored = writeln!(stdout, "  {}: {count}", applicability.Label());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{Digest128, EvidenceClass, GateCategory, RuleId, SubjectId};

    fn Finding_With(applicability: Applicability) -> Finding
    {
        return Finding {
            rule: RuleId::New("completeness-mirror"),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([3; Digest128::BYTE_LENGTH])),
            subject_name: "Example".to_owned(),
            applicability,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Advisory,
            summary: "example".to_owned(),
            locations: vec!["a.rs".to_owned()],
        };
    }

    /// All eleven variants land in a bucket, and no two variants share one.
    #[test]
    fn Test_Every_Applicability_Should_Land_In_Its_Own_Bucket()
    {
        let all = [
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

        for applicability in all
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
        for debt in [
            Applicability::MissingCapability,
            Applicability::ProviderUnavailable,
            Applicability::DependencyUnavailable,
            Applicability::Unparseable,
            Applicability::AnalysisFailed,
            Applicability::AgentRequired,
        ]
        {
            assert_eq!(
                Claim_Of(&[Finding_With(debt)]),
                Claim::Incomplete,
                "{debt} must mark the claim incomplete"
            );
        }

        for deliberate in [Applicability::NotApplicable, Applicability::ConfigurationDisabled]
        {
            assert_eq!(
                Claim_Of(&[Finding_With(deliberate)]),
                Claim::Complete,
                "{deliberate} is a decision, not a gap"
            );
        }

        assert_eq!(Claim_Of(&[Finding_With(Applicability::Supported)]), Claim::Complete);
    }

    /// The negative control `OD-COMPLETENESS-004` asks for: a subject the run could not
    /// judge must render distinguishably from a subject that was judged clean, in the
    /// rendered text a person actually reads — not only in a field nobody prints.
    #[test]
    fn Test_A_Coverage_Debt_Subject_Must_Not_Render_The_Same_As_A_Clean_Run()
    {
        let mut clean_stdout = Vec::new();
        let _clean_code = Report(&[], Examined { files: 1, facts: 1 }, &mut clean_stdout);
        let clean_rendered = String::from_utf8(clean_stdout).expect("utf-8");

        let debt = vec![Finding_With(Applicability::DependencyUnavailable)];
        let mut debt_stdout = Vec::new();
        let _debt_code = Report(&debt, Examined { files: 1, facts: 0 }, &mut debt_stdout);
        let debt_rendered = String::from_utf8(debt_stdout).expect("utf-8");

        assert_ne!(
            clean_rendered, debt_rendered,
            "a run carrying coverage debt rendered identically to a clean run"
        );
        assert!(clean_rendered.contains("claim: complete"), "{clean_rendered}");
        assert!(debt_rendered.contains("claim: incomplete"), "{debt_rendered}");
        assert!(debt_rendered.contains("DependencyUnavailable"), "{debt_rendered}");
        assert!(!clean_rendered.contains("DependencyUnavailable"), "{clean_rendered}");
    }
}

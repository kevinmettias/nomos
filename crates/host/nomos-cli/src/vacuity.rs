//! Where every group's no-vacuous-success guarantee is declared, once, in one place a
//! fifth group cannot skip.
//!
//! `OD-GATE-003` is the record this module exists to satisfy. `check.rs`'s own doc comment
//! already argues, correctly, that the guard itself belongs with the caller that chose the
//! subject — "did I see a plausible amount of the world" is a question only that caller can
//! answer, and this module does not move it: `check::sources::Walked` and
//! `nomos_check_orchestration::Run` still decide `check`'s answer, and
//! `spec::reporting::Absent_Or` still decides `spec`'s. What was missing was not the guard,
//! it was a place the guard's *existence* is recorded where the next author is standing —
//! two independent doc comments, each readable only by whoever opened that one module,
//! is a decision the next author does not encounter.
//!
//! [`Group`] closes the set of things `main.rs` dispatches to, and [`Stance_Of`] matches it
//! with no wildcard arm: a sixth group added to [`Group`] without a corresponding arm here
//! fails `cargo test` and the gate's `Lint` step at this match, beside the group it belongs
//! to, rather than shipping a vacuous-success risk nobody wrote down. The tests below are
//! the other half — they do not trust a [`Stance::Guarded`] line, they drive the named
//! group's own `Run` over a subject deliberately emptied and check the exit code is not
//! that group's `Ok`.
//!
//! # What this does not cover
//!
//! This is a guarantee about the exit code `main.rs` produces, which is the boundary
//! `.github/workflows/gate.yml` can actually observe — `OD-GATE-004` is why that boundary is
//! the one worth guarding. It says nothing about a caller that never goes through `main`: a
//! library consumer that calls `check::Run` or `spec::Run` directly gets exactly the guard
//! each module's own code already gives it, no more and no less. A type in `nomos-contracts`
//! that made a judging command's success value unconstructible without evidence would cover
//! that caller too, because it would not be possible to *have* an `Ok` value without having
//! gone through the construction that proves something was judged, however it was reached.
//! `docs/records/OD-GATE-003` names that alternative and why this module does not become it:
//! `nomos-contracts` is Band 0, read by peers that never compile this crate, and is admitted
//! to only by `OD-CONTRACTS-001`'s test — would a peer that never compiles this crate be
//! unable to agree with us without this type? An exit code this binary's own `main` produces
//! is not part of that shared protocol; it is what happens at a process boundary this crate's
//! other peers never cross.
//!
//! Adding a [`Group`] variant when `main.rs` gains a dispatch arm is still on the next
//! author to remember — nothing here makes that step itself compile-fail. What compiles or
//! tests fail is everything downstream of remembering: an added variant with no [`Stance_Of`]
//! arm, or a [`Stance::Guarded`] claim the real `Run` does not actually honour.

/// Every group this binary dispatches to, named once so `main.rs`'s match and this module's
/// [`Stance_Of`] cannot silently drift apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Group
{
    Work,
    Spec,
    Check,
    Request,
    Gate,
}

#[cfg(test)]
impl Group
{
    /// Every group, for a test to walk without hand-maintaining a second list.
    pub(crate) const ALL: [Self; 5] =
        [Self::Work, Self::Spec, Self::Check, Self::Request, Self::Gate];
}

/// Every group's on-argv spelling, paired with the [`Group`] `Stance_Of` reads.
///
/// `main.rs`'s dispatch matches through [`Named`] rather than comparing strings itself, so
/// a fifth group has to be spelled here — beside [`Stance_Of`]'s own match — before `main.rs`
/// can route to it at all. That is what makes this module the entry point every group's
/// dispatch actually passes through, and not merely a place a stance happens to be written
/// down beside the code it describes.
pub(crate) const NAMES: [(&str, Group); 5] = [
    ("work", Group::Work),
    ("spec", Group::Spec),
    ("check", Group::Check),
    ("request", Group::Request),
    ("gate", Group::Gate),
];

/// The group named on argv, if [`NAMES`] spells it.
pub(crate) fn Named(text: &str) -> Option<Group>
{
    for (name, group) in NAMES
    {
        if name == text
        {
            return Some(group);
        }
    }

    return None;
}

/// Whether a group can report a clean run over a subject set it never examined, and where
/// that is decided if it can.
///
/// `#[cfg(test)]`, along with [`Stance_Of`] below: nothing at runtime consults a group's
/// stance, so carrying it into the shipped binary would be dead weight the compiler is
/// right to flag. The exhaustiveness this type exists for is still checked on every commit
/// — `.github/workflows/gate.yml`'s `Lint` step runs `cargo clippy --workspace
/// --all-targets` and its `Test` step runs `cargo test --workspace`, and `--all-targets`
/// and `test` both compile this module's `#[cfg(test)]` code. A group added to [`Group`]
/// with no arm here fails both, not neither.
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Stance
{
    /// This group looks up or walks a caller-chosen subject set and refuses its own `Ok`
    /// when that set turns out empty. `decided_in` names the function that makes the
    /// refusal, so a reader can go see the actual guard rather than trust this line.
    Guarded { decided_in: &'static str },
    /// This group does not judge a caller-chosen subject set, so there is nothing here that
    /// could be vacuously clean. `because` is the reason, not a placeholder for one.
    NotApplicable { because: &'static str },
}

/// The one place every group's stance is declared. No wildcard arm: a group added to
/// [`Group`] without an arm here fails the build at this match, beside the group it belongs
/// to.
#[cfg(test)]
pub(crate) fn Stance_Of(group: Group) -> Stance
{
    return match group
    {
        Group::Check => Stance::Guarded {
            decided_in: "check::sources::Walked / nomos_check_orchestration::Run",
        },
        Group::Spec => Stance::Guarded {
            decided_in: "spec::reporting::Absent_Or",
        },
        Group::Work => Stance::NotApplicable {
            because: "work verbs report the ledger's own state; an empty board is a true \
                      empty board, not a broken read of one that has entries",
        },
        Group::Request => Stance::NotApplicable {
            because: "submit always names exactly one submission the caller wrote; there is \
                      no caller-chosen subject set for it to have walked and found empty the \
                      way a checked-out tree or a queried record can be",
        },
        Group::Gate => Stance::Guarded {
            decided_in: "gate::sources::Walked / nomos_check_orchestration::Run",
        },
    };
}

#[cfg(test)]
mod tests
{
    use super::{Group, Stance, Stance_Of};

    /// Every declared group has a stance, and calling it does not panic. Mostly load-bearing
    /// for the match arm itself: the real guarantee below is that a declared
    /// [`Stance::Guarded`] is checked against the group's actual `Run`, not trusted.
    #[test]
    fn Test_Every_Group_Should_Have_A_Declared_Stance()
    {
        for group in Group::ALL
        {
            let _stance = Stance_Of(group);
        }
    }

    /// `check` walks a caller-chosen tree, so an empty one must not report `Ok`.
    ///
    /// This is the same claim `check.rs`'s own doc comment and `check::tests` already make
    /// about `check::Run`; asserted again here so a reader who wants "does every group this
    /// module calls `Guarded` actually refuse" does not have to trust a `decided_in` string
    /// and go find it themselves.
    #[test]
    fn Test_Check_Should_Refuse_Ok_Over_An_Empty_Tree()
    {
        assert!(matches!(Stance_Of(Group::Check), Stance::Guarded { .. }));

        let empty = std::env::temp_dir().join("nomos-cli-vacuity-guard-empty-check-tree");
        let _ignored = std::fs::remove_dir_all(&empty);
        std::fs::create_dir_all(&empty).expect("creates an empty directory");

        let command = crate::check::CheckCommand { root: empty.clone() };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = crate::check::Run(&command, &mut stdout, &mut stderr);

        let _ignored = std::fs::remove_dir_all(&empty);

        assert_ne!(code.Value(), 0, "an empty tree must not report the same code as a clean run");
        assert_eq!(code, crate::check::ExitCode::Vacuous);
    }

    /// `spec record` looks up one identifier in an assembled store, so an identifier the
    /// store never had — with no corpus configured to have supplied it — must not report
    /// `Ok`.
    #[test]
    fn Test_Spec_Should_Refuse_Ok_Over_A_Record_The_Store_Never_Had()
    {
        assert!(matches!(Stance_Of(Group::Spec), Stance::Guarded { .. }));

        let arguments = vec![
            "record".to_owned(),
            "--id".to_owned(),
            "OD-GATE-003-VACUITY-GUARD-TEST-NONEXISTENT".to_owned(),
        ];
        let command = crate::spec::Parse(&arguments).expect("parses");
        let request = nomos_spec_orchestration::corpus::CorpusRequest {
            variable: "NOMOS_VACUITY_GUARD_TEST_CORPUS_UNSET".to_owned(),
            root: None,
            revision: nomos_spec_orchestration::corpus::DEFAULT_REVISION.to_owned(),
        };
        let mut output = Vec::new();
        let mut notes = Vec::new();

        let code = crate::spec::Run(&command, &request, &mut output, &mut notes);

        assert_ne!(code.Value(), 0, "a record the store never had must not report the same code as a clean run");
        assert_eq!(code, crate::spec::ExitCode::Absent);
    }

    /// `work` and `request` are declared [`Stance::NotApplicable`] rather than
    /// [`Stance::Guarded`], and that declaration is what this test holds to — not a claim
    /// this module can drive their `Run` functions to falsify, because neither takes a
    /// caller-chosen subject set that could be made empty the way a tree or a store lookup
    /// can be. The reasoning is `Stance_Of`'s `because` field, read by a human, not asserted
    /// by this test.
    #[test]
    fn Test_Work_And_Request_Are_Declared_Not_Applicable()
    {
        assert!(matches!(Stance_Of(Group::Work), Stance::NotApplicable { .. }));
        assert!(matches!(Stance_Of(Group::Request), Stance::NotApplicable { .. }));
    }

    /// `gate run` walks a caller-chosen tree exactly as `check` does, so an empty one must
    /// not report `Ok` there either -- the same claim
    /// `Test_Check_Should_Refuse_Ok_Over_An_Empty_Tree` makes for `check`, checked at this
    /// seam too rather than trusted by analogy. `gate plan` itself has no vacuity condition
    /// of its own -- it reports a fixed registry, not a caller-chosen subject set -- but
    /// `Group::Gate`'s stance is declared for the group, not per verb, and `run`'s real
    /// guard is what makes `Guarded` the honest declaration now.
    #[test]
    fn Test_Gate_Run_Should_Refuse_Ok_Over_An_Empty_Tree()
    {
        assert!(matches!(Stance_Of(Group::Gate), Stance::Guarded { .. }));

        let empty = std::env::temp_dir().join("nomos-cli-vacuity-guard-empty-gate-tree");
        let _ignored = std::fs::remove_dir_all(&empty);
        std::fs::create_dir_all(&empty).expect("creates an empty directory");

        let invocation = crate::gate::GateInvocation::Run(crate::gate::GateCommand { root: empty.clone() });
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = crate::gate::Run(&invocation, &mut stdout, &mut stderr);

        let _ignored = std::fs::remove_dir_all(&empty);

        assert_ne!(code.Value(), 0, "an empty tree must not report the same code as a clean run");
        assert_eq!(code, crate::gate::ExitCode::Vacuous);
    }
}

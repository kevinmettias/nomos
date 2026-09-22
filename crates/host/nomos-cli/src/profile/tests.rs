//! What `nomos profile` reports, driven over temporary trees of known shape.
//!
//! The assertions are structural rather than textual wherever a structure exists to assert
//! on. [`Section`] parses the rendered report back into headings and their indented entries,
//! so a test says "every policy file the profile knows about has exactly one line, and the
//! absent ones say absent" rather than matching a sentence -- which would go red on a
//! reworded explanation while staying green on a dropped row, exactly backwards.

use super::{
    availability, parsing, report, starter_policy, CapabilityStanding, ExitCode, OfferStanding, ProfileCommand, ProviderStanding, Run,
    StarterOutcome,
};
use nomos_composer_std::FILE_SYSTEM;
use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_platform::{Environment, EnvironmentError, Timestamp};
use nomos_workspace_discovery::WorkspaceProfile;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The heading the source counts are rendered under.
const SOURCES_HEADING: &str = "sources the walk found";

/// The heading the language manifests are rendered under.
const MANIFESTS_HEADING: &str = "language manifests at the root";

/// The heading the repository policy files are rendered under.
const POLICY_FILES_HEADING: &str = "repository policy files at the root";

/// The heading the capability standings are rendered under.
const CAPABILITIES_HEADING: &str = "capabilities this workspace declares, and what this host settles about answering them";

/// The gate policy file this verb can write a starter for.
const GATE_POLICY_FILE: &str = "nomos-gate.json";

/// The indent a section's own entries carry, removed by [`Section`] so what is left is the
/// structure inside the section rather than the report's own margin.
const SECTION_INDENT: &str = "  ";

/// The further indent a provider row carries under its capability's verdict line, which is
/// what tells the two kinds of row in the capabilities section apart.
const OFFER_INDENT: &str = "  ";

#[test]
fn Test_The_Parser_Should_Default_The_Root_And_Write_Nothing()
{
    let command = parsing::Profile_Command_From_String_Arguments(&[]).expect("no argument is present for the parser to refuse");

    assert_eq!(command, ProfileCommand { root: PathBuf::from("."), write_gate_policy: false });
}

#[test]
fn Test_The_Parser_Should_Read_A_Root_And_The_Starter_Flag()
{
    let arguments = Arguments("--root some/tree --write-gate-policy");

    let command = parsing::Profile_Command_From_String_Arguments(&arguments).expect("both flags are ones this group accepts");

    assert_eq!(command, ProfileCommand { root: PathBuf::from("some/tree"), write_gate_policy: true });
}

/// A mistyped starter flag must be refused rather than read as "no starter file wanted",
/// which would report success for a write that never happened.
#[test]
fn Test_The_Parser_Should_Refuse_A_Flag_It_Does_Not_Accept()
{
    let arguments = Arguments("--write-gate-policies");

    let error = parsing::Profile_Command_From_String_Arguments(&arguments).expect_err("that flag is not this group's");

    assert!(error.contains("unknown argument"), "{error}");
}

/// A root of known shape: one Rust source, no Go, Cargo's manifest but neither of Go's, and
/// two of the four policy files.
///
/// Every list is asserted whole -- one line per entry the profile carries, in the profile's
/// own order -- rather than by looking for the entries this fixture happens to have. A
/// report that silently dropped the rows for what is *absent* would pass every "contains"
/// assertion and fail this one, and the absent rows are the half a person adopting the tool
/// acts on.
#[test]
fn Test_A_Rich_Root_Should_Render_Every_Row_The_Profile_Carries()
{
    let root = Fresh_Root("nomos-cli-profile-rich-root");
    Write_Fixture(&root.join("a.rs"), "pub fn One() {}\n");
    Write_Fixture(&root.join("Cargo.toml"), "[package]\nname = \"example\"\n");
    Write_Fixture(&root.join("standards.json"), "{}\n");

    let rendered = Rendered_Profile(&root);
    let profile = WorkspaceProfile::Of_Root(&root).expect("the root was created above as a directory");
    let _ignored = std::fs::remove_dir_all(&root);

    assert_eq!(Section(&rendered, SOURCES_HEADING), vec!["rs: 1".to_owned(), "go: 0".to_owned()]);
    assert_eq!(Section(&rendered, MANIFESTS_HEADING), Manifest_Rows(&profile));
    assert_eq!(Section(&rendered, POLICY_FILES_HEADING), Policy_Rows(&profile));
    assert!(rendered.contains("repository root marker: present"), "{rendered}");
}

/// An empty directory is a real profile, not a refusal: every count zero, every manifest and
/// policy file absent, no root marker. Somebody adopting this tool from an empty directory
/// should read exactly that, which is why `crate::vacuity` declares this group not-applicable
/// rather than guarded.
#[test]
fn Test_An_Empty_Root_Should_Render_Every_Row_At_Its_Empty_Value()
{
    let root = Fresh_Root("nomos-cli-profile-empty-root");

    let rendered = Rendered_Profile(&root);
    let profile = WorkspaceProfile::Of_Root(&root).expect("the root was created above as a directory");
    let _ignored = std::fs::remove_dir_all(&root);

    assert_eq!(Section(&rendered, SOURCES_HEADING), vec!["rs: 0".to_owned(), "go: 0".to_owned()]);
    assert!(Section(&rendered, MANIFESTS_HEADING).iter().all(|row| return row.contains("absent")), "{rendered}");
    assert!(Section(&rendered, POLICY_FILES_HEADING).iter().all(|row| return row.ends_with("absent")), "{rendered}");
    assert_eq!(Section(&rendered, POLICY_FILES_HEADING).len(), profile.policy_files.len());
    assert!(rendered.contains("repository root marker: absent"), "{rendered}");
}

/// A path that is not a directory is not an empty repository. Reporting it as one would tell
/// a reader a walk found nothing where a walk could not have looked.
#[test]
fn Test_A_Root_That_Is_Not_A_Directory_Should_Refuse_Rather_Than_Profile_As_Empty()
{
    let command = ProfileCommand { root: PathBuf::from("no-such-tree-anywhere-for-the-profile-verb"), write_gate_policy: false };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&command, &mut stdout, &mut stderr);

    assert_eq!(code, ExitCode::Unusable);
    assert!(stdout.is_empty(), "nothing was profiled, so nothing may be rendered as a profile");
}

/// Every capability the composed registry declares gets exactly one verdict, and every offer
/// it holds gets exactly one row, in one fixed order. This is the claim that makes the
/// section a diagnosis rather than a list: a capability or an offer the run will ask about
/// and this report skipped is the silent incompleteness the whole verb exists to prevent.
#[test]
fn Test_Every_Declared_Capability_And_Every_Offer_Should_Get_Exactly_One_Row()
{
    let registry = nomos_check_orchestration::Registered().expect("this build's own composition must not be self-contradictory");
    let mut expected: Vec<(String, Vec<String>)> = registry
        .Declared()
        .map(|contract| {
            let offers = registry.Offers(&contract.id).iter().map(|offer| return offer.provider.As_Str().to_owned());
            return (contract.id.As_Str().to_owned(), offers.collect());
        })
        .collect();
    expected.sort();

    let standings = availability::Capability_Standings(&registry, &Fixed_Host(&[]), &FILE_SYSTEM);

    let reported: Vec<(String, Vec<String>)> = standings
        .iter()
        .map(|standing| {
            let offers = standing.offers.iter().map(|offer| return offer.provider.clone());
            return (standing.capability.clone(), offers.collect());
        })
        .collect();
    assert_eq!(reported, expected);
}

/// A provider that runs a program this host does not have is *unavailable*, with the tool
/// named -- and the name comes from the environment variable the real providers read, not
/// from the fallback, so a host whose `CARGO` points somewhere is told about that program
/// rather than about one it never launches.
#[test]
fn Test_A_Provider_Whose_Program_Is_Absent_Should_Name_The_Missing_Tool()
{
    let empty = Fresh_Root("nomos-cli-profile-no-programs-here");
    let host = Fixed_Host(&[("PATH", &empty.display().to_string()), ("CARGO", "a-cargo-that-is-not-here")]);

    let standing = availability::Standing_Of_Provider("nomos.lang.rust.clippy", &host, &FILE_SYSTEM);

    let _ignored = std::fs::remove_dir_all(&empty);
    assert_eq!(standing, ProviderStanding::ToolMissing { tool: "a-cargo-that-is-not-here".to_owned() });
}

/// The same provider on a host that *does* carry the program is not available -- it is
/// undetermined. `cargo` being on the path says nothing about whether `cargo clippy` is
/// installed, and the only thing that settles it is running it.
///
/// This is the falsifier for the whole undetermined/available distinction: delete the
/// `ProgramSearch::Found` arm's `Undetermined` and make it `InThisBinary` and this test is
/// what goes red, while `Test_A_Provider_Whose_Program_Is_Absent_Should_Name_The_Missing_Tool`
/// above stays green.
#[test]
fn Test_A_Provider_Whose_Program_Is_Present_Should_Be_Undetermined_Rather_Than_Available()
{
    let directory = Fresh_Root("nomos-cli-profile-a-program-lives-here");
    Write_Fixture(&directory.join("a-cargo-that-is-here"), "not really a program\n");
    let host = Fixed_Host(&[("PATH", &directory.display().to_string()), ("CARGO", "a-cargo-that-is-here")]);

    let standing = availability::Standing_Of_Provider("nomos.lang.rust.clippy", &host, &FILE_SYSTEM);

    let _ignored = std::fs::remove_dir_all(&directory);
    assert!(matches!(standing, ProviderStanding::Undetermined { .. }), "{standing:?}");
}

/// A host with no search path at all could not look, which is not the same as having looked
/// and found nothing. Naming a tool to install here would be naming one that may well
/// already be installed.
#[test]
fn Test_A_Host_With_No_Search_Path_Should_Be_Undetermined_Rather_Than_Missing_A_Tool()
{
    let standing = availability::Standing_Of_Provider("nomos.lang.rust.clippy", &Fixed_Host(&[]), &FILE_SYSTEM);

    assert!(matches!(standing, ProviderStanding::Undetermined { .. }), "{standing:?}");
}

/// A provider that answers from inside this binary is available whatever the host carries --
/// there is no program for a search path to be missing.
#[test]
fn Test_A_Provider_That_Launches_Nothing_Should_Be_Available_On_A_Bare_Host()
{
    let standing = availability::Standing_Of_Provider("nomos.repo.standards", &Fixed_Host(&[]), &FILE_SYSTEM);

    assert_eq!(standing, ProviderStanding::InThisBinary);
}

/// The conservative default, driven rather than declared: a provider this build carries no
/// declaration for reads as undetermined, never as available.
///
/// This is the falsifier for `provider_need::Need_Of`'s `None` arm. A provider composed after
/// that table was written must show up as something this verb does not know about, because
/// the alternative -- defaulting to available -- would hand out exactly the silent green this
/// verb exists to replace.
#[test]
fn Test_An_Unknown_Provider_Should_Be_Undetermined_Rather_Than_Available()
{
    let standing = availability::Standing_Of_Provider("nomos.provider.nothing.declares", &Fixed_Host(&[]), &FILE_SYSTEM);

    assert!(matches!(standing, ProviderStanding::Undetermined { .. }), "{standing:?}");
}

/// Every capability gets a verdict line and every offer under it gets a row, whichever
/// standing it carries -- the rendering is exercised over hand-built standings rather than
/// only over this host's, so the arms a given machine does not reach are still executed
/// somewhere.
#[test]
fn Test_Every_Capability_Should_Render_A_Verdict_And_Every_Offer_A_Row()
{
    let standings = Hand_Built_Standings();
    let mut stdout = Vec::new();

    report::Render_Standings(&standings, &mut stdout);

    let rendered = String::from_utf8(stdout).expect("the renderer writes text");
    assert_eq!(Verdict_Lines(&rendered), vec!["a.capability: available", "b.capability: unavailable", "c.capability: unavailable"]);
    assert_eq!(
        Offer_Lines(&rendered),
        vec![
            "a.linked: available -- answers from inside this binary and launches nothing",
            "a.unsettled: undetermined -- a-reason",
            "b.toolless: unavailable -- runs `b-tool`, and no directory on this host's search path holds it",
            "no provider offers this capability, so nothing can answer it",
        ]
    );
}

/// A capability whose strongest offer is undetermined is *not* available, and a capability
/// whose weakest offer is undetermined still is -- the verdict is the best of the rows under
/// it, which is what makes the two levels say different things.
///
/// This is the falsifier for `CapabilityStanding::Summary`: replace the maximum with the
/// first offer and the first assertion goes red, replace it with the minimum and the second
/// does.
#[test]
fn Test_A_Verdict_Should_Be_The_Best_Of_Its_Offers_Rather_Than_The_First()
{
    let undetermined = ProviderStanding::Undetermined { because: "not settled".to_owned() };

    let weakest_first = Standing("x", &[("x.slow", undetermined.clone()), ("x.fast", ProviderStanding::InThisBinary)]);
    let strongest_first = Standing("y", &[("y.fast", ProviderStanding::InThisBinary), ("y.slow", undetermined)]);
    let none = Standing("z", &[]);

    assert_eq!(weakest_first.Summary(), ProviderStanding::InThisBinary);
    assert_eq!(strongest_first.Summary(), ProviderStanding::InThisBinary);
    assert_eq!(none.Summary(), ProviderStanding::NothingOffered);
}

/// The starter file lands where a root has none, and the same call over the same root then
/// refuses -- naming the path rather than replacing what is there.
#[test]
fn Test_The_Starter_Policy_Should_Be_Written_Once_And_Refused_After()
{
    let root = Fresh_Root("nomos-cli-profile-starter-written-once");

    let first = starter_policy::Write_Starter_Gate_Policy(&root, &FILE_SYSTEM);
    let written = std::fs::read_to_string(root.join(GATE_POLICY_FILE)).expect("the first call reported writing it");
    let second = starter_policy::Write_Starter_Gate_Policy(&root, &FILE_SYSTEM);
    let after = std::fs::read_to_string(root.join(GATE_POLICY_FILE)).expect("the refusal leaves the file where it was");

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(first, StarterOutcome::Written { path: root.join(GATE_POLICY_FILE) });
    assert_eq!(second, StarterOutcome::AlreadyDeclared { path: root.join(GATE_POLICY_FILE) });
    assert_eq!(written, after, "a refusal must leave the existing declaration byte for byte");
}

/// A declaration somebody wrote survives the starter being asked for -- asserted over
/// contents this test authored, so "unchanged" means unchanged rather than "the starter file
/// happens to be what a starter file writes".
#[test]
fn Test_An_Existing_Declaration_Should_Survive_Being_Asked_For_A_Starter()
{
    let root = Fresh_Root("nomos-cli-profile-existing-declaration");
    let declared = "{\n  \"coverage\": \"require-completeness\"\n}\n";
    Write_Fixture(&root.join(GATE_POLICY_FILE), declared);

    let outcome = starter_policy::Write_Starter_Gate_Policy(&root, &FILE_SYSTEM);
    let after = std::fs::read_to_string(root.join(GATE_POLICY_FILE)).expect("nothing was written, so the file is still there");

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(outcome, StarterOutcome::AlreadyDeclared { path: root.join(GATE_POLICY_FILE) });
    assert_eq!(after, declared);
}

/// Asking for a starter over a root that has one is `ExitCode::Refused`, and the profile is
/// still rendered: what a person needs to see is that a declaration is already there, which
/// the policy-file section said one moment earlier.
#[test]
fn Test_Asking_For_A_Starter_Over_An_Existing_One_Should_Refuse_And_Still_Profile()
{
    let root = Fresh_Root("nomos-cli-profile-starter-refused-end-to-end");
    Write_Fixture(&root.join(GATE_POLICY_FILE), "{}\n");
    let command = ProfileCommand { root: root.clone(), write_gate_policy: true };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&command, &mut stdout, &mut stderr);

    let _ignored = std::fs::remove_dir_all(&root);
    let rendered = String::from_utf8(stdout).expect("the renderer writes text");
    assert_eq!(code, ExitCode::Refused);
    assert!(Section(&rendered, POLICY_FILES_HEADING).iter().any(|row| return row.starts_with(GATE_POLICY_FILE)), "{rendered}");
}

/// The starter file is accepted by the reader that actually resolves it, spelling and all.
///
/// `Run_Gate` resolves `nomos-gate.json` before it judges anything, so an empty walk is
/// enough to exercise the read without running a single rule. A key this reader does not
/// know is refused rather than ignored (`deny_unknown_fields`), so this is what checks the
/// starter's six keys are the six keys -- not the spelling in `starter_policy.rs`.
#[test]
fn Test_The_Starter_Policy_Should_Be_Accepted_By_The_Real_Gate_Reader()
{
    let root = Fresh_Root("nomos-cli-profile-starter-accepted");
    let written = starter_policy::Write_Starter_Gate_Policy(&root, &FILE_SYSTEM);

    let verdict = Gate_Verdict_Over(&root);

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(written, StarterOutcome::Written { path: root.join(GATE_POLICY_FILE) });
    assert_eq!(verdict, None, "the starter policy must resolve, not refuse");
}

/// The falsifier for the test above: a policy file this reader cannot accept produces a
/// refusal, so `None` there means the starter was read rather than that this assertion
/// cannot fail.
#[test]
fn Test_A_Policy_File_The_Reader_Refuses_Should_Withhold_A_Verdict()
{
    let root = Fresh_Root("nomos-cli-profile-starter-falsifier");
    Write_Fixture(&root.join(GATE_POLICY_FILE), "{\n  \"suppresions\": []\n}\n");

    let verdict = Gate_Verdict_Over(&root);

    let _ignored = std::fs::remove_dir_all(&root);
    assert!(verdict.is_some(), "a misspelled key must be refused rather than ignored");
}

/// What `nomos_gate_orchestration` says about the policy file under `root`, over a walk of
/// nothing -- the policy is resolved before anything is judged, so no rule runs and no
/// provider is launched.
fn Gate_Verdict_Over(root: &Path) -> Option<nomos_gate_orchestration::NoVerdict>
{
    use nomos_composer_std::{ENVIRONMENT, LAUNCHER};

    let now = Timestamp::From_Unix_Seconds(0);
    let command = nomos_gate_orchestration::GateCommand { root: root.to_path_buf(), ..Default::default() };
    let environment = nomos_gate_orchestration::GateEnvironment {
        variant: nomos_workspace::BuildVariant::New("test", "test", "test", std::iter::empty::<String>()),
        launcher: &LAUNCHER,
        filesystem: &FILE_SYSTEM,
        environment: &ENVIRONMENT,
        now,
    };

    return nomos_gate_orchestration::Run_Gate(Some(Vec::new()), environment, &command, nomos_gate_orchestration::Fresh_Run_Id(now))
        .no_verdict;
}

/// One capability's standing, built by hand for the rendering tests.
fn Standing(capability: &str, offers: &[(&str, ProviderStanding)]) -> CapabilityStanding
{
    return CapabilityStanding {
        capability: capability.to_owned(),
        offers: offers
            .iter()
            .map(|(provider, standing)| return OfferStanding { provider: (*provider).to_owned(), standing: standing.clone() })
            .collect(),
    };
}

/// Three capabilities covering every standing a row can carry, including one nothing offers.
fn Hand_Built_Standings() -> Vec<CapabilityStanding>
{
    return vec![
        Standing(
            "a.capability",
            &[
                ("a.linked", ProviderStanding::InThisBinary),
                ("a.unsettled", ProviderStanding::Undetermined { because: "a-reason".to_owned() }),
            ],
        ),
        Standing("b.capability", &[("b.toolless", ProviderStanding::ToolMissing { tool: "b-tool".to_owned() })]),
        Standing("c.capability", &[]),
    ];
}

/// The capability verdict lines: the ones indented one level.
fn Verdict_Lines(rendered: &str) -> Vec<String>
{
    return Section(rendered, CAPABILITIES_HEADING)
        .into_iter()
        .filter(|line| return !line.starts_with(OFFER_INDENT))
        .map(|line| return line.trim().to_owned())
        .collect();
}

/// The provider rows: the ones indented two levels.
fn Offer_Lines(rendered: &str) -> Vec<String>
{
    return Section(rendered, CAPABILITIES_HEADING)
        .into_iter()
        .filter(|line| return line.starts_with(OFFER_INDENT))
        .map(|line| return line.trim().to_owned())
        .collect();
}

/// The profile half of the report over `root`, as text.
fn Rendered_Profile(root: &Path) -> String
{
    let command = ProfileCommand { root: root.to_path_buf(), write_gate_policy: false };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Run(&command, &mut stdout, &mut stderr);

    assert_eq!(code, ExitCode::Ok, "{}", String::from_utf8_lossy(&stderr));
    return String::from_utf8(stdout).expect("the renderer writes text");
}

/// The manifest rows the profile carries, as this report renders them.
fn Manifest_Rows(profile: &WorkspaceProfile) -> Vec<String>
{
    return profile
        .manifests
        .iter()
        .map(|manifest| {
            let presence = if manifest.is_present { "present" } else { "absent" };
            return format!("{}: {presence} ({})", manifest.name, manifest.extension);
        })
        .collect();
}

/// The policy-file rows the profile carries, by name, so a dropped row is a length mismatch
/// rather than a missing sentence.
fn Policy_Rows(profile: &WorkspaceProfile) -> Vec<String>
{
    return profile
        .policy_files
        .iter()
        .map(|policy_file| {
            let presence = if policy_file.name == "standards.json" { "present" } else { "absent" };
            return format!("{}: {presence}", policy_file.name);
        })
        .collect();
}

/// The indented entries rendered under `heading`, with their indent removed.
///
/// The report's structure is a heading at column zero followed by its entries at an indent,
/// so reading it back this way asserts the shape rather than the wording. A heading nothing
/// matches yields an empty list, which fails the length assertions above rather than passing
/// them silently.
fn Section(rendered: &str, heading: &str) -> Vec<String>
{
    return rendered
        .lines()
        .skip_while(|line| return line.trim_end() != heading)
        .skip(1)
        .take_while(|line| return line.starts_with(SECTION_INDENT))
        .filter_map(|line| return line.strip_prefix(SECTION_INDENT))
        .map(|line| return line.trim_end().to_owned())
        .collect();
}

/// A command line, split the way a shell would have.
fn Arguments(text: &str) -> Vec<String>
{
    return text.split_whitespace().map(str::to_owned).collect();
}

/// Removes and recreates a directory under the system temp directory, so a test starts from
/// a clean tree regardless of what an earlier run left behind.
fn Fresh_Root(name: &str) -> PathBuf
{
    let root = std::env::temp_dir().join(name);
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the temporary root is creatable");
    return root;
}

/// Writes a fixture file, failing loudly rather than leaving a test to assert over a tree
/// that was never built.
fn Write_Fixture(path: &Path, contents: &str)
{
    std::fs::write(path, contents).expect("the fixture's own root was created just above");
}

/// A host whose environment is exactly the pairs given.
fn Fixed_Host(variables: &[(&str, &str)]) -> FixedEnvironment
{
    return FixedEnvironment {
        variables: variables
            .iter()
            .map(|&(name, value)| return (name.to_owned(), OsString::from(value)))
            .collect(),
    };
}

/// An [`Environment`] that answers from a fixed list rather than from this process.
///
/// The whole availability answer turns on what `PATH` and `CARGO` say, so a test reading
/// this machine's own would assert something different on every host -- green here, red on a
/// runner with no `cargo`, and proving nothing either way.
struct FixedEnvironment
{
    variables: Vec<(String, OsString)>,
}

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for FixedEnvironment
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl Environment for FixedEnvironment
{
    fn Variable(&self, name: &str) -> Option<OsString>
    {
        return self
            .variables
            .iter()
            .find(|(declared, _)| return declared == name)
            .map(|(_, value)| return value.clone());
    }

    fn Working_Directory(&self) -> Result<PathBuf, EnvironmentError>
    {
        unimplemented!("no availability question reaches the working directory")
    }
}

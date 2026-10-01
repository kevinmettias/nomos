//! `Cargo.toml`'s `rust-version` and the toolchain the gate's `Compatibility floor` lane
//! installs are one fact about this workspace, typed twice.
//!
//! `OD-GATE-008` measured two of the three ways the two files can drift and both are
//! caught without help from this file: code that outruns the declaration fails to
//! compile on the floor (the `E0658` let-chain failure that produced the evidence for
//! `1.88`), and a declaration raised above the pinned toolchain is refused by `cargo`
//! itself before anything compiles (`rustc 1.88.0 is not supported by the following
//! packages`, measured against a declared `1.90`).
//!
//! The third direction has nothing watching it. Lower `rust-version` below what the
//! workspace needs and the lane still installs the same, higher toolchain, still compiles
//! clean, and still reports green — the false low claim survives untouched, because
//! `RUSTUP_TOOLCHAIN` in `.github/workflows/gate.yml` names the compiler the lane actually
//! runs on and nothing compares it against the number `Cargo.toml` promises a consumer.
//!
//! This reads both files as text and compares them, the shape `tests/contract/tests/boundaries`
//! already uses for a README table checked both ways. The toolchain the gate installs is
//! read through [`nomos_ledger::Derive_Step`] rather than re-parsed here, because a second
//! `run:` line reader is a second place for the two to disagree about what a step means.

use nomos_contract_tests::Workspace;

/// The step that installs the compiler the `Compatibility floor` lane compiles under.
const INSTALL_STEP: &str = "Install compatibility floor toolchain";

/// `Cargo.toml`'s declared floor must be satisfied by the toolchain the gate installs to
/// check it, or a lowered declaration would compile clean on a compiler that was never
/// asked to prove it and report green regardless.
#[test]
fn Test_The_Declared_Floor_Should_Match_The_Toolchain_The_Gate_Installs()
{
    let root = Workspace::Workspace_Root();
    let cargo_toml = std::fs::read_to_string(root.join("Cargo.toml"))
        .expect("the workspace root must have a Cargo.toml");
    let workflow = std::fs::read_to_string(nomos_ledger::Workflow_Path(&root))
        .expect("the workspace root must have .github/workflows/gate.yml");

    let declared = Declared_Rust_Version(&cargo_toml)
        .expect("Cargo.toml must declare rust-version under [workspace.package]");
    let installed = Installed_Floor_Toolchain(&workflow);

    assert!(
        Toolchain_Satisfies_Declared_Floor(&declared, &installed),
        "Cargo.toml declares rust-version = \"{declared}\" but the gate's `{INSTALL_STEP}` \
         step installs toolchain {installed}. A declared floor the installed toolchain does \
         not match is unchecked either way it drifts: lower than the toolchain and the lane \
         proves nothing about the number it claims to; higher and cargo would already refuse, \
         which is not the case being measured here."
    );
}

/// The control this item's `done_when` asks for: the comparison has to actually fail when
/// the two numbers stop agreeing, not merely hold today by coincidence.
#[test]
fn Test_A_Lowered_Declaration_Should_Fail_The_Comparison()
{
    assert!(
        !Toolchain_Satisfies_Declared_Floor("1.86", "1.88.0"),
        "a declared floor of 1.86 against an installed toolchain of 1.88.0 is exactly the \
         unguarded direction this item exists to close, and must not compare equal"
    );
}

/// The declaration and the patch release it names both pass, which is the shape the real
/// files are in today.
#[test]
fn Test_A_Matching_Declaration_Should_Pass_The_Comparison()
{
    assert!(Toolchain_Satisfies_Declared_Floor("1.88", "1.88.0"));
    assert!(Toolchain_Satisfies_Declared_Floor("1.88.0", "1.88.0"));
}

/// A declaration naming a different minor version must not pass by loose prefix matching —
/// `1.8` is not a prefix match for `1.88.0` under this comparison, but the two are unrelated
/// floors and a naive `starts_with` on the raw strings would conflate them.
#[test]
fn Test_An_Unrelated_Minor_Version_Should_Fail_The_Comparison()
{
    assert!(!Toolchain_Satisfies_Declared_Floor("1.8", "1.88.0"));
}

/// The real workflow has to still name the step this file reads, or the comparison above
/// is quietly checking nothing.
#[test]
fn Test_The_Real_Workflow_Should_Still_Have_The_Install_Step()
{
    let root = Workspace::Workspace_Root();
    let workflow = std::fs::read_to_string(nomos_ledger::Workflow_Path(&root))
        .expect("the workspace root must have .github/workflows/gate.yml");

    let argv = nomos_ledger::Derive_Step(&workflow, INSTALL_STEP)
        .unwrap_or_else(|refusal| panic!("{}", refusal.Describe()));

    assert!(
        argv.contains(&"install".to_owned()),
        "the `{INSTALL_STEP}` step no longer runs `rustup ... install`, so the version this \
         file reads out of it is no longer the toolchain the lane compiles under: {argv:?}"
    );
}

/// `Cargo.toml`'s `rust-version`, read from the line that declares it under
/// `[workspace.package]`.
///
/// Deliberately not a TOML parser. `Cargo.toml` has exactly one line starting with
/// `rust-version`, and a full parser would be more code trusted to answer a question one
/// line already answers.
fn Declared_Rust_Version(cargo_toml: &str) -> Option<String>
{
    let line = cargo_toml
        .lines()
        .map(str::trim)
        .find(|line| return line.starts_with("rust-version"))?;
    let (_, value) = line.split_once('=')?;

    return Some(value.trim().trim_matches('"').to_owned());
}

/// The toolchain version the gate's `Compatibility floor` lane installs before compiling,
/// read from the `Install compatibility floor toolchain` step's own argv.
///
/// # Panics
///
/// Panics if the step cannot be derived, or if its argv does not run `rustup ... install
/// <version>` — both mean the version this function would report is not the version the
/// lane actually compiles under, and a wrong answer here is worse than a loud one.
fn Installed_Floor_Toolchain(workflow: &str) -> String
{
    let argv = nomos_ledger::Derive_Step(workflow, INSTALL_STEP)
        .unwrap_or_else(|refusal| panic!("{}", refusal.Describe()));

    let after_install = argv
        .iter()
        .position(|argument| return argument == "install")
        .and_then(|position| return position.checked_add(1))
        .unwrap_or_else(|| {
            panic!("the `{INSTALL_STEP}` step does not run `rustup ... install`: {argv:?}")
        });

    return argv
        .get(after_install)
        .unwrap_or_else(|| panic!("`install` in the `{INSTALL_STEP}` step names no version: {argv:?}"))
        .clone();
}

/// Whether an installed toolchain proves the declared floor rather than merely coexisting
/// with it.
///
/// A declared floor of `1.88` is satisfied by an installed `1.88.0` because `1.88.0` is the
/// patch release `1.88` names. It is not satisfied by `1.880` or by an unrelated minor such
/// as `1.8`, which is why this compares against `{declared}.` as a whole dotted segment
/// rather than testing the raw strings for a shared prefix.
fn Toolchain_Satisfies_Declared_Floor(declared: &str, installed: &str) -> bool
{
    return installed == declared || installed.starts_with(&format!("{declared}."));
}

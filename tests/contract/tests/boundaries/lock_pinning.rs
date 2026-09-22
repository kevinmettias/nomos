//! The committed lock still pins the XVPE crossing.
//!
//! `D-130`'s surviving clause requires this crossing be adopted by git reference and
//! commit. The manifests say so, and `Cargo.lock` is what turns that from a statement into
//! a recorded fact: it names the revision every build resolved. `OD-PLATFORM-004` permits a
//! local substitution of that crossing for sibling development, denies it any authority,
//! and names the debt this module pays.
//!
//! # Where the revision reading lives now
//!
//! In `nomos_contract_tests::crossing`, with `crossing_cold`'s copy of it. Until
//! 2026-09-21 each target scanned manifests itself, and `e0197368` moved the declaration
//! into the root table and taught only this one to follow. The other kept scanning a member
//! manifest for a `rev = "` that inheritance no longer spells, and panicked rather than
//! failing, in a test the gate reaches only with `--ignored`. One authority, named outside
//! both, is `OD-GATE-011`'s remedy; the constructed-disagreement control moved there with it.
//!
//! # Why a lock needs its own guard
//!
//! A sibling-development build rewrites `Cargo.lock`, deleting the `source` line from all
//! twelve `xvpe-` packages, and reports nothing. `Cargo.lock` is tracked on purpose, so
//! `.gitignore` cannot protect it the way it protects the override file, and the scoped
//! `git add` `AGENTS.md` asks for everywhere else is the gesture that would publish the
//! damage.
//!
//! # Why this reads the committed lock and not the file on disk
//!
//! Because cargo repairs the file on disk before this test can look at it. Measured
//! 2026-09-14: an opt-in build left the working tree at zero of twelve packages pinned, and
//! a single ordinary `cargo metadata` call restored all twelve. `cargo test` is an ordinary
//! cargo command, so it re-resolves and re-pins the lock on its way to running this module.
//! A version of this guard that read the working tree passed over a lock that had been
//! un-pinned seconds earlier, because by the time it opened the file the damage had been
//! undone underneath it.
//!
//! The earlier version appeared to work only because the override was auto-discovered at
//! that moment: `cargo test` kept the patch applied, so the lock stayed un-pinned long
//! enough to be caught. `OD-PLATFORM-004`'s amendment made substitution opt-in, which
//! removed the one condition under which that guard could fail at all.
//!
//! The claim is about what a commit publishes, so the committed blob is the right artifact
//! and the working tree is not. A dirty lock in somebody's tree is not yet a defect; the
//! same lock committed is.
//!
//! # Why the judging is a function over text
//!
//! So it can be made to fail on demand. A guard believed because it passes on a tree that
//! already satisfies it has proved nothing, which is the failure mode this repository's own
//! records are about. [`Unpinned_Among`] takes lock text, so
//! [`Test_An_Unpinned_Lock_Should_Be_Rejected`] can hand it a lock that violates the rule
//! and observe the rejection, with no build and no repository state involved.
//!
//! # Why the root manifest is read as well
//!
//! Because that is where the crossing is declared. Until 2026-09-21 each of the seven
//! members that name an `xvpe-` crate spelled the git reference and the revision in full,
//! fifteen times over, and a bump was fifteen edits that had to agree. They are declared
//! once in `[workspace.dependencies]` now and inherited with `workspace = true`, so a
//! reader of member manifests alone would find no revision at all and this module would
//! fail for the opposite of its own reason.
//!
//! Every member is still read, because inheriting is a convention and not a constraint: a
//! member may spell its own `git` and `rev` again at any time, and that is exactly the
//! second snapshot of one engine the assertion below exists to refuse.
//!
//! # What this cost, stated as the class it belongs to
//!
//! `OD-COMPLETENESS-001`: a guard must say what universe it quantifies over, and a guard
//! whose universe is derived owes nothing further. This one quantified over workspace
//! members while claiming something about every manifest, and the two differed by one --
//! the root, which is a virtual manifest and therefore not a member. That difference was
//! invisible for as long as members did the pinning, and it was still invisible when seven
//! of the eight moved, because `tests/contract` is itself a member and its own surviving
//! pin went on answering. The guard only went quiet once the last one moved. A universe
//! narrower than the claim does not fail; it stops noticing, in exactly the case it was
//! built for.
//!
//! # Why the revision is not written down here
//!
//! It is read from the manifests, which is the half a substitution does not touch. A
//! legitimate revision bump edits manifests and lock together and this module follows it; a
//! hardcoded revision would fail on every bump and teach the next reader to edit the test
//! rather than look.

use nomos_contract_tests::{Declared_Revisions, Repository_Root, Revisions_In, XVPE_PREFIX};

/// `Cargo.lock` as the current commit publishes it.
///
/// # Panics
///
/// Panics if git cannot produce it. A guard that cannot see its subject must fail loudly:
/// falling back to the working tree would silently restore the defect this module exists to
/// remove, because that file is the one cargo repairs.
fn Committed_Lock_Text() -> String
{
    let output = std::process::Command::new("git")
        .args(["show", "HEAD:Cargo.lock"])
        .current_dir(Repository_Root())
        .output()
        .unwrap_or_else(|error| panic!("cannot run git to read the committed lock: {error}"));

    assert!(
        output.status.success(),
        "git could not read HEAD:Cargo.lock ({}).\n\
         This guard judges the committed lock rather than the working-tree file, because \
         cargo re-pins the latter before this test runs. Without git there is nothing \
         honest to check, so this fails rather than reporting a pass it did not earn.",
        String::from_utf8_lossy(&output.stderr).trim()
    );

    return String::from_utf8_lossy(&output.stdout).into_owned();
}

/// Every `xvpe-` package in `lock`, paired with its `source` value if it has one.
fn Crossing_Packages_In(lock: &str) -> Vec<(String, Option<String>)>
{
    const NAME_KEY: &str = "name = \"";
    const SOURCE_KEY: &str = "source = \"";

    let mut found = Vec::new();

    for block in lock.split("[[package]]")
    {
        let mut name: Option<String> = None;
        let mut source: Option<String> = None;

        for line in block.lines()
        {
            let trimmed = line.trim();

            if let Some(rest) = trimmed.strip_prefix(NAME_KEY)
            {
                name = rest.strip_suffix('\"').map(str::to_string);
            }
            else if let Some(rest) = trimmed.strip_prefix(SOURCE_KEY)
            {
                source = rest.strip_suffix('\"').map(str::to_string);
            }
        }

        if let Some(name) = name
        {
            if name.starts_with(XVPE_PREFIX)
            {
                found.push((name, source));
            }
        }
    }

    return found;
}

/// The crossing packages in `lock` that are not pinned to `revision`, as reportable lines.
///
/// A function over text rather than over the repository, so a negative control can make it
/// fail without a build. See this module's own documentation for why that matters.
fn Unpinned_Among(lock: &str, revision: &str) -> Vec<String>
{
    let mut unpinned = Vec::new();

    for (name, source) in Crossing_Packages_In(lock)
    {
        let pinned = source
            .as_deref()
            .is_some_and(|source| source.starts_with("git+") && source.contains(revision));

        if !pinned
        {
            let seen = source.as_deref().unwrap_or("no source, so a local path").to_string();
            unpinned.push(format!("{name} -> {seen}"));
        }
    }

    unpinned.sort();

    return unpinned;
}

/// The manifests agree on one revision for the crossing.
///
/// Two revisions would mean one workspace adopting two snapshots of one engine, which is a
/// defect on its own and would also leave the comparison below without a single answer to
/// check against.
#[test]
fn Test_Every_Manifest_Should_Pin_The_Crossing_To_One_Revision()
{
    let declared = Declared_Revisions();

    assert!(
        !declared.is_empty(),
        "no manifest pins the XVPE crossing to a git revision.\n\
         Either this workspace stopped depending on XVPE -- in which case this module and \
         D-130's surviving clause both need revisiting -- or a dependency was respelled as \
         a path, which is the form D-130 refuses."
    );

    assert!(
        declared.len() == 1,
        "the XVPE crossing is pinned to {} different revisions: {declared:?}.\n\
         One workspace adopting two snapshots of one engine is a defect by itself, and it \
         leaves no single revision for the committed lock to be checked against.",
        declared.len()
    );
}

/// The judging rejects a lock that violates it.
///
/// The negative control. Without it the assertion below is only known to pass on a tree
/// that already satisfies it, which is indistinguishable from an assertion that cannot
/// fail -- and this guard spent one commit in exactly that state.
#[test]
fn Test_An_Unpinned_Lock_Should_Be_Rejected()
{
    const REVISION: &str = "82a3c8fccf4ef7f3759f36d3f320a91d0f96341c";

    let pinned = "\
[[package]]\n\
name = \"xvpe-primitives\"\n\
version = \"0.1.0\"\n\
source = \"git+https://github.com/kevinmettias/xvpe.git?rev=82a3c8fccf4ef7f3759f36d3f320a91d0f96341c#82a3c8fccf4ef7f3759f36d3f320a91d0f96341c\"\n";

    let substituted = "\
[[package]]\n\
name = \"xvpe-primitives\"\n\
version = \"0.1.0\"\n\
dependencies = [\n\
]\n";

    let stale = "\
[[package]]\n\
name = \"xvpe-primitives\"\n\
version = \"0.1.0\"\n\
source = \"git+https://github.com/kevinmettias/xvpe.git?rev=0000000000000000000000000000000000000000\"\n";

    assert!(
        Unpinned_Among(pinned, REVISION).is_empty(),
        "a correctly pinned lock was reported as unpinned; the judging is wrong in the \
         direction that would make every commit fail."
    );

    assert!(
        !Unpinned_Among(substituted, REVISION).is_empty(),
        "a lock with the source line removed -- exactly what a sibling-development build \
         writes -- was accepted. This assertion cannot fail, and a guard that cannot fail \
         is worse than no guard."
    );

    assert!(
        !Unpinned_Among(stale, REVISION).is_empty(),
        "a lock pinned to a different revision than the manifests name was accepted. The \
         crossing would be adopted from one commit and described as another."
    );
}

/// The committed lock names the declared revision for every crate of the crossing.
#[test]
fn Test_The_Committed_Lock_Should_Pin_Every_Crossing_Package_To_That_Revision()
{
    let declared = Declared_Revisions();
    let Some(revision) = declared.iter().next()
    else
    {
        panic!(
            "no declared revision. \
             Test_Every_Manifest_Should_Pin_The_Crossing_To_One_Revision says why."
        );
    };

    let lock = Committed_Lock_Text();
    let packages = Crossing_Packages_In(&lock);

    assert!(
        !packages.is_empty(),
        "the committed Cargo.lock names no {XVPE_PREFIX} package at all.\n\
         This assertion would pass over an empty set while proving nothing. If the crossing \
         really is gone, remove this module rather than let it report a clean result over \
         nothing."
    );

    let unpinned = Unpinned_Among(&lock, revision);

    assert!(
        unpinned.is_empty(),
        "{} of {} packages in the XVPE crossing are not pinned to {revision} in the \
         committed lock:\n  {}\n\n\
         A package with no source at all resolves from a local path. The cause is a \
         sibling-development build, which rewrites Cargo.lock and reports nothing. Since \
         OD-PLATFORM-004's amendment that substitution is opt-in -- \
         `cargo --config .cargo/xvpe-local.toml ...` -- so an ordinary command cannot have \
         produced it. A lock from such a build was committed.\n\n\
         To restore, from a clean working tree:\n  git checkout -- Cargo.lock\n\
         then commit the restored lock, because this reads what HEAD publishes rather than \
         what is on disk.",
        unpinned.len(),
        packages.len(),
        unpinned.join("\n  ")
    );
}

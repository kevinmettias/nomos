//! The committed lock still pins the XVPE crossing.
//!
//! `D-130`'s surviving clause requires this crossing be adopted by git reference and
//! commit. The manifests say so, and `Cargo.lock` is what turns that from a statement into
//! a recorded fact: it names the revision every build resolved. `OD-PLATFORM-004` permits a
//! local substitution of that crossing for sibling development, denies it any authority,
//! and names the debt this module pays.
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

use crate::bands::Repository_Root;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// The repository the XVPE crossing is adopted from.
const XVPE_GIT_URL: &str = "https://github.com/kevinmettias/xvpe.git";

/// The prefix naming a package that belongs to that crossing.
const XVPE_PREFIX: &str = "xvpe-";

/// The value of `rev = "..."` in `line`, if it carries one.
fn Revision_In(line: &str) -> Option<String>
{
    const KEY: &str = "rev = \"";

    let from = line.find(KEY)?.checked_add(KEY.len())?;
    let rest = line.get(from..)?;
    let to = rest.find('\"')?;

    return Some(rest.get(..to)?.to_string());
}

/// Every revision `text` pins the crossing to.
///
/// Over text rather than over a path, so that the multi-revision case below can be
/// exhibited on a constructed manifest: a tree that satisfies the rule cannot demonstrate
/// that the rule has teeth.
fn Revisions_In(text: &str) -> BTreeSet<String>
{
    let mut found = BTreeSet::new();

    for line in text.lines()
    {
        // A guard clause rather than a nested `if`: the two conditions one inside the
        // other put this at four levels of control flow, which `nesting-depth` reports,
        // and flattening with an early `continue` is the first remedy it names.
        if !line.contains(XVPE_GIT_URL)
        {
            continue;
        }

        if let Some(revision) = Revision_In(line)
        {
            found.insert(revision);
        }
    }

    return found;
}

/// Every manifest that can pin the crossing.
///
/// The workspace root first, because `[workspace.dependencies]` is where the crossing is
/// declared; then every member, because a member that spells its own `git` and `rev`
/// instead of inheriting is precisely what a single declaring site stops being able to
/// prevent.
fn Pinning_Manifests() -> Vec<PathBuf>
{
    use nomos_contract_tests::Workspace;

    let mut manifests = vec![Repository_Root().join("Cargo.toml")];

    for member in Workspace::Load().Members()
    {
        manifests.push(member.root.join("Cargo.toml"));
    }

    return manifests;
}

/// Every revision this workspace's own manifests pin the crossing to.
fn Declared_Revisions() -> BTreeSet<String>
{
    let mut found = BTreeSet::new();

    for manifest in Pinning_Manifests()
    {
        let Ok(text) = std::fs::read_to_string(&manifest)
        else
        {
            continue;
        };

        found.extend(Revisions_In(&text));
    }

    return found;
}

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

/// A member that re-pins the crossing instead of inheriting it is seen as a second
/// revision.
///
/// The negative control for the assertion above, and the one the single declaring site
/// made necessary: with the revision written once in the root table and inherited
/// everywhere else, the real tree can no longer exhibit disagreement, so a guard believed
/// because it passes over that tree would be believed for no reason. The disagreement is
/// constructed here instead, in the two manifests it would really live in.
#[test]
fn Test_A_Member_Re_Pinning_The_Crossing_Should_Be_Seen_As_A_Second_Revision()
{
    let root = "[workspace.dependencies]\n\
                xvpe-primitives = { git = \"https://github.com/kevinmettias/xvpe.git\", rev = \"82a3c8fccf4ef7f3759f36d3f320a91d0f96341c\" }\n";
    let inheriting_member = "[dependencies]\nxvpe-primitives = { workspace = true }\n";
    let re_pinning_member = "[dependencies]\n\
                             xvpe-primitives = { git = \"https://github.com/kevinmettias/xvpe.git\", rev = \"0000000000000000000000000000000000000000\" }\n";

    let mut inherited = Revisions_In(root);
    inherited.extend(Revisions_In(inheriting_member));

    assert!(
        inherited.len() == 1,
        "a member inheriting the crossing added a revision of its own: {inherited:?}. \
         Inheritance carries no `rev` of its own, so the declaring site must be the only \
         answer."
    );

    let mut disagreeing = Revisions_In(root);
    disagreeing.extend(Revisions_In(re_pinning_member));

    assert!(
        disagreeing.len() == 2,
        "a member that re-pinned the crossing to its own revision was not seen: \
         {disagreeing:?}. The assertion above would then pass over a workspace holding two \
         snapshots of one engine, which is the whole thing it refuses."
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

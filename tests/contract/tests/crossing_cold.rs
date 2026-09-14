//! The XVPE crossing really builds from the pinned revision, proved by a build that cannot
//! have inherited its answer.
//!
//! `D-130`'s surviving clause requires this crossing be adopted by git reference and commit,
//! and `boundaries/lock_pinning.rs` asserts the committed lock still says so. That is a claim
//! about a file. This is the other half: that the thing the file describes actually compiles.
//!
//! # Why this one has to be cold
//!
//! Because a warm one answered wrongly, and did so while exiting zero. Measured 2026-09-14:
//! `cargo check --workspace --all-targets`, run precisely to establish that this workspace
//! builds against pinned XVPE, finished in twenty seconds having compiled no `xvpe-` crate at
//! all -- a concurrent session had already warmed the shared target directory. The identical
//! command with an isolated `CARGO_TARGET_DIR` compiled ten of them from the pinned revision by
//! name. The exit code was evidence that nothing failed to typecheck against artifacts somebody
//! else produced, possibly from other sources, which is not the claim it was read as.
//!
//! `OD-GATE-028` names that defect class: the verification mechanism modified or bypassed the
//! condition it existed to observe. This module is the response for this one crossing.
//!
//! # What it does not do
//!
//! It does not make the gate cold. A cold workspace build would cost the gate many minutes to
//! re-derive what an incremental build already establishes for every other claim, and nothing
//! about most of those claims depends on the target directory being empty. Only this one does,
//! so only this one pays. It checks a single crate -- the platform adapter, which is the
//! narrowest target that must reach XVPE -- and asserts from the build's own output that the
//! crossing was really traversed, rather than inferring it from an exit code.
//!
//! # Why it is `#[ignore]`
//!
//! So an ordinary `cargo test -p nomos-contract-tests` does not pay for a cold build. The gate
//! runs it as its own step with `--ignored`, which is the one place the cost is worth paying.

use std::path::{Path, PathBuf};

/// The crate whose compilation must reach XVPE.
const SUBJECT: &str = "nomos-platform-xvpe";

/// The manifest whose pinned revision the build is checked against.
const PINNED_MANIFEST: &str = "crates/platform/nomos-platform-xvpe/Cargo.toml";

/// The workspace root, from this crate's manifest directory.
fn Repository_Root() -> PathBuf
{
    return Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
}

/// The revision `PINNED_MANIFEST` adopts the crossing at.
///
/// Read rather than written down, so a legitimate bump moves this with the manifest instead of
/// failing and teaching the next reader to edit the test.
fn Declared_Revision() -> String
{
    const KEY: &str = "rev = \"";

    let path = Repository_Root().join(PINNED_MANIFEST);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

    for line in text.lines()
    {
        if !line.contains("xvpe")
        {
            continue;
        }

        let Some(offset) = line.find(KEY)
        else
        {
            continue;
        };

        let Some(rest) = line.get(offset.saturating_add(KEY.len())..)
        else
        {
            continue;
        };

        if let Some(end) = rest.find('"')
        {
            if let Some(revision) = rest.get(..end)
            {
                return revision.to_string();
            }
        }
    }

    panic!(
        "{PINNED_MANIFEST} names no pinned revision for the XVPE crossing. Either the crossing \
         was respelled as a path -- the form D-130 refuses -- or this workspace stopped \
         depending on XVPE, and either way this module's premise no longer holds."
    );
}

/// What a build log says about the crossing: how many xvpe crates came from `revision`, and
/// which ones came from anywhere else.
///
/// A function over text, because the assertion below is otherwise only ever exercised against a
/// tree that already satisfies it, which is indistinguishable from an assertion that cannot
/// fail. `OD-GATE-028` records why that distinction is not optional, and
/// [`Test_A_Log_That_Misses_The_Crossing_Should_Be_Rejected`] is the control this shape buys.
///
/// Note that the spawned build does not inherit the flags of whatever `cargo` invoked this test:
/// an outer `cargo --config .cargo/xvpe-local.toml test ...` leaves the inner `cargo check`
/// resolving the pinned source. That immunity is deliberate and worth keeping -- the check
/// cannot be subverted by the environment that runs it -- but it does mean the real build is not
/// a way to make this fail, and the control below is.
fn Verdict(log: &str, revision: &str) -> (usize, Vec<String>)
{
    let building: Vec<&str> = log
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("Checking xvpe-")
                || trimmed.starts_with("Compiling xvpe-")
                || trimmed.starts_with("Fresh xvpe-")
        })
        .collect();

    let pinned = building.iter().filter(|line| line.contains(revision)).count();

    let unpinned: Vec<String> = building
        .iter()
        .filter(|line| !line.contains(revision))
        .map(|line| (*line).trim().to_string())
        .collect();

    return (pinned, unpinned);
}

/// A target directory that has never been built in.
fn Empty_Target_Directory() -> PathBuf
{
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();

    let directory = std::env::temp_dir().join(format!("nomos-crossing-cold-{unique}"));

    std::fs::create_dir_all(&directory)
        .unwrap_or_else(|error| panic!("cannot create {}: {error}", directory.display()));

    return directory;
}

/// The crossing is traversed by a build that cannot have inherited it.
///
/// Ignored by default: this is a cold build and costs minutes. The gate runs it with
/// `--ignored` as its own step.
#[test]
#[ignore = "cold build by construction; the gate runs this with --ignored"]
fn Test_The_Pinned_Crossing_Should_Really_Compile_From_A_Cold_Target_Directory()
{
    let revision = Declared_Revision();
    let target = Empty_Target_Directory();

    let output = std::process::Command::new("cargo")
        .args(["check", "-p", SUBJECT])
        .current_dir(Repository_Root())
        .env("CARGO_TARGET_DIR", &target)
        .output()
        .unwrap_or_else(|error| panic!("cannot run cargo: {error}"));

    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Gather every verdict before cleaning up, so a failure does not leave the directory behind
    // and an assertion does not run before the removal it would skip.
    let succeeded = output.status.success();
    let (compiled_count, unpinned) = Verdict(&log, &revision);
    let tail: String = log.lines().rev().take(15).collect::<Vec<&str>>().join("\n");

    let _ = std::fs::remove_dir_all(&target);

    assert!(
        succeeded,
        "a cold `cargo check -p {SUBJECT}` failed. The pinned crossing does not build from an \
         empty target directory, whatever an incremental build in the shared one reports.\n\n{tail}"
    );

    assert!(
        compiled_count > 0,
        "a cold `cargo check -p {SUBJECT}` succeeded without compiling any xvpe- crate at \
         revision {revision}.\n\n\
         That is the exact failure this module exists for: an exit code of zero that answered a \
         different question from the one asked. Either the crossing is no longer reached from \
         this crate -- in which case the subject is wrong and this module needs a new one -- or \
         the target directory was not actually empty and the build inherited its answer.\n\n{tail}"
    );

    assert!(
        unpinned.is_empty(),
        "a cold build reached xvpe crates that did not come from revision {revision}: \
         {unpinned:?}.\n\n\
         A crate built here without the pinned git source came from somewhere else -- a path, a \
         patch, or a different revision -- and the crossing this workspace publishes is not the \
         one it actually compiled against. That is the whole failure D-130's surviving clause \
         exists to prevent, arriving in the build rather than in the lock.\n\n{tail}"
    );
}

/// The verdict rejects a log that does not show the crossing, and accepts one that does.
///
/// Not ignored: this is pure text and costs nothing. It is the only way this module's real
/// assertion is known to be capable of failing, since the spawned build is immune to anything
/// the invoking environment could do to it.
#[test]
fn Test_A_Log_That_Misses_The_Crossing_Should_Be_Rejected()
{
    const REVISION: &str = "82a3c8fccf4ef7f3759f36d3f320a91d0f96341c";

    let warm = "    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.20s\n";
    let pinned = "    Checking xvpe-primitives v0.1.0 (https://github.com/kevinmettias/xvpe.git?rev=82a3c8fccf4ef7f3759f36d3f320a91d0f96341c#82a3c8fc)\n";
    let from_a_path = "    Checking xvpe-primitives v0.1.0 (/repos/xvpe/crates/foundations/contracts/xvpe-primitives)\n";
    let wrong_revision = "    Checking xvpe-clock v0.1.0 (https://github.com/kevinmettias/xvpe.git?rev=0000000000000000000000000000000000000000)\n";

    let (count, unpinned) = Verdict(warm, REVISION);
    assert!(
        count == 0 && unpinned.is_empty(),
        "a log showing no crossing at all reported {count} pinned crates. This is the warm-build \
         case the module exists for, and it must read as nothing proved rather than as clean."
    );

    let (count, unpinned) = Verdict(pinned, REVISION);
    assert!(
        count == 1 && unpinned.is_empty(),
        "a correctly pinned build log was not accepted; the verdict is wrong in the direction \
         that would fail every honest commit."
    );

    let (_, unpinned) = Verdict(from_a_path, REVISION);
    assert!(
        !unpinned.is_empty(),
        "an xvpe crate built from a local path was accepted. That is the form D-130 refuses, \
         arriving in the build rather than in the lock, and it is the substitution this check is \
         supposed to be able to see."
    );

    let (_, unpinned) = Verdict(wrong_revision, REVISION);
    assert!(
        !unpinned.is_empty(),
        "an xvpe crate built from a different revision was accepted. The crossing would be \
         published as one commit and compiled from another."
    );
}

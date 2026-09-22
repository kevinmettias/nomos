//! Proof that a profile is what its doc says: a pure function of one tree, refused for a
//! root that is not a directory, and the same value every time it is computed.
//!
//! File names are spelled as literals here on purpose. An assertion against the constant
//! that produced a name would pass whatever the constant said; the literal pins the
//! spelling a reader in this workspace actually opens.

use super::WorkspaceProfile;
use crate::test_support::{Fresh_Root, Make_Fixture_Directory, Write_Fixture, Write_Fixture_Bytes};
use crate::{LanguageManifest, PolicyFile, PolicyFilePresence, ProfileRefusal, ROOT_MARKER, SourceCount};

/// Bytes no UTF-8 decoder accepts, so a file made of them is there and cannot be read as
/// text on any platform -- the portable way to make a policy file present but unreadable.
const NOT_TEXT: &[u8] = &[0xff, 0xfe, 0xfd];

#[test]
fn Test_Of_Root_Should_Refuse_A_Root_That_Does_Not_Exist()
{
    let root = std::env::temp_dir().join("nomos-workspace-discovery-profile-missing-root");
    let _ignored = std::fs::remove_dir_all(&root);

    let refusal = WorkspaceProfile::Of_Root(&root).expect_err("a path that is not a directory is refused");

    assert_eq!(refusal, ProfileRefusal::RootIsNotADirectory { root: root.clone() });
    assert!(refusal.to_string().contains("is not a directory"), "{refusal}");
}

/// A file exists, so a guard that asked only whether the path exists would profile it; the
/// refusal is about being a directory, not about being there.
#[test]
fn Test_Of_Root_Should_Refuse_A_File_Given_As_The_Root()
{
    let root = Fresh_Root("nomos-workspace-discovery-profile-file-root");
    let file = root.join("a.rs");
    Write_Fixture(file.clone(), "pub fn One() {}\n");

    let outcome = WorkspaceProfile::Of_Root(&file);

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(outcome, Err(ProfileRefusal::RootIsNotADirectory { root: file }));
}

#[test]
fn Test_Of_Root_Should_Profile_An_Empty_Directory_Rather_Than_Refuse_It()
{
    let root = Fresh_Root("nomos-workspace-discovery-profile-empty");

    let profile = WorkspaceProfile::Of_Root(&root).expect("an empty directory is still a directory");

    let _ignored = std::fs::remove_dir_all(&root);
    assert!(profile.sources.iter().all(|count| return count.sources == 0), "{profile:?}");
    assert!(profile.manifests.iter().all(|manifest| return !manifest.is_present), "{profile:?}");
    assert!(profile.policy_files.iter().all(|file| return file.presence == PolicyFilePresence::Absent), "{profile:?}");
    assert!(!profile.has_root_marker);
}

/// Two Rust files at different depths, one Go file, one Rust file under `target` the walk
/// skips, and one file of no registered language: the counts are the walk's population,
/// not the directory's.
#[test]
fn Test_Of_Root_Should_Count_Sources_Per_Registered_Extension()
{
    let root = Fresh_Root("nomos-workspace-discovery-profile-counts");
    Write_Fixture(root.join("a.rs"), "pub fn One() {}\n");
    Make_Fixture_Directory(root.join("nested"));
    Write_Fixture(root.join("nested/b.rs"), "pub fn Two() {}\n");
    Write_Fixture(root.join("main.go"), "package main\n");
    Make_Fixture_Directory(root.join("target"));
    Write_Fixture(root.join("target/built.rs"), "pub fn Built() {}\n");
    Write_Fixture(root.join("notes.md"), "not source\n");

    let profile = WorkspaceProfile::Of_Root(&root).expect("a directory profiles");

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(
        profile.sources,
        vec![
            SourceCount { extension: nomos_lang_rust::RUST_EXTENSION, sources: 2 },
            SourceCount { extension: nomos_lang_go::GO_EXTENSION, sources: 1 },
        ]
    );
}

#[test]
fn Test_Of_Root_Should_Report_Which_Manifests_Are_Present()
{
    let root = Fresh_Root("nomos-workspace-discovery-profile-manifests");
    Write_Fixture(root.join("Cargo.toml"), "[workspace]\n");
    Write_Fixture(root.join("go.work"), "go 1.21\n");

    let profile = WorkspaceProfile::Of_Root(&root).expect("a directory profiles");

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(
        profile.manifests,
        vec![
            LanguageManifest { name: "Cargo.toml", extension: nomos_lang_rust::RUST_EXTENSION, is_present: true },
            LanguageManifest { name: "go.mod", extension: nomos_lang_go::GO_EXTENSION, is_present: false },
            LanguageManifest { name: "go.work", extension: nomos_lang_go::GO_EXTENSION, is_present: true },
        ]
    );
}

/// A manifest below the root is a member's, and does not make the root a manifest root.
#[test]
fn Test_Of_Root_Should_Not_Report_A_Nested_Manifest_As_The_Roots()
{
    let root = Fresh_Root("nomos-workspace-discovery-profile-nested-manifest");
    Make_Fixture_Directory(root.join("member"));
    Write_Fixture(root.join("member/Cargo.toml"), "[package]\n");

    let profile = WorkspaceProfile::Of_Root(&root).expect("a directory profiles");

    let _ignored = std::fs::remove_dir_all(&root);
    assert!(profile.manifests.iter().all(|manifest| return !manifest.is_present), "{profile:?}");
}

/// All four files asked about, three of the answers present in one tree: text, bytes that
/// are not text, and nothing at all.
#[test]
fn Test_Of_Root_Should_Report_Each_Policy_File_By_Name_With_What_It_Found()
{
    let root = Fresh_Root("nomos-workspace-discovery-profile-policy-files");
    Write_Fixture(root.join(ROOT_MARKER), "{}\n");
    Write_Fixture_Bytes(root.join("nomos-gate.json"), NOT_TEXT);
    Write_Fixture(root.join("nomos-test-material.json"), "{}\n");

    let profile = WorkspaceProfile::Of_Root(&root).expect("a directory profiles");

    let _ignored = std::fs::remove_dir_all(&root);
    let names: Vec<&str> = profile.policy_files.iter().map(|file| return file.name).collect();
    assert_eq!(names, vec!["standards.json", "nomos-gate.json", "nomos-architecture.json", "nomos-test-material.json"]);
    assert_eq!(Presence_Of(&profile, "standards.json"), &PolicyFilePresence::Present);
    assert!(matches!(Presence_Of(&profile, "nomos-gate.json"), PolicyFilePresence::PresentButUnreadable { .. }), "{profile:?}");
    assert_eq!(Presence_Of(&profile, "nomos-architecture.json"), &PolicyFilePresence::Absent);
    assert_eq!(Presence_Of(&profile, "nomos-test-material.json"), &PolicyFilePresence::Present);
    assert!(profile.has_root_marker);
}

/// A directory of a policy file's name is the other way a path can be there and not be
/// readable, and it is the one a reader would refuse with a different reason.
#[test]
fn Test_Of_Root_Should_Report_A_Directory_Of_A_Policy_Files_Name_As_Present_But_Unreadable()
{
    let root = Fresh_Root("nomos-workspace-discovery-profile-policy-directory");
    Make_Fixture_Directory(root.join("nomos-architecture.json"));

    let profile = WorkspaceProfile::Of_Root(&root).expect("a directory profiles");

    let _ignored = std::fs::remove_dir_all(&root);
    assert!(
        matches!(Presence_Of(&profile, "nomos-architecture.json"), PolicyFilePresence::PresentButUnreadable { .. }),
        "{profile:?}"
    );
}

/// The root marker and the `standards.json` policy entry are one file read two ways, so a
/// tree without it must say so in both places.
#[test]
fn Test_Of_Root_Should_Report_No_Root_Marker_When_The_Root_Has_No_Standards_File()
{
    let root = Fresh_Root("nomos-workspace-discovery-profile-no-marker");
    Write_Fixture(root.join("a.rs"), "pub fn One() {}\n");

    let profile = WorkspaceProfile::Of_Root(&root).expect("a directory profiles");

    let _ignored = std::fs::remove_dir_all(&root);
    assert!(!profile.has_root_marker);
    assert_eq!(Presence_Of(&profile, ROOT_MARKER), &PolicyFilePresence::Absent);
}

/// Determinism, proven two ways: the same tree profiled twice gives equal values, and the
/// whole profile of one tree is asserted as a literal, so the order of every list is pinned
/// rather than merely stable. The files are written in an order that is not the reported
/// one.
#[test]
fn Test_Of_Root_Should_Be_Deterministic_For_A_Given_Tree()
{
    let root = Fresh_Root("nomos-workspace-discovery-profile-deterministic");
    Write_Fixture(root.join("z.go"), "package z\n");
    Write_Fixture(root.join("nomos-test-material.json"), "{}\n");
    Write_Fixture(root.join("go.mod"), "module example.com/z\n");
    Write_Fixture(root.join("a.rs"), "pub fn One() {}\n");
    Write_Fixture(root.join(ROOT_MARKER), "{}\n");

    let first = WorkspaceProfile::Of_Root(&root).expect("a directory profiles");
    let second = WorkspaceProfile::Of_Root(&root).expect("a directory profiles");

    let _ignored = std::fs::remove_dir_all(&root);
    assert_eq!(first, second);
    assert_eq!(first, Expected_Deterministic_Profile());
}

/// The whole profile of the tree `Test_Of_Root_Should_Be_Deterministic_For_A_Given_Tree`
/// builds, as a literal.
fn Expected_Deterministic_Profile() -> WorkspaceProfile
{
    return WorkspaceProfile {
        sources: vec![SourceCount { extension: "rs", sources: 1 }, SourceCount { extension: "go", sources: 1 }],
        manifests: vec![
            LanguageManifest { name: "Cargo.toml", extension: "rs", is_present: false },
            LanguageManifest { name: "go.mod", extension: "go", is_present: true },
            LanguageManifest { name: "go.work", extension: "go", is_present: false },
        ],
        policy_files: vec![
            PolicyFile { name: "standards.json", presence: PolicyFilePresence::Present },
            PolicyFile { name: "nomos-gate.json", presence: PolicyFilePresence::Absent },
            PolicyFile { name: "nomos-architecture.json", presence: PolicyFilePresence::Absent },
            PolicyFile { name: "nomos-test-material.json", presence: PolicyFilePresence::Present },
        ],
        has_root_marker: true,
    };
}

/// What the profile found of the policy file called `name`.
fn Presence_Of<'profile>(profile: &'profile WorkspaceProfile, name: &str) -> &'profile PolicyFilePresence
{
    let file = profile.policy_files.iter().find(|file| return file.name == name);
    return &file.expect("every policy file the profile asks about is reported").presence;
}

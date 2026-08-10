//! The two governing tables, assembled from one registration file per record.
//!
//! `records/<ID>.record` is the declaration that makes a record governing. This script
//! reads **that directory and only that directory**, and writes two bare expressions into
//! `OUT_DIR` for `src/governing.rs` to `include!` as initializers.
//!
//! # Why the initializers and not the items
//!
//! `RECORDS` and `GOVERNING_RECORD_IDS` keep their declaration sites written by hand in
//! `governing.rs`. `nomos_rules::Universes_In` recognises a declared universe by matching
//! `syn::Item::Const` with a slice type and public visibility, and never inspects the
//! initializer — so an `include!` in initializer position is invisible to it and the
//! completeness census is unchanged. An `include!` at *item* position is `syn::Item::Macro`
//! and is dropped: the workspace's most-cited declared universe would vanish from the
//! census, `Test_The_Declared_Table_Should_Match_What_Is_Derived` would report it as
//! vanished, and deleting the row to make that green would leave it uncounted forever. The
//! declaration site is the thing being preserved; the initializer is the thing generated.
//!
//! For the same reason the generated fragments go to `OUT_DIR` and never into `src/`: a
//! bare `&[…]` expression is not a file, the scanner reports it `Unparseable`, and
//! `Check_Completeness_Mirrors` turns that into a finding.
//!
//! # Why this script is not where the parse lives
//!
//! It is in `src/registration.rs`, reached below with `#[path]`, because a build script
//! cannot depend on its own crate and a second copy of a format's reader is how two readers
//! come to disagree. See that module for the rest of the reasoning, and for the refusals.
//!
//! This script has no dependencies and must keep none: adding one would move the crate's
//! band and the `tests/contract` allowlist with it.

#[path = "src/registration.rs"]
mod registration;

use registration::{Registration, Registrations_In};
use std::fmt::Write;
use std::path::{Path, PathBuf};

fn main()
{
    let manifest = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("cargo sets CARGO_MANIFEST_DIR for every build script"),
    );
    let out = PathBuf::from(
        std::env::var("OUT_DIR").expect("cargo sets OUT_DIR for every build script"),
    );

    let directory = manifest.join("records");
    let root = Repository_Root(&manifest);
    let registrations = Registered(&directory, &root);

    let identifiers = Identifier_Table(&registrations);
    std::fs::write(out.join("governing_record_ids.rs"), identifiers)
        .expect("the generated identifier table must be writable");

    let records = Record_Table(&registrations, &root);
    std::fs::write(out.join("governing_records.rs"), records)
        .expect("the generated record table must be writable");

    Rerun_Triggers(&directory, &root, &registrations);
}

/// `crates/spec/nomos-spec-store` -> the repository.
///
/// Used only to check that a named record is on disk and to spell the `include_str!`
/// argument; nothing under it is enumerated, which is the whole of why this arrangement is
/// not vacuous.
fn Repository_Root(manifest: &Path) -> PathBuf
{
    return manifest
        .ancestors()
        .nth(3)
        .expect("the crate sits three directories below the repository root")
        .to_path_buf();
}

/// Every registered record, or a stopped build.
///
/// A refusal here stops the build, which is the point. A registration skipped instead of
/// refused is a governing record that quietly stops governing.
fn Registered(directory: &Path, root: &Path) -> Vec<Registration>
{
    return match Registrations_In(directory, root)
    {
        Ok(registrations) => registrations,
        Err(error) => panic!("{}", error.Describe()),
    };
}

/// The initializer for `GOVERNING_RECORD_IDS`.
fn Identifier_Table(registrations: &[Registration]) -> String
{
    let mut table = String::from("&[\n");

    for registration in registrations
    {
        // `{:?}` so the string literal is escaped by the formatter rather than by hand.
        writeln!(table, "    {:?},", registration.id).expect("a string is always writable");
    }

    table.push_str("]\n");
    return table;
}

/// The initializer for `RECORDS`.
///
/// The first element of each pair stays repository-relative: it becomes
/// `source_documents.path` in the store, and a machine-specific path there would put the
/// build machine's directory layout into a content-addressed row.
///
/// The `include_str!` argument is absolute, because a relative path inside a file included
/// from `OUT_DIR` resolves against `OUT_DIR`, which is opaque. It reaches no hash and no
/// stored row, so determinism is unaffected.
fn Record_Table(registrations: &[Registration], root: &Path) -> String
{
    let mut table = String::from("&[\n");

    for registration in registrations
    {
        let absolute = Absolute(root, &registration.path);
        writeln!(
            table,
            "    (\n        {:?},\n        include_str!({absolute:?}),\n    ),",
            registration.path
        )
        .expect("a string is always writable");
    }

    table.push_str("]\n");
    return table;
}

/// A repository-relative record path as an absolute one, in the spelling rustc accepts on
/// every platform this builds on.
fn Absolute(root: &Path, path: &str) -> String
{
    return root.join(path).display().to_string().replace('\\', "/");
}

/// What must change for these tables to be assembled again.
///
/// The directory catches a registration added or removed; each registration catches its own
/// body being edited; each record catches its bytes changing, which `include_str!` embeds.
fn Rerun_Triggers(directory: &Path, root: &Path, registrations: &[Registration])
{
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=src/registration.rs");
    println!("cargo::rerun-if-changed={}", directory.display());

    for registration in registrations
    {
        let file = directory.join(format!("{}.record", registration.id));
        println!("cargo::rerun-if-changed={}", file.display());
        println!("cargo::rerun-if-changed={}", Absolute(root, &registration.path));
    }
}

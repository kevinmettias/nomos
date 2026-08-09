//! What this binary is actually being built as, passed through to the crate.
//!
//! A [`nomos_workspace::BuildVariant`] names a target triple, a profile, a toolchain and a
//! feature set. None of the four is reachable from a compiled program: `std::env::consts`
//! offers an architecture and an operating system, which is not a triple —
//! `x86_64-pc-windows-msvc` and `x86_64-pc-windows-gnu` agree on both and are two different
//! programs with two different ABIs. Cargo knows the answer at build time and nothing knows
//! it afterwards, so it is captured here.
//!
//! The alternative was writing the triple down as a constant. That is precisely the defect
//! this item exists to remove: an identity component nobody derived from anything, which is
//! wrong on every machine but the one it was authored on and cannot be seen to be wrong.

use std::collections::BTreeSet;

fn main()
{
    // Cargo sets both for every build script. A missing one means the contract with cargo
    // has changed, and guessing would produce a variant identity describing a build that
    // never happened.
    let target = std::env::var("TARGET").expect("cargo sets TARGET for every build script");
    let profile = std::env::var("PROFILE").expect("cargo sets PROFILE for every build script");

    // Absent outside rustup — a direct rustc invocation, a distribution toolchain, a
    // vendored compiler. Named as unstated rather than defaulted to a version, because a
    // variant claiming `1.85` on a toolchain nobody identified is a false statement about
    // which compiler produced the facts.
    let toolchain = std::env::var("RUSTUP_TOOLCHAIN").unwrap_or_else(|_| return "unstated".to_owned());

    // Cargo exports one variable per enabled feature and no list. A set, so the order they
    // are read in cannot reach the variant's identity.
    let features: BTreeSet<String> = std::env::vars()
        .filter_map(|(name, _)| {
            return name
                .strip_prefix("CARGO_FEATURE_")
                .map(|feature| return feature.to_lowercase().replace('_', "-"));
        })
        .collect();

    println!("cargo::rustc-env=NOMOS_TARGET={target}");
    println!("cargo::rustc-env=NOMOS_PROFILE={profile}");
    println!("cargo::rustc-env=NOMOS_TOOLCHAIN={toolchain}");
    println!(
        "cargo::rustc-env=NOMOS_FEATURES={}",
        features.into_iter().collect::<Vec<String>>().join(",")
    );

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=RUSTUP_TOOLCHAIN");
}

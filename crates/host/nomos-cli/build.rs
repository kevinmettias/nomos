//! What this binary is actually being built as, passed through to the crate.
//!
//! A [`nomos_workspace::BuildVariant`] names a target triple, a profile, a toolchain and a
//! feature set, and every fact `nomos check` materializes is filed under its identity.
//! None of the four is reachable from a compiled program: `std::env::consts` offers an
//! architecture and an operating system, which is not a triple —
//! `x86_64-pc-windows-msvc` and `x86_64-pc-windows-gnu` agree on both and are two
//! different programs with two different ABIs. Cargo knows the answer at build time and
//! nothing knows it afterwards, so it is captured here.
//!
//! The alternative was writing a constant into the composition root. An identity component
//! nobody derived from anything cannot be *wrong*, so it can never be observed to be
//! wrong: two machines, two toolchains and two policies all agree, and the disagreement
//! they should have had is the one a fact key exists to detect. `tests/integration/build.rs`
//! captures the same four values for the same reason and this file is deliberately its
//! twin — a build script may not depend on a crate, so the two cannot be one.

use std::collections::BTreeSet;

fn main() -> Result<(), String>
{
    // Cargo sets both for every build script. A missing one means the contract with cargo
    // has changed, and guessing would produce a variant identity describing a build that
    // never happened.
    let target = Cargo_Variable("TARGET")?;
    let profile = Cargo_Variable("PROFILE")?;

    // Absent outside rustup — a direct rustc invocation, a distribution toolchain, a
    // vendored compiler. Named as unstated rather than defaulted to a version, because a
    // variant claiming `1.85` on a toolchain nobody identified is a false statement about
    // which compiler produced the facts.
    let toolchain =
        std::env::var("RUSTUP_TOOLCHAIN").unwrap_or_else(|_| return "unstated".to_owned());

    let features = Enabled_Features();

    println!("cargo::rustc-env=NOMOS_TARGET={target}");
    println!("cargo::rustc-env=NOMOS_PROFILE={profile}");
    println!("cargo::rustc-env=NOMOS_TOOLCHAIN={toolchain}");
    println!("cargo::rustc-env=NOMOS_FEATURES={features}");

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=RUSTUP_TOOLCHAIN");

    return Ok(());
}

/// One variable cargo sets for every build script.
///
/// Returned rather than unwound. A build script's caller is cargo, which prints the `Err`
/// and fails the build — the same stop, with the same sentence, reached by a path the
/// caller acknowledged.
fn Cargo_Variable(name: &str) -> Result<String, String>
{
    return std::env::var(name)
        .map_err(|_| return format!("cargo sets {name} for every build script, and did not"));
}

/// The enabled features as one comma-separated field.
///
/// Cargo exports one variable per enabled feature and no list. Collected into a set, so the
/// order they happen to be read in cannot reach the variant's identity.
fn Enabled_Features() -> String
{
    let features: BTreeSet<String> = std::env::vars()
        .filter_map(|(name, _)| {
            return name
                .strip_prefix("CARGO_FEATURE_")
                .map(|feature| return feature.to_lowercase().replace('_', "-"));
        })
        .collect();

    return features.into_iter().collect::<Vec<String>>().join(",");
}

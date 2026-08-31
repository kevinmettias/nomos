//! The seam between `nomos_cli`'s walk (`check/sources.rs`, `gate/sources.rs`) and
//! `nomos_lang_rust`.
//!
//! Both walks hardcode `"rs"` as a literal rather than depending on `nomos-lang-rust` in
//! production code -- `Read_Entry`'s own doc comment says so explicitly: "The extension
//! check is a literal, the same as `nomos_lang_rust::RUST_EXTENSION` ... already states,
//! rather than a dependency on either crate: this walk decides which bytes are worth
//! reading at all, not which registered provider answers for them." That is a claim of
//! agreement between two independently-spelled values, and nothing before this file checked
//! it: `nomos_lang_rust::RUST_EXTENSION` could change and the walk's own literal would not
//! notice.
//!
//! This proves both halves: the constant equals the literal the walk actually uses, and the
//! real binary actually walks and materializes a fact for that exact extension.

#[path = "support/mod.rs"]
mod support;

use support::{Run, Tree};

/// The literal `check/sources.rs` and `gate/sources.rs` hardcode, restated here as an
/// assertion against the real constant rather than left as two independently-spelled
/// strings that happen to agree today.
#[test]
fn Test_Rust_Extension_Constant_Should_Equal_The_Literal_The_Walk_Hardcodes()
{
    assert_eq!(nomos_lang_rust::RUST_EXTENSION, "rs");
}

/// A tree holding nothing but a file named with `nomos_lang_rust::RUST_EXTENSION` is
/// walked, examined, and materializes a syntax fact -- driven through the real binary, not
/// through `check::sources::Read_Sources` directly, since that is not part of this
/// bin-only crate's public surface.
#[test]
fn Test_Check_Should_Examine_And_Materialize_A_Fact_For_A_File_Named_With_The_Rust_Extension()
{
    let tree = Tree::New("lang-rust-seam")
        .With(&format!("a.{}", nomos_lang_rust::RUST_EXTENSION), "pub fn One() {}\n");

    let ran = Run(&["check", "--root", &tree.Root()]);

    assert_eq!(ran.code, 0, "{}", ran.stdout);
    assert!(
        ran.stdout.contains("1 file(s) examined, 1 with a syntax fact"),
        "a lone `.{}` file must be walked and materialize a fact: {}",
        nomos_lang_rust::RUST_EXTENSION,
        ran.stdout
    );
}

//! The seam between `nomos_cli`'s walk (`check/sources.rs`, `gate/sources.rs`) and
//! `nomos_lang_go`.
//!
//! Production code takes no dependency on `nomos-lang-go` -- `Read_Entry`'s own doc comment
//! is explicit: the `"go"` extension check is a literal, "the same as ...
//! `nomos_lang_go::GO_EXTENSION` already state[s], rather than a dependency on either crate."
//! That sentence is an unchecked claim of agreement between two independently-spelled
//! values. `nomos-lang-go` is added below only as a `[dev-dependencies]` entry (see this
//! crate's own `Cargo.toml`) so this one test can check the claim without giving the shipped
//! binary a dependency its walk does not need.
//!
//! This proves both halves: the constant equals the literal the walk actually uses, and the
//! real binary actually walks and materializes a fact for that exact extension -- measured
//! directly against the built binary: a tree holding nothing but one `.go` file is walked,
//! examined, and reports `1 file(s) examined, 1 with a syntax fact`, exit `0`; the same tree
//! with an unrecognized extension (`.golang`) is not walked at all and exits `6`.

#[path = "support/mod.rs"]
mod support;

use support::{Run, Tree};

/// What a tree the walk recognizes no source in leaves the process with -- `check/exit_code.rs::
/// ExitCode::Vacuous`'s own value, the same code an empty walk produces.
const VACUOUS_EXIT_CODE: i32 = 6;

/// The literal `check/sources.rs` and `gate/sources.rs` hardcode, restated here as an
/// assertion against the real constant rather than left as two independently-spelled
/// strings that happen to agree today.
#[test]
fn Test_Go_Extension_Constant_Should_Equal_The_Literal_The_Walk_Hardcodes()
{
    assert_eq!(nomos_lang_go::GO_EXTENSION, "go");
}

/// A tree holding nothing but a file named with `nomos_lang_go::GO_EXTENSION` is walked,
/// examined, and materializes a syntax fact -- driven through the real binary, not through
/// `check::sources::Read_Sources` directly, since that is not part of this bin-only crate's
/// public surface. This is the walk-level guarantee `check/sources.rs`'s own inline
/// `Test_Read_Sources_Should_Discover_A_Go_File_Alongside_A_Rust_One` proves in process; this
/// is the same property proven through the compiled binary, tied to the real constant rather
/// than to a repeated literal.
#[test]
fn Test_Check_Should_Examine_And_Materialize_A_Fact_For_A_File_Named_With_The_Go_Extension()
{
    let tree = Tree::New("lang-go-seam-recognized")
        .With(&format!("main.{}", nomos_lang_go::GO_EXTENSION), "package main\n\nfunc One() {}\n");

    let ran = Run(&["check", "--root", &tree.Root()]);

    assert_eq!(ran.code, 0, "{}", ran.stdout);
    assert!(
        ran.stdout.contains("1 file(s) examined, 1 with a syntax fact"),
        "a lone `.{}` file must be walked and materialize a fact: {}",
        nomos_lang_go::GO_EXTENSION,
        ran.stdout
    );
}

/// The negative control: a file named with an extension that is close to, but not, the real
/// constant is not walked at all. Without this, the test above could pass for the wrong
/// reason -- a walk that recognized every extension, not specifically this one.
#[test]
fn Test_Check_Should_Not_Walk_A_File_Named_With_An_Unrecognized_Extension_Close_To_Go()
{
    assert_ne!(nomos_lang_go::GO_EXTENSION, "golang", "the negative control must actually differ from the real constant");

    let tree = Tree::New("lang-go-seam-unrecognized").With("main.golang", "package main\n\nfunc One() {}\n");

    let ran = Run(&["check", "--root", &tree.Root()]);

    assert_eq!(ran.code, VACUOUS_EXIT_CODE, "an extension the walk does not recognize must not be examined: {}", ran.stdout);
    assert!(!ran.stdout.contains("file(s) examined"), "{}", ran.stdout);
}

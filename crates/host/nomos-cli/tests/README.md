<!-- folder-organization: coherent: each `*_seam.rs` file proves exactly one crate-boundary
claim from `check-integration-coverage`'s own findings -- `nomos-cli` is a `[[bin]]`-only
crate, so every one of them drives the compiled binary itself (see `support/mod.rs`) rather
than a shared internal module tree, and there is no subsystem grouping among them that isn't
already named by the file: `lang_go_seam.rs` is the nomos_lang_go seam, `spec_store_seam.rs`
is the nomos_spec_store seam, and so on. A subfolder over any subset would assert a
relationship between those claims that nothing decides, the same reasoning
`nomos-ledger/tests/exclusion_holds` gives for its own flat module list. -->

# Integration tests

`check_command.rs` and `list_tells_the_truth.rs` predate the `*_seam.rs` files below; the
seam files were added to close `check-integration-coverage` findings and follow the same
subprocess-driven convention `check_command.rs` established.

<!-- folder-organization: coherent: one directory per responsibility family, not one per
     dependency band -- substrate/ alone holds four (18, 20, 21, 22). The band table in
     root README.md is the sole authority for what each crate may depend on, and the
     boundary assertions in tests/contract check that table against this tree in both
     directions regardless of which family folder a crate sits in. A family folder groups
     crates by what they do; interposing a folder between this level and a band, so that
     each folder held exactly one band, would put a name in every crate path that no
     authority defines, and would move the crates the band table names to paths it does
     not. -->

# crates

One directory per responsibility family -- contracts, kernel, platform, substrate,
capabilities, packages, languages, rules, corrections, orchestration, spec, host -- not one
per dependency band; several of these folders hold crates from more than one band. What the
bands are, what each may depend on, and which crate belongs to which are answered by root
README.md and checked against this tree by the boundary assertions; none of it is restated
here, because a restated table is an unchecked copy of a checked one.

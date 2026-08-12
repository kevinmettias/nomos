<!-- folder-organization: coherent: one directory per dependency band, which is the
     workspace's architecture rather than a grouping chosen here. The band table in the
     repository README is the authority for what each one may depend on, and the boundary
     assertions in tests/contract check that table against this tree in both directions.
     Interposing a folder between this level and a band would put a name in every crate
     path that no authority defines, and would move the crates the band table names to
     paths it does not. -->

# crates

One directory per dependency band. What the bands are, what each may depend on, and which
crate belongs to which are answered by the repository's own README and checked against this
tree by the boundary assertions; none of it is restated here, because a restated table is an
unchecked copy of a checked one.

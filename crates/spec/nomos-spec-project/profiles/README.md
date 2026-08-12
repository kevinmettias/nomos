<!-- folder-organization: coherent: one JSON file per shipped projection profile, each
     reached by name through an include_str! in src/catalogue.rs. The flatness is the
     contract: the catalogue names every file at this level exactly once, and grouping them
     into subfolders would put a classification in the path that nothing else in this
     workspace decides, while changing every include_str! to carry it. Which profiles this
     repository is obliged to ship is OD-PROJECT-002's list, not a directory. -->

# profiles

One file per shipped projection profile. `src/catalogue.rs` is what compiles them in, and
`OD-PROJECT-002` is what decides which of them this repository is required to render; a
profile is present here whether or not it is required, so this directory is not that list.

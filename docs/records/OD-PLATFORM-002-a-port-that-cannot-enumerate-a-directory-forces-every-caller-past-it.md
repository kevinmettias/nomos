---
id: OD-PLATFORM-002
type: decision
title: A port that cannot enumerate a directory forces every caller past it
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - platform
  - filesystem
relations:
  - target: OD-PLATFORM-001
    type: relates-to
---

# A port that cannot enumerate a directory forces every caller past it

## Question

`nomos_platform::FileSystem` offers `Read_To_String`, `Replace_Atomically`, `Exists` and
`Remove_File` and nothing that lists a directory's own contents. `crates/host/nomos-cli/src/
check/sources.rs` documents the consequence directly: its recursive source walk stays on
`std::fs::read_dir` because "the same division `nomos-cli::work::Published_Records` draws
around the directory listing `nomos_platform::FileSystem` has no port for" — a defensible
division while one host walked a tree, and a mechanical cause of duplication once more than
one did. `nomos-cli` and `nomos-api` each carry their own walker and their own hardcoded
extension list, and neither reaches the port for the one operation both actually need.

## What Was Measured

Grepped across `crates/host/` for `std::fs::read_dir`: `nomos-api/src/sources.rs`,
`nomos-api/src/work/add_response.rs`, `nomos-cli/src/check/sources.rs`, `nomos-cli/src/
correct.rs`, `nomos-cli/src/gate/sources.rs`, `nomos-cli/src/work.rs` and `nomos-cli/src/
workflow.rs` each call it directly. `nomos-cli/src/work.rs::Record_Files` is the simplest of
the seven — one level, no recursion, already carrying the comment naming the port gap as the
reason it stayed at the composition root rather than moving into `nomos-work-orchestration`
with everything else `WorkCommand::Add` needs.

## The Decision

**`FileSystem` gains `Read_Directory(&self, path: &Path) -> Result<Vec<PathBuf>,
FileSystemError>`**, returning the immediate children of one directory — files and
subdirectories together, one level, not a recursive walk. A caller that needs to descend
composes its own recursion from this primitive, the same way every other port method leaves
retry, fallback and iteration to its caller.

**Defaulted, like `Remove_File`, and for the identical reason.** Every existing implementor
of `FileSystem` in this workspace is a narrow, hand-written fake built to exercise one test's
own fixture; none has a directory to enumerate. Making the method required would force every
one of those fakes — scattered across dozens of test files, none of them this record's
territory — to grow an implementation, or fail to compile, for a capability they do not
exercise. The default refuses rather than reports an empty listing, so a fake that never
overrides it says so honestly instead of reading like a real, empty directory.

**`nomos-platform-std::StdFileSystem` provides the real implementation**, over
`std::fs::read_dir`, classifying its failure the same three ways every other method on this
struct already does.

**`nomos-cli/src/work.rs::Record_Files` and `Published_Records` are the first real
caller**, migrated from `std::fs::read_dir` to `filesystem.Read_Directory`, taking a
`&impl FileSystem` parameter rather than reaching past the port. `Run` passes `&StdFileSystem`
at the one call site, the same composition root that already chooses every other concrete
platform type for this binary.

## What This Does Not Do

It does not migrate the other six call sites this record's own measurement found.
`check/sources.rs`'s own walk is recursive and stated to be its own increment, not a
consequence of the port gaining a one-level primitive — collapsing a multi-level walk onto
`Read_Directory` is real work with its own territory, and `nomos-api`'s three call sites are
a second crate's own item to claim. This record closes the port-level gap and proves it
against one real, simple caller; it does not claim every caller now goes through it.

It does not give the port a recursive walk, a glob, or a filter. `Read_Directory` is the
same shape `std::fs::read_dir` already has — one directory's own immediate contents — and a
caller wanting more composes it, the same division this port already draws for every other
operation.

## Status

Accepted. `FileSystem::Read_Directory` exists, `StdFileSystem` implements it, and
`nomos-cli`'s own record-listing function is its first real caller.

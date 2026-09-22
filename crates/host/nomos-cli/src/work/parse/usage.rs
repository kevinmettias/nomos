//! The two blocks of prose `nomos work`'s usage text is assembled from.
//!
//! Split out of `parse.rs` once that file passed the ~500-line review trigger. Parsing a
//! command line and printing what the parser accepts on a refusal are different reading;
//! nothing in this file decides anything, it only says.

/// Every verb and what it takes.
pub(super) const VERBS: &str = "\x20 list     [--state ready|waiting|held|lapsed|snagged|stranded|claimed|blocked|done|declined] [--all]\n\
     \x20          the live board by default: every item that has not ended. `--all` prints \
     the whole board, the finished and the declined with it, and a bounded listing says at \
     its foot how many rows that would add. `--state` names one bucket and answers with it \
     whether or not that bucket has ended. `OD-LEDGER-041`.\n\
     \x20          `ready` means claimable now. An item nothing can claim is reported as \
     `waiting` (a dependency is unfinished), `held` (somebody holds overlapping territory), \
     `lapsed` (its holder's lease ran out, so `takeover` applies), `snagged` (independence \
     cannot be established) or `stranded` (a dependency was declined and will never finish), \
     from the same refusal `claim` would give.\n\
     \x20 show     --item <id>\n\
     \x20          one item in full: its claim, every claim given up on it with the reason \
     given, its verification, and then its contract — the `why` and the `done_when` whole and \
     unabridged, the paths it reserves, and the predicate declared to judge it. `list` is a \
     column per item and cannot carry prose. This is the verb `AGENTS.md`'s loop means when it \
     says to read the item's terms before editing anything, so nothing here is clipped or \
     summarized however long it runs.\n\
     \x20 add      --item <id> --title <text> --why <text> --done-when <text>\n\
     \x20          --kind capability|decision|validation|correction|cleanup\n\
     \x20          --origin required|proposed\n\
     \x20          --territory <path> [--territory <path> …]\n\
     \x20          [--amends <record> …]\n\
     \x20          [--depends-on <id> …]\n\
     \x20          [-- <program> <args…>] [--timeout 2h]\n\
     \x20          `--kind` says what sort of work this is; `--origin` says whether a person \
     required it or a session proposed it. Both are required and both are closed sets: an \
     unrecognized value is refused rather than stored. `OD-LEDGER-024`.\n\
     \x20          `--amends` reserves a record this repository has already published and \
     says the item edits it. Reserving a published record any other way is refused, because \
     an identifier is allocated once and the two acts are otherwise the same act. Either \
     spelling works — the identifier or the file — and it reserves what it names, so the \
     record does not also need `--territory`.\n\
     \x20          `--timeout` bounds the predicate named after `--`, defaulting to 10 \
     minutes like the predicate itself does; it is refused if given with no predicate to \
     bound. `finish` halves it for the idle bound (`OD-PLATFORM-001`), so a predicate \
     expected to run quiet for a while — a corpus-backed test, say — needs a longer one \
     declared here rather than left at the default.\n\
     \x20 claim    --item <id> --holder <name> [--lease 2h]\n\
     \x20 renew    --item <id> --holder <name> [--lease 2h]\n\
     \x20 takeover --item <id> --holder <name> [--lease 2h]\n\
     \x20          takes over an item listed `lapsed` — one whose holder's lease ran out. \
     `claim` never takes over a lapsed item; `takeover` does, and records the claim it \
     displaced, which `show` then reports.\n\
     \x20 finish   --item <id> --holder <name>\n\
     \x20 abandon  --item <id> --holder <name> --reason <text>\n\
     \x20 decline  --item <id> --holder <name> --reason <text>\n\
     \x20          ends an item that turned out not to be work — superseded by another item, \
     or refused by a record since it was written. `abandon` ends a *claim* and puts the item \
     back on the board for somebody else; `decline` ends the *item*, and takes no claim, \
     because an item nobody intends to do should not have to be claimed first. It refuses an \
     item somebody is holding, and one already done or already declined.\n\
     \x20 widen    --item <id> --holder <name> --territory <path> [--territory <path> ...]\n\
     \x20          adds paths to the territory of an item you hold, when execution proved \
     the reservation short of the change. It only ever adds: dropping a path drops the \
     `done_when` clause it carried, so there is no spelling here for a replacement \
     territory. The enlarged territory is checked against every live claim by the check \
     a `claim` goes through, the enlargement is recorded on the item and `show` reports \
     it, and a holder whose lease has run out is refused -- a lapsed claim stops \
     excluding, so `takeover` comes first. `OD-LEDGER-039`.\n\
     \x20 validate\n\
     \x20 audit\n";

/// What holds across every verb: the predicate, and the codes an agent branches on.
pub(super) const NOTES: &str = "\neverything after `--` is the verification predicate, run directly \
     with no shell. `finish` runs the gate's own lint step first, derived from \
     .github/workflows/gate.yml rather than written here, then the item's predicate, and \
     records the item done only if both exit zero. An item whose predicate passes while the \
     gate is red is not finished.\n\
     \n\
     exit codes: 0 ok, 1 validation error, 2 usage, 3 claim unavailable (retryable), \
     4 conflict, 5 store error";

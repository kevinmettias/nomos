//! Band 10 — the `nomos` binary.
//!
//! One binary with subcommands, not several binaries. A separate `nomos-work` would
//! grow its own flags, its own output conventions and eventually its own idea of what a
//! claim is — and the architecture's rule that every user-visible action has one
//! canonical service would have been violated by the first milestone.
//!
//! Everything here is a projection. The CLI parses arguments, calls a library, and
//! renders the result; it decides nothing. When the service layer arrives these
//! subcommands become renderers over service contracts rather than direct library
//! calls, and the shape will not have to change.

#![forbid(unsafe_code)]

mod work;

use std::path::PathBuf;

fn main() -> std::process::ExitCode
{
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut stdout = std::io::stdout();

    let code = match arguments.split_first()
    {
        Some((group, rest)) if group == "work" =>
        {
            match work::Parse(rest)
            {
                Ok(command) => work::Run(&command, &Work_Directory(), &mut stdout),
                Err(message) =>
                {
                    eprintln!("{message}");
                    work::ExitCode::Usage
                }
            }
        }
        _ =>
        {
            eprintln!("usage: nomos <group> <command>\n\n  work   coordinate concurrent work over this repository");
            work::ExitCode::Usage
        }
    };

    return std::process::ExitCode::from(u8::try_from(code.Value()).unwrap_or(1));
}

/// Where the ledger lives.
///
/// Overridable so that tests and tools can point at a scratch ledger without changing
/// directory, which is what makes the whole surface testable from one process.
fn Work_Directory() -> PathBuf
{
    return std::env::var_os("NOMOS_WORK_DIR")
        .map_or_else(|| PathBuf::from("work"), PathBuf::from);
}

//! `--sarif`, the one flag `nomos gate run` and `nomos check` both take to emit their judgment
//! as a SARIF 2.1.0 log, and where that log is written.
//!
//! The projection is `nomos_gate_orchestration::SarifLog`, the judging service's own
//! (`OD-HOST-017`), and nothing here renders a finding. This module reads the flag, hands the
//! verb's judgment to the projection, and writes the text the projection returns -- through
//! `nomos_platform::FileSystem`, so a path it cannot write is refused by the same port every
//! other file this binary writes goes through.
//!
//! # Where the human report goes
//!
//! Given a path, the log goes to that file and the human report to standard output, unchanged.
//! Given none, the log is standard output, so the human report moves to standard error: a CI
//! job that redirects standard output into a `.sarif` file must get a document and nothing
//! else, and a person at the terminal still sees the report.
//!
//! # Exit codes
//!
//! The flag changes none. A gate's disposition is its judgment and not a property of how it was
//! printed, so a log that could not be written is reported on standard error with the path it
//! could not write, and the verb exits with the code its judgment earned.

use nomos_composer_std::FILE_SYSTEM;
use nomos_gate_orchestration::SarifLog;
use nomos_platform::FileSystem;
use std::io::Write;
use std::path::PathBuf;

/// The flag, spelled the same for both verbs.
pub(crate) const SARIF_FLAG: &str = "--sarif";

/// Where a `--sarif` log is written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SarifDestination
{
    /// No path followed the flag, so the log is written to standard output.
    StandardOutput,
    /// The path that followed the flag.
    File(PathBuf),
}

/// Where `arguments` ask for a SARIF log, or `None` when they carry no `--sarif`.
///
/// The path is optional, so the argument after the flag is its path only when that argument is
/// not itself a flag. A path that begins with `-` cannot be named directly for that reason;
/// `./-log.sarif` can.
pub(crate) fn Sarif_Destination_From_Arguments(arguments: &[String]) -> Option<SarifDestination>
{
    let mut from_the_flag = arguments.iter().skip_while(|argument| return argument.as_str() != SARIF_FLAG);
    from_the_flag.next()?;

    return Some(match from_the_flag.next()
    {
        Some(value) if !value.starts_with('-') => SarifDestination::File(PathBuf::from(value)),
        _ => SarifDestination::StandardOutput,
    });
}

/// Runs `render` with the writer the human report belongs on, given where the log goes, and
/// answers with what `render` answered.
///
/// A log with a file of its own leaves standard output to the report. A log that is standard
/// output sends the report to standard error, after anything `render` itself wrote there, so the
/// two streams never interleave inside the document.
pub(crate) fn Rendered_Beside_Log<Code>(
    destination: &SarifDestination,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
    render: impl FnOnce(&mut dyn Write, &mut dyn Write) -> Code,
) -> Code
{
    return match destination
    {
        SarifDestination::StandardOutput =>
        {
            let mut report = Vec::new();
            let code = render(&mut report, stderr);
            stderr.write_all(&report).ok();
            code
        }
        SarifDestination::File(_) => render(stdout, stderr),
    };
}

/// Writes `log` where `destination` names, and says on `stderr` when it could not -- naming the
/// path for a file, since a refusal that does not say where is one a CI job cannot act on.
pub(crate) fn Emit_Log(log: &SarifLog, destination: &SarifDestination, stdout: &mut impl Write, stderr: &mut impl Write)
{
    let text = match log.Serialized()
    {
        Ok(text) => text,
        Err(error) =>
        {
            writeln!(stderr, "the SARIF log could not be serialized: {error}").ok();
            return;
        }
    };

    match destination
    {
        SarifDestination::StandardOutput =>
        {
            writeln!(stdout, "{text}").ok();
        }
        SarifDestination::File(path) =>
        {
            if let Err(error) = FILE_SYSTEM.Replace_Atomically(path, &format!("{text}\n"))
            {
                writeln!(stderr, "the SARIF log could not be written to {}: {error}", path.display()).ok();
            }
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Arguments(words: &[&str]) -> Vec<String>
    {
        return words.iter().map(|word| return (*word).to_owned()).collect();
    }

    #[test]
    fn Test_No_Flag_Should_Ask_For_No_Log()
    {
        assert_eq!(Sarif_Destination_From_Arguments(&Arguments(&["--root", "."])), None);
    }

    #[test]
    fn Test_A_Flag_Followed_By_A_Path_Should_Name_That_File()
    {
        assert_eq!(
            Sarif_Destination_From_Arguments(&Arguments(&["--sarif", "out.sarif", "--root", "."])),
            Some(SarifDestination::File(PathBuf::from("out.sarif")))
        );
    }

    /// The path is optional, and the flag with nothing after it or with another flag after it
    /// means standard output -- the two positions a caller can leave the value out in.
    #[test]
    fn Test_A_Flag_With_No_Path_Should_Mean_Standard_Output()
    {
        assert_eq!(Sarif_Destination_From_Arguments(&Arguments(&["--root", ".", "--sarif"])), Some(SarifDestination::StandardOutput));
        assert_eq!(Sarif_Destination_From_Arguments(&Arguments(&["--sarif", "--root", "."])), Some(SarifDestination::StandardOutput));
    }
}

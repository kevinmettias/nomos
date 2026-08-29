//! Starting a child process with both output streams piped, and nothing on stdin.

use nomos_platform::Command;

use super::drain::Drain;

/// Starts `program` as a child and begins draining both its streams immediately.
///
/// Reading starts from the moment the child starts, and not after it ends. A child
/// cannot finish writing more than a pipeful unless somebody is taking it away, so this
/// is what makes the wait that follows a wait on the program rather than a wait on its
/// volume.
pub(super) fn Spawned_With_Streams(
    command: &Command,
    program: &str,
) -> Result<(std::process::Child, Option<Drain>, Option<Drain>), String>
{
    let mut child = Spawned_Program(command, program)?;
    let stdout = child.stdout.take().map(Drain::Reading);
    let stderr = child.stderr.take().map(Drain::Reading);

    return Ok((child, stdout, stderr));
}

/// The child's two captured streams, carried together.
///
/// The wait loop and the length check inside it both need "how much has either stream
/// produced" and neither ever judges one stream without the other, so the pair travels
/// as one value instead of two positional parameters a caller could transpose.
pub(super) struct Streams<'a>
{
    pub(super) stdout: Option<&'a Drain>,
    pub(super) stderr: Option<&'a Drain>,
}

/// The child, started with both its streams piped and nothing on its input.
///
/// `stdin` is null rather than inherited, because a predicate that stops to read from a
/// terminal nobody is at would hang until the timeout and report as slow work.
fn Spawned_Program(command: &Command, program: &str) -> Result<std::process::Child, String>
{
    use std::process::Stdio;

    let arguments = command.argv.get(1..).unwrap_or_default();
    let mut builder = std::process::Command::new(program);
    builder
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(directory) = &command.working_directory
    {
        builder.current_dir(directory);
    }

    return builder
        .spawn()
        .map_err(|error| return format!("could not start `{program}`: {error}"));
}

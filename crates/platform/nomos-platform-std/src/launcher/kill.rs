//! Killing a child that outran its budget, tree and all, and reaping it.

/// Kills a child that outran its budget, along with whatever it spawned, and reaps it.
///
/// A direct kill only ever reaches the one process this launcher started. `cargo test`
/// starts the compiled test binary as its own child, and killing `cargo` alone leaves
/// that binary running — which is the process that was actually hanging, still there
/// after the thing watching it believes the hang is over. [`Kill_Tree`] is what reaches
/// the whole tree instead of the one process at its root.
///
/// A child that ended between the last poll and this kill is already in the state the kill was
/// after, and the platforms disagree about how they say so: Unix reports success against a
/// not-yet-reaped child, Windows answers `InvalidInput`. Every other failure means a process
/// this function promised to stop is still running, and reporting the timeout would be a lie
/// about it.
///
/// A wait that fails leaves the zombie this exists to avoid, so it is reported rather than
/// dropped.
pub(super) fn Killed_And_Reaped(child: &mut std::process::Child, program: &str) -> Result<(), String>
{
    if let Err(cause) = Kill_Tree(child)
        && cause.kind() != std::io::ErrorKind::InvalidInput
    {
        return Err(format!("`{program}` outran its timeout and could not be killed: {cause}"));
    }

    child
        .wait()
        .map_err(|cause| return format!("could not reap `{program}` after killing it: {cause}"))?;

    return Ok(());
}

/// Kills a child and every process it spawned, where the platform can be asked to.
///
/// `Child::kill` is `TerminateProcess` on Windows and `SIGKILL` on Unix, and both reach
/// only the one process named by the handle or the pid — neither walks down to what that
/// process started. `taskkill /T` does: given a pid it walks the process tree rooted
/// there and terminates every process in it, which is what "kill the child" has to mean
/// when the child is `cargo test` and the thing actually hanging is the test binary
/// `cargo` spawned underneath it.
///
/// `cfg!(windows)` rather than `#[cfg(windows)]`: this workspace's gate lints only the
/// Linux leg on the strength of there being no conditionally-compiled code for it to
/// miss, and a compile-time `#[cfg]` here would make that no longer true. The runtime
/// check keeps both branches ordinary, always-compiled Rust, so nothing about this
/// function is invisible to the leg that does not run it.
///
/// A `taskkill` that cannot run at all — wrong platform, not on `PATH`, the process
/// already gone — is not reported as this function's own failure. [`Child::kill`] below
/// is the fallback that actually matters for those cases, and it already carries its own
/// accounting for "the process was already gone" via the `InvalidInput` the caller
/// checks for.
fn Kill_Tree(child: &mut std::process::Child) -> std::io::Result<()>
{
    if cfg!(windows)
    {
        let tree_killed = std::process::Command::new("taskkill")
            .args(["/T", "/F", "/PID", &child.id().to_string()])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        if matches!(tree_killed, Ok(status) if status.success())
        {
            return Ok(());
        }
    }

    return child.kill();
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Killed_And_Reaped_Should_Stop_And_Reap_A_Running_Child()
    {
        let mut child = Spawned_A_Long_Running_Child();

        let result = Killed_And_Reaped(&mut child, "nomos-platform-std-kill-test");

        assert!(result.is_ok(), "killing a still-running child must succeed: {result:?}");
        assert!(
            matches!(child.try_wait(), Ok(Some(_))),
            "the child must already be reaped, not merely killed"
        );
    }

    fn Spawned_A_Long_Running_Child() -> std::process::Child
    {
        let mut command = if cfg!(windows)
        {
            let mut command = std::process::Command::new("ping");
            command.args(["-n", "30", "127.0.0.1"]);
            command
        }
        else
        {
            let mut command = std::process::Command::new("sleep");
            command.arg("30");
            command
        };

        return command
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("spawns a long-running child for the kill test");
    }
}

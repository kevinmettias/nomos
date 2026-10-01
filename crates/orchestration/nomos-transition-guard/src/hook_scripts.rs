//! The hook scripts `nomos guard install` writes (`OD-POLICY-002` decision 6).
//!
//! `core.hooksPath` replaces a repository's `.git/hooks` entirely, so every script this module
//! writes -- judged or not -- ends by handing over to the repository's own hook of the same
//! name. Without that, pointing git at these scripts would silently stop a repository's Git LFS
//! hooks, or a push refusal it keeps for itself, from ever running.

/// One script: the hook name git runs it for, and its text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HookScript
{
    /// The git hook name, which is also the file name.
    pub name: &'static str,
    /// The `sh` script.
    pub text: String,
}

/// The hooks that are judged, each with the `nomos guard` verb it runs and whether git feeds it
/// standard input. `pre-merge-commit` is judged as `pre-commit`: a merge commit adds content too.
const JUDGED: [(&str, &str, bool); 4] =
    [("pre-commit", "pre-commit", false), ("pre-merge-commit", "pre-commit", false), ("commit-msg", "commit-msg", false), ("pre-push", "pre-push", true)];

/// Hooks only handed over. `reference-transaction` is left out: git runs it on every ref
/// update, and a script it would only pass through is a cost every repository pays for nothing.
const HANDED_OVER: [&str; 13] = [
    "applypatch-msg",
    "pre-applypatch",
    "post-applypatch",
    "prepare-commit-msg",
    "post-commit",
    "pre-rebase",
    "post-checkout",
    "post-merge",
    "post-rewrite",
    "pre-auto-gc",
    "push-to-checkout",
    "sendemail-validate",
    "post-index-change",
];

const HEADER: &str = "#!/bin/sh\n# Written by `nomos guard install` (OD-POLICY-002). Re-run install rather than editing.\n";

/// Every script, for a `nomos` binary at `nomos` judging against every policy in `policies`, in
/// the order given. A judged hook runs one `nomos guard` line naming them all, so git sees one
/// answer and its standard input is read once.
#[must_use]
pub fn Hook_Scripts(nomos: &str, policies: &[String]) -> Vec<HookScript>
{
    let nomos = Single_Quoted(nomos);
    let policies: String = policies.iter().map(|policy| return format!(" --policy {}", Single_Quoted(policy))).collect();
    let judged = JUDGED.iter().map(|(name, verb, reads_input)| {
        let judge_line = format!("{nomos} guard {verb}{policies} \"$@\"");
        let text = if *reads_input
        {
            format!(
                "{HEADER}input=$(cat; echo x); input=${{input%x}}\nprintf '%s' \"$input\" | {judge_line} || exit $?\n{}\
                 if [ -f \"$own_hook\" ]; then printf '%s' \"$input\" | \"$own_hook\" \"$@\"; exit $?; fi\nexit 0\n",
                Own_Hook_Line(name)
            )
        }
        else
        {
            format!("{HEADER}{judge_line} || exit $?\n{}if [ -f \"$own_hook\" ]; then exec \"$own_hook\" \"$@\"; fi\nexit 0\n", Own_Hook_Line(name))
        };
        return HookScript { name, text };
    });
    let handed_over = HANDED_OVER.iter().map(|name| {
        let text = format!("{HEADER}{}if [ -f \"$own_hook\" ]; then exec \"$own_hook\" \"$@\"; fi\nexit 0\n", Own_Hook_Line(name));
        return HookScript { name, text };
    });

    return judged.chain(handed_over).collect();
}

fn Own_Hook_Line(name: &str) -> String
{
    return format!("own_hook=\"$(git rev-parse --git-common-dir)/hooks/{name}\"\n");
}

/// `text` as one `sh` word, whatever it holds.
fn Single_Quoted(text: &str) -> String
{
    return format!("'{}'", text.replace('\'', "'\\''"));
}

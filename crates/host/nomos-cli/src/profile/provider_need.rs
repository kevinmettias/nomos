//! What one composed provider needs from this host before it can answer.
//!
//! # Why this table exists here, and what it is not allowed to be
//!
//! Every offer `nomos_check_orchestration::Registered` composes is linked into this binary,
//! so a provider's *code* is present by construction. What is not present by construction is
//! whatever that provider then runs: three of them build a command for a
//! [`nomos_platform::ProgramLauncher`] and one runs GitHub's own CLI. Nothing this crate may
//! name declares which of them do — `nomos_capability::ProviderOffer` carries an identity, a
//! version and a [`nomos_contracts::Guarantee`], and a guarantee is a statement about the
//! fact's strength rather than about the machine it was produced on. `nomos-cli` does not
//! depend on a single provider crate and `P128` adds no dependency, so the pairing is
//! restated here.
//!
//! A restatement can go stale, and the way it goes stale decides whether this verb lies. So
//! the default is the conservative one: [`Need_Of`] answers `None` for a provider nothing
//! here names, and the caller renders that as *undetermined* rather than as available. A
//! provider composed after this table was written therefore shows up in the report as
//! something this verb does not know about — which is the diagnosis, not a silent green.
//! `Test_An_Unknown_Provider_Should_Be_Undetermined_Rather_Than_Available` is the falsifier
//! for exactly that, and removing the `None` arm is what it is written to catch.
//!
//! The crates that own each row are named beside it, so the next author can go read the
//! launch rather than trust this line.

/// The environment variable three Rust providers read the program name from —
/// `nomos-lang-rust-cargo`, `nomos-lang-rust-clippy` and `nomos-lang-rust-deny` each call
/// `Environment::Variable("CARGO")` and fall back to the literal below. Read here through
/// the same port, by the same rule, so this verb probes for the program a real run would
/// actually launch rather than for the one a reader assumed.
const CARGO_VARIABLE: &str = "CARGO";

/// What those three fall back to when `CARGO` is unset.
const CARGO_PROGRAM: &str = "cargo";

/// GitHub's own CLI, which `nomos-connector-coderabbit::fetching::Fetch_Review_Comment`
/// runs as `gh api repos/{repository}/pulls/comments/{id}`. It reads no variable for the
/// program name, so there is none to name beside it.
const GITHUB_CLI_PROGRAM: &str = "gh";

/// What a provider needs from this host beyond being linked into this binary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProviderNeed
{
    /// The provider answers from inside this binary: it parses text, or reads a file
    /// through the filesystem port, and launches nothing. There is nothing about this host
    /// that could make it unavailable.
    InThisBinary,
    /// The provider runs a program, and whether that program is here is a fact about this
    /// machine rather than about this build.
    HostProgram
    {
        /// The environment variable naming the program, when the provider reads one.
        variable: Option<&'static str>,
        /// The program name the provider uses when the variable is unset or there is none.
        fallback: &'static str,
    },
}

/// A program read from the environment, with the fallback the provider itself would use.
const fn Named_By(variable: &'static str, fallback: &'static str) -> ProviderNeed
{
    return ProviderNeed::HostProgram { variable: Some(variable), fallback };
}

/// A program named by a literal in the provider, with no variable to override it.
const fn Named_Outright(fallback: &'static str) -> ProviderNeed
{
    return ProviderNeed::HostProgram { variable: None, fallback };
}

/// How many providers this table names — every provider identity
/// `nomos_check_orchestration::Registered` offers as of `P128`, counted here so a row
/// dropped by an edit is a compile error rather than a quietly shorter list.
const DECLARED_NEED_COUNT: usize = 16;

/// Every composed provider, paired with what it needs from this host.
///
/// Spelled as the identity strings the registry reports, because that is what this verb has
/// in hand: `nomos_capability::ProviderOffer::provider` is what a capability's offers are
/// read off, and matching on it is the only join available to a crate that may not name a
/// provider crate.
const DECLARED_NEEDS: [(&str, ProviderNeed); DECLARED_NEED_COUNT] = [
    // `nomos-lang-rust`, for both `nomos.cap.syntax.items` and
    // `nomos.cap.controlflow.reachability` — its reachability offer reuses the same identity.
    ("nomos.lang.rust.syn", ProviderNeed::InThisBinary),
    ("nomos.lang.rust.scan", ProviderNeed::InThisBinary),
    ("nomos.lang.go.tree-sitter", ProviderNeed::InThisBinary),
    // `nomos-lang-go-modules` reads `go.work` and `go.mod` as text and launches nothing;
    // its own module doc says so in as many words.
    ("nomos.lang.go.modules", ProviderNeed::InThisBinary),
    // The six repository policy families and the architecture and requirement-trace
    // declarations: `nomos-repo-policy` and `nomos-cap-requirement-trace` read a file at the
    // root through the filesystem port.
    ("nomos.repo.standards", ProviderNeed::InThisBinary),
    ("nomos.repo.limits", ProviderNeed::InThisBinary),
    ("nomos.repo.scripting", ProviderNeed::InThisBinary),
    ("nomos.repo.goals", ProviderNeed::InThisBinary),
    ("nomos.repo.words", ProviderNeed::InThisBinary),
    ("nomos.repo.test-material", ProviderNeed::InThisBinary),
    ("nomos.repo.architecture", ProviderNeed::InThisBinary),
    ("nomos.repo.requirement.trace", ProviderNeed::InThisBinary),
    // `nomos-lang-rust-cargo` runs `cargo metadata`.
    ("nomos.lang.rust.cargo", Named_By(CARGO_VARIABLE, CARGO_PROGRAM)),
    // `nomos-lang-rust-clippy` runs `cargo clippy`.
    ("nomos.lang.rust.clippy", Named_By(CARGO_VARIABLE, CARGO_PROGRAM)),
    // `nomos-lang-rust-deny` runs `cargo deny`.
    ("nomos.lang.rust.deny", Named_By(CARGO_VARIABLE, CARGO_PROGRAM)),
    // `nomos-connector-coderabbit` runs `gh api`.
    ("nomos.connector.coderabbit", Named_Outright(GITHUB_CLI_PROGRAM)),
];

/// What `provider` needs from this host, or `None` when this table does not name it.
///
/// `None` is not "needs nothing". It is "this verb has no declaration for this provider",
/// and the caller is required to render it as undetermined — see this module's own doc for
/// why the default falls that way rather than the other.
#[must_use]
pub(crate) fn Need_Of(provider: &str) -> Option<ProviderNeed>
{
    for (name, need) in DECLARED_NEEDS
    {
        if name == provider
        {
            return Some(need);
        }
    }

    return None;
}

//! The baseline-debt lines a judged run prints: which declared scope holds more than it
//! accepted, and what a reader may and may not conclude from that.

use nomos_gate_orchestration::{BaselineAllowance, BaselinePopulation};
use std::io::Write;

/// Says which baselined scopes hold more debt than they accepted, and refuses to say more.
///
/// Only the exceeded ones. A line for every baselined scope would put the ordinary case --
/// adopted debt sitting where it was adopted -- in front of a reader on every clean run, and a
/// report whose every line is routine is one whose exceptional line gets skipped.
///
/// The closing sentence is the part that is easy to drop and must not be. `OD-GATE-030` refuses
/// attribution inside an exceeded population: a run knows the scope is over its allowance by a
/// number and does not know which of the occurrences present are the adopted ones. Saying only
/// "4 more than accepted" invites a reader to decide for themselves which four, which is
/// exactly the claim nothing here can support.
pub(super) fn Report_Exceeded_Baselines(populations: &[BaselinePopulation], stdout: &mut impl Write)
{
    let exceeded: Vec<&BaselinePopulation> = populations.iter().filter(|population| return population.Is_Exceeded()).collect();
    if exceeded.is_empty()
    {
        return;
    }

    let _ = writeln!(stdout, "\nbaseline debt has grown past what was adopted:");
    for population in exceeded
    {
        Report_Exceeded_Scope(population, stdout);
    }

    Report_Unattributable_Debt(stdout);
}

/// One exceeded scope's arithmetic: the rule, the scope as its author wrote it, how many
/// occurrences it holds now, what was accepted at adoption, and the excess between them.
fn Report_Exceeded_Scope(population: &BaselinePopulation, stdout: &mut impl Write)
{
    let _ = writeln!(
        stdout,
        "  {} at {}: {} occurrence(s) now, {} accepted at adoption, {} more than accepted",
        population.rule.As_Str(),
        Scope_Name(population),
        population.observed,
        Accepted_Count(population.allowed),
        population.Excess()
    );
}

/// The part of an exceeded population's report that is easy to drop and must not be.
///
/// `OD-GATE-030` refuses attribution inside an exceeded population: a run knows the scope is
/// over its allowance by a number and does not know which of the occurrences present are the
/// adopted ones. Saying only "4 more than accepted" invites a reader to decide for themselves
/// which four, which is exactly the claim nothing here can support.
fn Report_Unattributable_Debt(stdout: &mut impl Write)
{
    let _ = writeln!(
        stdout,
        concat!(
            "Which of the occurrences present are the ones that were adopted is not known, ",
            "so none of them is reported as new. What is known is the quantity: a scope ",
            "holding more than it accepted holds at least that many occurrences that cannot ",
            "be the adopted ones."
        )
    );
}

/// The scope as its author wrote it, which is the whole reason this is not `population.subject`.
///
/// A declared entry names a path and the run matches the digest that path folds to, so the
/// digest is the right identity and the wrong thing to print: the fold is one way, and a reader
/// told `a scope at e3f85dfb…` cannot find the entry they wrote. Several spellings of one path
/// fold together -- `./a.rs`, `a.rs`, `A.rs` -- so the author's own spelling is not recoverable
/// from the subject by anyone, this function included, which is why the entry carries it.
///
/// The digest appears only when there is no authored spelling to prefer, which means a policy
/// built in code rather than declared in a file. Saying the digest there is honest and saying
/// one of several possible paths would not be. It is printed bare rather than dressed as a path
/// so that a reader can tell the two cases apart.
fn Scope_Name(population: &BaselinePopulation) -> String
{
    return match &population.declared_path
    {
        Some(path) => path.clone(),
        None => population.subject.to_string(),
    };
}

/// The accepted quantity, for a scope that has one.
///
/// An unbounded scope never reaches this, because it can never be exceeded. The word is here
/// rather than a zero anyway, so that a caller who later prints every population does not
/// report "0 accepted" for an entry that accepted everything.
fn Accepted_Count(allowance: BaselineAllowance) -> String
{
    return match allowance
    {
        BaselineAllowance::Unbounded => "no stated limit".to_owned(),
        BaselineAllowance::AtMost(count) => count.to_string(),
    };
}

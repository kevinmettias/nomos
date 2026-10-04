# Live-file checks in wave corrections

Both correction entry points compare each target's live text with the candidate's
base before committing the workspace model or requesting an atomic replacement.
A changed, removed or unreadable target refuses the plan. The wave report retains
any earlier commits, names the refusal, and leaves remaining plans and waves
unattempted. An unchanged target can still commit.

The public wave regression changes or removes a real target during the baseline
rejudgment, after candidate construction. A separate invalid UTF-8 regression
checks that a failed reread leaves both file bytes and the workspace snapshot
unchanged. The partial-commit regression retains the first committed file,
preserves a competing edit to the second, and never attempts the next wave.

```powershell
cargo test --locked --offline -p nomos-correction-orchestration --no-fail-fast
```

This reread detects observed staleness. The filesystem port still exposes reread
and replacement as separate operations, so an external writer can change a target
between them. Race-free concurrent publication needs a stronger shared filesystem
contract. The model's validation also does not execute a repository's compiler
or tests over an isolated staged tree. These remaining limits keep this evidence
from establishing the complete Nomos 0.1 correction acceptance bar.

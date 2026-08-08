---
id: D-045
type: decision
title: Runtime capture boundary
status: accepted
version: 1
authority: canonical-normative-record
relations: []
---

# Runtime capture boundary

## Decision

OfflineCaptureEvaluation and StreamingObservation are inside the approved release-scope product contract.

## Rationale

RUNTIME-005 depends on this decision to keep runtime observation capabilities explicit and bounded.

## Consequences

ActiveRuntimeGuard remains outside the approved contract unless a later product, security, and authorization decision admits it.

## Alternatives Considered

The prior prose did not preserve an authored decision artifact for this ID. This record restores the stable decision target without expanding the approved runtime scope.

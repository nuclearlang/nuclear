---
layer: L4
component: system
read-when: you start or continue a feature that has a plan, or you write a new feature plan
---

# Feature specs (L4)

One folder per feature: `NNN-slug/`. A feature starts as `plan.md` (goal, diagnosis, features with a "done"
criterion, work order, decisions pending for the user) and, when it starts, gets `tasks.md` (an ordered,
test-first checklist). The rules it produces end up in `spec/` and the decisions in `docs/adr/`; the plan
records how the work is split, not the final rules.

| Spec | Status |
|---|---|
| [001-agent-support](001-agent-support/plan.md) | ready, not started (3 questions for the user in §6) |
| [002-dialects-with-agents](002-dialects-with-agents/plan.md) | decided, not started |
| [003-shared-contracts](003-shared-contracts/plan.md) | step 3 of the competitiveness plan; delivered 2026-11-06 (ADR-0098) |
| [004-backend-parity](004-backend-parity/plan.md) | step 4 of the competitiveness plan; plan with examples, not started (ADR first for the data parts) |
| [005-language-1-0](005-language-1-0/plan.md) | step 6 of the competitiveness plan; plan with examples, not started |
| [006-public-repositories](006-public-repositories/plan.md) | step 1 of the competitiveness plan: the organization `nuclearlang` and the split into public repositories; not started |

Status and progress live in `STATUS.md`; this table only points to the folders.

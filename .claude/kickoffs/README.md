# agent-mcp local kickoffs (leaf under wsfull orchestrator)

This is a **leaf kickoff** managed by the central workspace orchestrator (`wsfull`).

Main brief: root (loc.md + wsfull.md for orchestrator direction).

**How it runs under wsfull**:
- Isolated worktree spawned by orchestrator.
- Working branch inside worktree.
- Scoped work + change-scoped tests + early security scans (patch vulns early).
- PR polished to dev.
- Orchestrator integrates in dev (wiring, integration/regression tests incl. security), then full PR to main (fully integrated).

Alpha orchestration, protocol, router. Use for coordinated waves / NL single interface under the central kickoff.

Disjoint ownership. PR to dev after tests + security.

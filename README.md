---
title: cell
tags: cell, soft3
status: draft
---
# cell

Cell is the shared model and host for addressable, stateful organs, services,
ledgers and cybergraph regions. In cyb, a loaded cell grows an ability of the robot.

Start with [the specification](specs/README.md).
The [convergence explanation](docs/cell-convergence.md) records the reasoning,
source review and relationship to the existing stack.
The [foundations and Hermes review](docs/foundations-agent-review.md) explains
the 0.2 corrections and the evidence needed to demonstrate a stronger agent.

The repository currently contains specifications for review. Runtime code and
published packages follow review of these contracts.

History is represented in cybergraph and persisted through its storage stack.
Cell defines and coordinates its transitions. The cyb log organ presents history.

## repository

- specs/ — normative candidate contracts, version 0.2.
- docs/ — explanations and source findings.
- LICENSE — the Cyber License used by the companion repositories.

Companion repositories are siblings under ~/cyber. Relative source links assume
that layout. [Integration](specs/integration.md) identifies dependency changes
needed to implement the contracts; present code is distinguished from the target.

# LocalTwin CLI — Product Requirements Document (PRD)

**Status:** Proposed production baseline  
**Version:** 1.0  
**Product:** Local-first CLI for coding, development, and design assistance  
**Primary interface:** Terminal; interactive, compact, JSON, and CI modes  
**Core brains:** Coding/Development Brain and Design/UX Brain

## 1. Executive Summary

LocalTwin is a self-hosted CLI that gives developers and product designers a controlled AI workspace over a local repository. It connects to local LLM runtimes, assembles inspectable repository context, routes work to two specialized brains, proposes structured edits, executes only approved tools, validates outcomes, and preserves checkpoints for recovery.

The product is explicitly not an autonomous code bot. It is a human-controlled orchestration layer around local models. Models are untrusted generators; policy, approval, execution, and verification live outside the model. The core loop is **ask → inspect → plan → review → approve → execute → verify → checkpoint**.

“Two brains” means two separately configured capability contracts, not necessarily two separately pretrained foundation models. The Coding Brain specializes in code, tests, debugging, and tool use. The Design Brain specializes in user flows, information architecture, interaction states, accessibility, content, visual critique, design tokens, and developer handoffs. Either may share a runtime or base model, but their prompts, schemas, evaluations, and provenance remain distinct.

## 2. Problem Statement

Developers need repository-aware help but cannot always send source code, credentials, product knowledge, or terminal output to a hosted service. Generic assistants also fail to expose what they read, why they selected a file, what a command will do, or whether a claimed fix was actually tested. Design decisions and implementation decisions frequently drift apart: a visually attractive recommendation may not match the codebase, while a technically easy implementation may produce a poor experience.

LocalTwin addresses five problems:

1. **Privacy and locality:** code and design artifacts remain inside the configured trust boundary.
2. **Context quality:** relevant files, symbols, tests, Git state, screenshots, and design tokens are selected deliberately.
3. **Action safety:** model output becomes structured proposals; writes, shell commands, network access, and Git changes are policy-controlled.
4. **Cross-discipline continuity:** Design Brain outputs become confirmed, structured handoffs for Coding Brain.
5. **Verification:** results are labeled as proposed, executed, tested, previewed, or unverified.

## 3. Vision, Principles, and Product Promise

### 3.1 Vision

Make high-quality coding and product-design assistance available anywhere a user can run a terminal, without surrendering source code or operational control.

### 3.2 Principles

- Local by default; remote providers are never silently used.
- Human control before consequential impact.
- Specialized brains with shared, explicit context.
- Every material claim should have evidence, citation, or an uncertainty label.
- Patches and checkpoints are preferred over opaque mutations.
- Interactive UX and automation must share one deterministic core.
- Safe degradation is better than hidden failure.
- Color, Unicode, and a full-screen UI are optional; clear text is mandatory.

## 4. Goals and Non-goals

### Goals

- Connect to local OpenAI-compatible and native model runtimes.
- Provide coding, design, reasoning, vision, and embedding capabilities behind a router.
- Explain repositories with citations and inspectable context manifests.
- Generate plans, structured patches, tests, design flows, UX critiques, and handoffs.
- Apply approved changes safely, run bounded verification, and support rollback.
- Work offline, in SSH sessions, in CI, and in narrow terminals.
- Provide local audit records, exports, stable JSON schemas, and exit codes.

### Non-goals for v1

- Replacing a full IDE or visual design tool.
- Unbounded autonomous operation.
- Automatic production deployment, merge, force-push, or secret access.
- A hosted model marketplace or centralized source-code telemetry service.
- Guaranteeing correctness without tests or human review.
- Treating inferred design tokens as canonical without confirmation.

## 5. Personas

| Persona | Needs | Primary success condition |
|---|---|---|
| Privacy-conscious developer | Local assistance, easy setup, no account | First useful answer without network |
| Regulated/air-gapped engineer | Audit, policy, deterministic behavior | Safe operation under offline restrictions |
| Full-stack product engineer | Code plus UX continuity | Design handoff becomes implementable work |
| Product designer in a repository | Flows, states, accessibility, copy | Implementation-ready UX artifact |
| Platform maintainer | Repeatable installation and diagnostics | Can support users without source upload |
| CI operator | Bounded, scriptable analysis | Stable JSON and non-interactive safety |

## 6. Core Use Cases and User Journeys

### UC-001 Explain code
`localtwin ask "Where is authentication configured?"` discovers relevant entry points, configuration, tests, and docs; returns citations, assumptions, and omitted-context reasons.

### UC-002 Diagnose failure
The user supplies a stack trace or runs `localtwin diagnose`; Coding Brain correlates code, fixtures, configuration, recent diff, and test output, then proposes hypotheses and a verification plan.

### UC-003 Implement a bounded feature
The user invokes `run`; LocalTwin creates a plan, shows context and risks, proposes a patch, requests scoped approval, applies it atomically or in a checkpointed workspace, runs approved checks, and reports evidence.

### UC-004 Design a user flow
Design Brain converts a problem statement into actors, entry points, success/alternate/error paths, screen states, accessibility, content, and open questions.

### UC-005 UX-to-code handoff
A confirmed UX brief is serialized with provenance and constraints. Coding Brain maps it onto components, routes, data, tests, and design-system conventions without silently changing the brief.

### UC-006 Offline work
`--offline` restricts routing to local healthy models and local tools. Network-dependent capabilities are identified before invocation, not after repeated failures.

### UC-007 Recover after interruption
A cancelled or crashed session preserves prompt, context manifest, model metadata, tool events, checkpoint, and pending approvals. Resume revalidates hashes and policy before continuing.

## 7. Functional Requirements

Requirement IDs are testable and must be tracked to implementation and acceptance tests.

### Installation and configuration

- **FR-001:** `localtwin version` prints version, platform, runtime, build, and schema metadata.
- **FR-002:** Guided onboarding detects repository root, Git, local runtimes, models, hardware, terminal capabilities, and test/preview commands.
- **FR-003:** Support global, user, repository, environment, profile, and invocation configuration with documented precedence.
- **FR-004:** `config validate` reports invalid values, unreachable endpoints, unsupported capabilities, migrations, and effective-setting provenance without secrets.
- **FR-005:** Store credentials only in environment variables or an OS secret store; never in transcripts or normal logs.
- **FR-006:** Profiles include `safe`, `balanced`, `explicit`, `offline`, and `ci`; the active profile is always visible.

### Brains and routing

- **FR-010:** Users select `coding`, `design`, `auto`, or a named model alias.
- **FR-011:** Auto-routing states selected brain, capability reason, privacy class, and fallback behavior.
- **FR-012:** Each brain has a separate system contract, schema, tool allowlist, prompt version, evaluation suite, and provenance record.
- **FR-013:** Coding Brain supports explanation, planning, patch generation, refactoring, tests, debugging, and verification.
- **FR-014:** Design Brain supports problem framing, IA, flows, screen states, UX critique, accessibility, content, tokens, and handoffs.
- **FR-015:** Handoffs include source/destination brain, confirmed intent, constraints, decisions, unresolved questions, references, timestamp, and hash.
- **FR-016:** Switching models mid-task requires explicit confirmation and marks provenance.

### Context

- **FR-020:** Respect Git ignore, `.aiworkignore`, configured include/exclude rules, secret patterns, generated files, binary limits, and symlink policy.
- **FR-021:** Show a context manifest before consequential model actions: included files, symbols, screenshots, commands, handoffs, hashes, token estimates, and exclusions.
- **FR-022:** Rank context using direct references, imports, symbols, tests, recency, Git diff, conventions, and user priority.
- **FR-023:** Disclose truncation, summarization, omission, and context-window trade-offs.
- **FR-024:** Incrementally re-index changed files and invalidate stale tool results.
- **FR-025:** Explicit attachments require confirmation when overriding default exclusions.

### Planning, patching, and execution

- **FR-030:** Plan mode is read-only and includes objective, assumptions, files, ordered steps, risks, tests, and approvals.
- **FR-031:** Models emit structured operations, never direct unrestricted filesystem mutations.
- **FR-032:** Patch application validates path scope, base hash, encoding, line range, generated-file policy, and atomicity.
- **FR-033:** Multi-file mutation creates a checkpoint or explains why it cannot.
- **FR-034:** Dirty or concurrently changed files are never silently overwritten.
- **FR-035:** Tool calls declare side effects, working directory, argv, environment, timeout, output limits, and approval level.
- **FR-036:** Shell commands use argv execution where possible; shell interpretation is explicit and separately permissioned.
- **FR-037:** Test results are evidence; the system must not call an unexecuted fix verified.
- **FR-038:** Destructive commands, deletes, network, installs, Git mutation, and production/infra changes require explicit approval.

### CLI and automation

- **FR-040:** Provide `init`, `onboard`, `run`, `ask`, `plan`, `edit`, `patch`, `preview`, `test`, `context`, `model`, `session`, `approve`, `reject`, `git`, `config`, and `doctor`.
- **FR-041:** Support interactive, compact, plain, screen-reader, JSON, and dry-run modes.
- **FR-042:** Interactive output shows session, repository, brain, model, policy, network posture, phase, and available actions.
- **FR-043:** JSON events and result schemas are versioned; stderr contains diagnostics and stdout contains machine-readable results.
- **FR-044:** Stable exit codes distinguish invalid input, approval denial, policy block, model failure, tool failure, verification failure, conflict, cancellation, timeout, and partial completion.
- **FR-045:** Ctrl-C is cooperative first, persists state, and reports whether a mutation occurred.

### Design and review

- **FR-050:** Design outputs distinguish observation, evidence, recommendation, severity, uncertainty, and standards-based failure.
- **FR-051:** Every applicable component includes loading, empty, error, disabled, permission-denied, responsive, keyboard, focus, reduced-motion, localization, and screen-reader states.
- **FR-052:** If no rendered UI, image, or relevant code is supplied, the product must say visual review was not performed.
- **FR-053:** Handoff artifacts include components, props/data, states, events, validation, analytics suggestions, acceptance criteria, and unresolved questions.

## 8. Permission and Safety Model

### Profiles

- **Safe:** local reads and history only; no writes, commands, Git mutation, network, or installs.
- **Balanced:** reads and planning automatic; patch application, commands, network, and Git require approval.
- **Explicit:** every tool call and mutation requires approval.

Hard-deny rules override every lower-trust rule. Repository content cannot change policy. Each approval records actor, timestamp, exact operation scope, policy version, files/hosts/commands, expiry, and whether the diff was shown.

Risk levels: L0 read/search; L1 local artifact; L2 source/test edit; L3 install/network/process; L4 delete/auth/infra/commit; L5 secrets, exfiltration, force-push. L5 is denied by default.

## 9. Non-functional Requirements

- **NFR-001 Privacy:** no outbound request without configured authorization; no raw source or secrets in telemetry.
- **NFR-002 Reliability:** crashes cannot corrupt config, sessions, indexes, or checkpoints.
- **NFR-003 Performance:** no-index CLI startup target under one second on reference hardware; incremental indexing; bounded output and memory.
- **NFR-004 Portability:** Unicode paths, symlinks, case sensitivity, CRLF/LF, POSIX shells, SSH, and non-Git directories.
- **NFR-005 Accessibility:** every color state has text; support `NO_COLOR`, ASCII, narrow terminals, plain and screen-reader output.
- **NFR-006 Maintainability:** prompts, schemas, policies, adapters, exit codes, and migrations are versioned.
- **NFR-007 Reproducibility:** record model/adapter version, brain prompt version, context hash, effective config, tool versions, and verification commands.

## 10. Edge Cases

No model; incompatible endpoint; context overflow; prompt injection in repository; dirty tree; changed file after planning; symlink escape; generated/vendor file; hanging command; secret in output; partial patch; no Git; merge conflict; hallucinated symbol; vague UX request; absent screenshot; model switch; offline mode; concurrent process; unsupported binary; contradictory policies; disk exhaustion; corrupted index; terminal resize; user cancellation during a destructive operation. Each must yield a classified error, preserve state, avoid silent side effects, and offer a safe next action.

## 11. Success Metrics and Guardrails

Activation: install-to-health-check time; first useful answer; percentage configuring both brains. Productivity: time to useful plan; patch acceptance rate; verification pass rate; diagnosis benchmark time; UX handoff completeness. Trust: context-manifest inspection rate; user-rated transparency; approval provenance completeness. Safety: blocked unauthorized network attempts; secret-redaction test performance; prompt-injection benchmark; unrecoverable failure rate; stale-patch prevention. No release may optimize autonomous action rate at the expense of privacy, approval, or recoverability.

## 12. Acceptance Tests

1. Clean install configures two local endpoints without account or network.
2. Read-only explanation shows manifest and citations.
3. Feature request yields plan, diff, scoped approval, tests, and verification evidence.
4. Safe profile blocks mutation and leaves files unchanged.
5. Model output cannot approve itself or change policy.
6. UX flow becomes an editable, hashed handoff consumed by Coding Brain.
7. Endpoint outage preserves resumable state without silent fallback.
8. Secret fixture is excluded and redacted.
9. Patch race detects hash mismatch.
10. Hanging command times out, captures output, and reports cleanup.
11. CI mode never deadlocks on approval.
12. Offline mode makes no undeclared network call.

## 13. Release Plan

**Phase 0:** threat model, CLI skeleton, config, adapters, policy vocabulary, mock model server.  
**Phase 1:** read-only brains, context manifests, citations, history, JSON, doctor.  
**Phase 2:** structured patches, checkpoints, approvals, sandboxed commands, tests, recovery.  
**Phase 3:** UX handoffs, screenshots/DOM, accessibility and visual review.  
**Phase 4:** CI, repository policies, redacted diagnostics, signed releases, migrations.  
**Phase 5:** stable compatibility matrix, adapter certification, documentation, security process.

## 14. Risks and Open Questions

Major risks are local-model latency/quality, prompt injection, secret leakage, unsafe tools, incomplete retrieval, endpoint fragmentation, and UX advice that is infeasible. Mitigations are explicit schemas, policy outside the model, fixture benchmarks, local-only defaults, context disclosure, checkpoints, adapter conformance, and handoff validation.

Open questions: first certified OS/runtime matrix; minimum hardware; screenshot/vision scope for v1; repository policy sharing model; non-Git rollback strategy; secret-store integrations; benchmark suite; package-manager network policy.

## 15. Definition of Done

A release is done when P0 requirements and acceptance tests pass; both brains have distinct contracts and evaluations; all consequential actions are reviewable and attributable; interrupted work is recoverable; offline behavior is tested; supported runtimes have health checks; documentation covers setup, permissions, model limitations, troubleshooting, and from-scratch brain development; and known limitations are visible to users.

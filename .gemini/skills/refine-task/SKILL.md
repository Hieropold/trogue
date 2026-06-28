---
name: refine-task
description: >-
  [scaffoldai] Interview the user relentlessly about a GitHub issue or Jira ticket until reaching shared understanding,
  resolving each branch of the decision tree. Use when the user wants to stress-test a task, get grilled on the design,
  or mentions "refine task". Targets GitHub issues by default.
---

# Refine Task

Interview the user relentlessly about every aspect of one task (GitHub issue or Jira ticket) until shared understanding
is reached. Walk down each branch of the design tree and resolve dependencies between decisions one by one. For each
question, provide your recommended answer.

## Refining is refinement-only

**Refining never writes code or repo files.** Do not write, scaffold, or modify any source files, documentation
files, `GLOSSARY.md`, or `docs/adr/` entries during a refining session. The sole purpose of refining is to refine
understanding of the task. Proposed documentation updates are collected during the session and posted verbatim to the
source ticket at the end (step 8); the actual repo files are applied during implementation, never here.

The one permitted write to an external system is posting a comment (and optionally sub-tasks) to the **source ticket
tracker** — done in steps 8–9, only after refining concludes and the user confirms the summary. Never mid-session,
never to repo files.

If the user's message looks like a request to implement, write, or modify code, **stop and ask for explicit
confirmation** before leaving the refining phase. Example: "It sounds like you want to start implementing — should I
exit refining and switch to implementation?" Only proceed after the user explicitly confirms.

## Workflow

1. Resolve the task and gather context:

   **GitHub (default):**
   ```sh
   gh issue view <ISSUE-NUMBER> --comments
   ```

   **Jira (if link provided):**
   ```sh
   acli jira workitem view <ISSUE-KEY> --fields '*all' --json
   ```

2. **Discover the project's domain docs (read-only).** Look for:

   - `GLOSSARY-MAP.md` at the repo root — if present, the repo has multiple bounded contexts; read it to find which
     context the task relates to (see [GLOSSARY-FORMAT.md](./GLOSSARY-FORMAT.md) for the decision tree).
   - A root `GLOSSARY.md` — if present, single context; read it for the glossary.
   - `docs/adr/` — if present, skim the titles to note any relevant prior decisions.

   Record what is missing (no `GLOSSARY.md`, no `docs/adr/`) — these become candidates in the summary.

3. Use the task contents to drive the questioning. Ask questions one at a time. For each question:

   - Before asking, check whether the codebase already answers it — if so, present your finding as the answer and move on without blocking the user.
   - Provide your recommended answer.
   - Wait for the user to respond. The user must do one of three things:
     - **Answer** — accept, correct, or redirect your recommendation.
     - **Push back** — challenge the question or reframe it; resolve the disagreement before continuing.
     - **Explicitly defer** — the user must say something like "defer", "skip for now", or "come back to this". Only then is the question added to the deferred list.
   - **Never silently skip a question.** Every question must be asked and receive one of the three responses above before the session ends.
   - Track all deferred questions in a running list as you go, including only those the user explicitly deferred.

4. **During each question, cross-reference docs and code:**

   - If the user (or the task) uses a term that conflicts with `GLOSSARY.md`, call it out immediately. Example: "The
     glossary defines 'cancellation' as X, but you seem to mean Y — which is it?"
   - If a term is fuzzy or overloaded, propose a precise canonical alternative. Example: "You're saying 'account' — do
     you mean the Customer or the User? Those are different things in the glossary."
   - If the user states how something works, check whether the code agrees. Surface contradictions: "Your code cancels
     entire Orders, but you just said partial cancellation is possible — which is right?"
   - Stress-test domain relationships with concrete edge-case scenarios to force precision about boundaries between
     concepts.
   - **Surface the security and attack-surface dimension.** Ask: does this change touch authn/authz, secrets, PII,
     or data that crosses a trust boundary? Does it introduce new endpoints, queue consumers, or CLI commands — and
     if so, what is their threat model? Does it require new external dependencies, new outbound network calls, or
     new Docker base images? If the task implies any new third-party dependencies, flag now that they will require
     explicit justification at implementation time (see the `implement-feature` Dependency Gate).
   - **Vault secrets**: does the task require new secrets to be provisioned in Vault? If so, what paths will be
     used? New secrets must be referenced as `secret://secret/path/to/key` placeholders in env template files — never as real
     values in source control. Secrets should be manually provisioned by developer in the Vault before implementation starts.
   - **GDPR / PII**: does the change create, modify, or remove storage of personal data? If so, which GDPR
     deletion category applies (account closure, formal deletion, or field redaction), and which team owns that
     deletion flow? Flag if a new PII store is introduced without a documented deletion pathway.
     Record all answers in the "Proposed security considerations" tracking list below.

5. **Track proposed doc updates as you go**, in three running lists:

   - **Proposed GLOSSARY.md additions/edits** — new or refined terms (format per [GLOSSARY-FORMAT.md](./GLOSSARY-FORMAT.md)).
   - **Candidate ADRs** — only when all three hold: hard to reverse, surprising without context, and a real trade-off
     existed. If any criterion is missing, drop the candidate (criteria in [ADR-FORMAT.md](./ADR-FORMAT.md)).
   - **Proposed security considerations** — anticipated new dependencies with rough justification, new trust
     boundaries, PII or secret handling changes, authn/authz changes, new outbound network calls, new Docker base
     images. These will be surfaced verbatim in the "Security Considerations" section of the comment
     refine-task posts to the source ticket (see step 8).

6. Continue until the important product, implementation, and testing decisions are resolved or deferred.

7. After refining ends, present a summary that includes:

   - **Key decisions reached.**
   - **Deferred questions** (omit if none): list each with your recommended answer so the user has context when
     revisiting them.
   - **Proposed GLOSSARY.md updates** (omit if none): new or revised term entries ready to paste in.
   - **Proposed ADRs** (omit if none): title + 1-3 sentence body per [ADR-FORMAT.md](./ADR-FORMAT.md).
   - **Proposed security considerations** (omit if none): anticipated dependencies, trust boundaries, PII/secret handling, authn/authz changes, new outbound calls or Docker images.

8. **Update the source ticket.** After the user confirms the summary, post a comment back to the source ticket.
   There is no separate skill for this — refine-task handles it directly. The comment must include **verbatim**:
   key decisions, deferred questions (if any), proposed GLOSSARY.md updates, proposed ADRs, and a
   "Security Considerations" section (if applicable).

   **GitHub (default):**
   ```sh
   # Write the comment body to a temp file, then post it
   gh issue comment <ISSUE-NUMBER> --body-file <file>
   # Optionally update the issue description with refined scope
   gh issue edit <ISSUE-NUMBER> --body-file <file>
   ```

   **Jira (if a Jira link was provided):**
   ```sh
   acli jira workitem comment add --key <ISSUE-KEY> --body "<comment text>"
   ```

   Note: `GLOSSARY.md` and `docs/adr/` files are written to the repo during implementation, not here — the comment
   only records the proposals so the team can review them.

9. **Create sub-tasks if necessary.** If refining revealed the task splits into multiple independent slices,
   confirm the breakdown with the user, then create one sub-task per slice linked to the source ticket.

   **GitHub:**
   ```sh
   gh issue create --title "<slice title>" --body "Parent: #<ISSUE-NUMBER>\n\n<slice description>"
   ```

   **Jira:**
   ```sh
   acli jira workitem create --type Sub-task --parent <ISSUE-KEY> --summary "<slice summary>"
   ```

10. **Recommend what should happen next** based on the outcome:

    - **Deferred questions exist** (most common): the ticket now carries the deferred list for the team to review.
      After the team has answered them, run `refine-deferred` to resolve the remaining open points before
      implementation.
    - **No deferred questions**: go straight to implementation (hand off to `implement-feature`).
    - Always ask the user to confirm which path to take.

## References

- [ADR-FORMAT.md](./ADR-FORMAT.md) — when and how to propose architecture decision records
- [GLOSSARY-FORMAT.md](./GLOSSARY-FORMAT.md) — glossary structure and single-vs-multi-context layout

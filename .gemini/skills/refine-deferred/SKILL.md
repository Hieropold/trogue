---
name: refine-deferred
description: >-
  [scaffoldai] Resolve a deferred-questions list on a GitHub issue or Jira ticket. Presents each
  deferred question to the user one by one, collects answers, then updates the ticket with the
  resolved answers and prunes the deferred list. Use when the team has answered deferred questions
  and the user wants to continue refining, or when refine-task ends with unresolved deferred
  questions. Targets GitHub issues by default.
---

# Refine Deferred

Walk through every deferred question on a source ticket that was left open by a previous
`refine-task` session. For each question, provide your recommended answer, collect the user's
response, then update the ticket with the answers and prune the deferred list.

## Resolving deferred is refinement-only

**Resolving deferred questions never writes code or repo files.** Do not write, scaffold, or modify
any source files, documentation files, `GLOSSARY.md`, or `docs/adr/` entries during this session.
Proposed documentation updates are collected during the session and posted verbatim to the source
ticket at the end (step 5); the actual repo files are applied during implementation, never here.

The one permitted write to an external system is updating the source ticket — posting a comment
with the answers and editing the deferred-questions list — done in step 5, only after all questions
are presented and the user confirms. Never mid-session, never to repo files.

If the user's message looks like a request to implement, write, or modify code, **stop and ask for
explicit confirmation** before leaving this phase. Example: "It sounds like you want to start
implementing — should I exit and switch to implementation?" Only proceed after the user explicitly
confirms.

## Workflow

1. Resolve the ticket and locate the deferred list:

   **GitHub (default):**
   ```sh
   gh issue view <ISSUE-NUMBER> --comments
   ```

   **Jira (if link provided):**
   ```sh
   acli jira workitem view <ISSUE-KEY> --fields '*all' --json
   ```

   Look for a "Deferred questions" section in the issue body or a comment posted by a previous
   `refine-task` session. If no deferred list exists, tell the user there is nothing to resolve
   and stop — do not continue.

2. **Discover the project's domain docs (read-only).** Look for:

   - `GLOSSARY-MAP.md` at the repo root — if present, the repo has multiple bounded contexts; read
     it to find which context the task relates to.
   - A root `GLOSSARY.md` — if present, single context; read it for the glossary.
   - `docs/adr/` — if present, skim the titles to note any relevant prior decisions.

3. **Present each deferred question one at a time.** For each question:

   - Before asking, check whether the codebase or docs already answer it — if so, present your
     finding as the recommended answer and move on without blocking the user.
   - Provide your recommended answer.
   - Wait for the user to respond. The user must do one of three things:
     - **Answer** — accept, correct, or redirect your recommendation; record as resolved.
     - **Push back** — challenge the question or reframe it; resolve the disagreement before
       continuing.
     - **Explicitly defer again** — the user must say something like "defer", "skip for now", or
       "come back to this". Only then is the question kept on the deferred list.
   - **Never silently skip a question.** Every question must receive one of the three responses
     above before the session ends.
   - Track two running lists as you go: **resolved** (question + agreed answer) and
     **still deferred** (questions the user explicitly deferred again).

4. **While presenting each question, cross-reference docs and code:**

   - Flag glossary conflicts immediately if the answer uses a term differently from `GLOSSARY.md`.
   - Surface any security or PII implications the answer introduces (authn/authz changes, new
     secrets or PII stores, new dependencies or outbound calls). Collect these under a "Proposed
     security considerations" list to include verbatim in the ticket update.
   - If the answer introduces a new term or a decision that warrants an ADR, collect it under
     "Proposed GLOSSARY.md updates" / "Candidate ADRs" — these will be posted to the ticket and
     applied to repo files during implementation, not here.

5. **Update the source ticket.** After the user confirms, post a comment and update the deferred
   list. There is no separate skill for this — refine-deferred handles it directly.

   **Comment** (both GitHub and Jira): include verbatim — resolved question→answer pairs, proposed
   GLOSSARY.md updates (if any), proposed ADRs (if any), proposed security considerations (if any).

   **GitHub (default):**
   ```sh
   # Post the answers comment
   gh issue comment <ISSUE-NUMBER> --body-file <answers-file>

   # If the deferred list lives in the issue body: rewrite it with remaining questions only,
   # or remove the section entirely if none remain
   gh issue edit <ISSUE-NUMBER> --body-file <updated-body-file>
   ```

   **Jira (if a Jira link was provided):**
   ```sh
   # Post the answers comment
   acli jira workitem comment add --key <ISSUE-KEY> --body "<answers text>"

   # Update the description with the pruned deferred list (or remove it if fully resolved)
   acli jira workitem update --key <ISSUE-KEY> --description "<updated description>"
   ```

   **Pruning rules:**
   - Remove every answered question from the "Deferred questions" list.
   - Keep every question the user explicitly deferred again.
   - If no questions remain, remove the "Deferred questions" section entirely and state it is fully
     resolved in the comment.

   Note: `GLOSSARY.md` and `docs/adr/` files are written to the repo during implementation, not
   here — the comment only records the proposals so the team can review them.

6. **Recommend what should happen next** based on the outcome:

   - **Questions still deferred**: the ticket now carries the updated deferred list for the team to
     review. Run `refine-deferred` again once the team has answered them.
   - **No questions remaining**: go straight to implementation (hand off to `implement-feature`).
   - Always ask the user to confirm which path to take.

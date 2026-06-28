---
name: implement-feature
description: "[scaffoldai] Implement a feature end-to-end using a tracer-bullet TDD loop while treating the feature source as read-only context. The source can be a Jira ticket, Github issue, file with specs, or plain prompt text. Use when the user wants a feature built, mentions an issue key, specifies a spec file, or provides a text prompt description."
---

# Implement Feature

Use this when the goal is to implement one feature, not to redesign the roadmap. The source of the feature (Jira ticket, Github issue, file with specs, or plain prompt text) is read-only in this skill: fetch or read the source context, then implement locally.

## Core Rules

- Work on exactly one feature from a specified source (Jira ticket, Github issue, file with specs, or plain prompt text) per invocation.
- Prefer the highest-priority unblocked ticket, issue, spec, or task if the user did not specify one.
- Keep the change as small as possible: bug fix first, then tracer bullet, then polish, then refactor.
- Use `RED -> GREEN -> REPEAT -> REFACTOR`.
- Tests must verify externally visible behavior through public interfaces, not implementation details.
- Do not assign, transition, edit, or comment on the source ticket or issue.
- Treat comments/descriptions as context only. Do not attempt to clean them up or publish back.

## Intake

1. Resolve the source and gather context:
   - **Jira ticket**:
     ```sh
     acli jira workitem view <ISSUE-KEY> --fields '*all' --json
     acli jira workitem comment list --key <ISSUE-KEY>
     ```
   - **Github issue**:
     Use appropriate GitHub CLI commands or tools to view the issue and its comments (e.g. `gh issue view <ISSUE-NUMBER> --comments`).
   - **File with specs**:
     Read the spec file's contents from the workspace directory.
   - **Plain prompt text**:
     Read the feature description provided directly in the prompt text.

   If the source references a parent, PRD, epic, or blocked-by item that matters for implementation, fetch that too.

2. Read the relevant code and tests before editing anything. Use the repo's domain language, docs, and ADRs.

3. Decide the smallest useful slice:
   - Prefer one demoable end-to-end behavior.
   - Avoid horizontal work like "backend only" unless the source explicitly scopes it that way.
   - If the source is too vague or contains unresolved product choices, stop and clarify.

## Execution Loop

4. Plan the first tracer bullet:
   - Name the first behavior to prove.
   - Identify the public interface to exercise.
   - Pick the narrowest failing test that would prove the path works.

5. Run the loop one behavior at a time:

   ```text
   RED: write one failing test for one behavior
   GREEN: write the minimum code to pass it
   REPEAT: add the next behavior only after green
   REFACTOR: clean up only while tests are green
   ```

6. Guardrails during execution:
   - Do not write all tests first.
   - Do not add speculative features.
   - Do not mock internals when a real integration-style path is practical.
   - If the feature expands into multiple independent slices, stop and discuss/create subtasks instead of quietly broadening scope.

## Continuous Instruction Feedback Loop

After **every** tool call or shell command, check whether the outcome revealed a wrong assumption about this repo's
conventions. If it did, and you subsequently found the correct approach, update the instruction file (like CLAUDE.md, AGENTS.md, or GEMINI.md) **immediately** —
do not accumulate these for the end of the session.

### When to trigger

Trigger this check after any step that:

- Returned a non-zero exit code or error message.
- Required more than one attempt to find the canonical way to do something (build, test, typecheck, publish, run
  migrations, authenticate, etc.).
- Succeeded only after discovering the correct binary, path, env var, command, or file location through trial and error.

Do **not** trigger for:

- Genuine bugs in the user's own code (compile errors, failing assertions, logic errors).
- Transient errors (network timeouts, missing credentials the user must supply).
- Errors that an experienced developer would diagnose instantly from general knowledge.

### Filter before proposing

Before proposing any update, apply both of the following tests. Drop the lesson if either test fails:

1. **Repo-specific**: The lesson is about this repo's conventions or toolchain — not general programming knowledge.
2. **First-move-changing**: A one-line note in the instructions file would have changed the agent's **first move**, not
   just corrected it after the fact.

### Inline update workflow

When both tests pass:

1. Find the project's instruction file (`CLAUDE.md`, `AGENTS.md`, or `GEMINI.md` — whichever exists at the repo root).
2. Identify the existing section the snippet belongs under. Prefer inserting into an existing section over adding a new
   heading.
3. Draft a snippet of one to three lines, imperative, present-tense (e.g. `Use yarn test, not npm test.`).
4. Present the proposed change in chat as a diff-style preview:
   ```
   File:    CLAUDE.md
   Section: ## Running commands
   Add:
     > Use `yarn test` — running `npm test` inside a package directory will not find the test runner.
   ```
5. **Wait for explicit user approval before editing.** If the user rejects it, drop it silently. If they ask to
   rephrase, revise and re-present.
6. After approval, apply the edit with the `Edit` tool. Do not commit, push, or stage.

### Hard rules

- Never invent a lesson that did not arise from an actual failure in this session.
- Never duplicate guidance already present in the existing instructions.
- Never edit any file before the user approves the proposed diff.
- Only the instruction file(s) may be edited — no source files, no config.

## Verify And Close

7. Run the project's relevant verification commands before declaring success. At minimum, run the focused tests you
   changed; also run broader test or typecheck commands when the repo norms require them.

8. If blocked, explain the blocker in the final response instead of changing the source. Include what you tried, the
   exact blocker, and what decision or upstream change is needed.

9. If complete, summarize the delivered behavior, tests added or updated, key implementation decisions, and verification
   commands in the final response. Do not transition or comment on the source ticket or issue from this skill.

## Per-Cycle Checklist

```text
[ ] Working on one feature/ticket/issue/spec/prompt only
[ ] Current test proves behavior, not implementation
[ ] Code is minimal for the current failing test
[ ] No speculative scope added
[ ] Source was used as read-only context only
[ ] After each failed command: feedback loop check done (repo-specific + first-move-changing)
[ ] Any approved instruction updates applied immediately, not deferred to end of session
```

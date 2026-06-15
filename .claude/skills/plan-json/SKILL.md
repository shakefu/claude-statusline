---
name: plan-json
description: JSON task-tracking protocol used by this repo. Defines the plan schema, session bootstrap, in-progress checkpoint format, and archive flow for `context/current-task.json` and `context/<plan>.json`. Invoke at the start of any session that will work on planned tasks, or before reading/updating those files. Triggers include "next task", "current task", "task loop", "resume task", "archive plan", or any reference to `context/current-task.json`.
user-invocable: true
disable-model-invocation: false
---

# Plan JSON Protocol

Multi-task work is tracked as JSON at `context/<plan>.json`, with the active plan pointed at by `context/current-task.json`. JSON only — no YAML, no markdown task lists.

## Session bootstrap

A fresh session has no memory. Run this checklist before executing work:

1. `git status`, `git stash list`, check unpushed commits. Ask before touching pending changes.
2. If starting new work, branch off latest `main` as `<type>/<description>-<session-id>` (`<type>` = conventional-commit type).
3. Kick off the project test command in the background (skip for docs/context-only work). See `CLAUDE.md` / `README.md` for the invocation.
4. Read `context/current-task.json` for the active task and plan path.
5. `git log --oneline -5` for recent history.
6. Pick the first task with `status: "pending"` and `blocked_by: null`. Execute. Update its status.

After edits, use `BashOutput` against the background test run to verify.

## Key files

- `context/current-task.json` — pointer to active task + plan file.
- `context/<plan>.json` — the plan itself (schema below).
- `context/completed/` — archived plans.

## Plan schema

```json
{
  "plan_name": "Feature or Project Name",
  "last_updated": "YYYY-MM-DD",
  "tasks": [
    {
      "id": "task-id-kebab-case",
      "name": "Human readable task name",
      "status": "pending",
      "priority": 1,
      "blocked_by": null,
      "output_file": "path/to/output",
      "steps": ["Concrete step 1", "Concrete step 2"],
      "acceptance_criteria": ["Verifiable check 1", "Verifiable check 2"]
    }
  ],
  "notes": ["Free-form reminders"]
}
```

Fields:

- `status` — `pending` | `in_progress` | `complete`.
- `priority` — integer; lower runs first when nothing else differentiates.
- `blocked_by` — id of a prerequisite task, or `null`.
- `output_file` — optional artifact path.
- `steps[]` — concrete actions in execution order.
- `acceptance_criteria[]` — checks that prove completion.

Tasks with `blocked_by: null` can in principle run in parallel.

## Shutdown checkpoint

Always update `context/current-task.json` before the session ends — including mid-task. A `/clear`-fresh session must resume without asking "where were we?". Treat the checkpoint update as a required step, not optional cleanup.

### Task complete

- Advance `current_task_id` to the next task per `blocked_by`.
- Set `status: "pending"`.
- Optionally record `previous_task` (commit hash + one-paragraph summary).

### Task in progress

- Keep `current_task_id` at the active task.
- Set `status: "in_progress"`.
- Record a `progress` block:
  - `completed_steps[]` — what's done, with file paths, line ranges, commit hashes, command outputs — enough that a fresh session can verify.
  - `next_step` — the single concrete next action, executable without conversation context.
  - `verification_commands[]` — commands to confirm working-tree state matches the checkpoint.
- Explicitly call out any uncommitted changes. The next session must verify the diff before continuing so work isn't accidentally discarded.

## Archiving a finished plan

When every task is complete:

1. `git mv context/<plan>.json context/completed/`
2. Update `current-task.json` to point at the next plan, or clear it.
3. Commit: `chore(context): archive completed <plan-name>`.

## Completion criteria

- End-to-end tests exist and pass — unit tests alone are insufficient.
- Each acceptance criterion explicitly checked.
- Test command actually run, not assumed.
- **User-facing docs updated in the same change whenever code changes user-facing behaviour** (a new/changed command, endpoint, request/response shape, error, or flag). When you modify code, check for doc drift: the OpenAPI/Swagger `doc:`/`example:` tags, the `huma.Operation` Summary/Description, and any README / API docs must still match the code. A doc that disagrees with the shipped code is incomplete work.

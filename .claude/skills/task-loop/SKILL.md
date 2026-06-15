---
name: task-loop
description: Drive the task-completion loop end-to-end. Creates a one-off agent team and, per cycle, spawns a fresh teammate to run "next task" against context/current-task.json, then explicitly messages that teammate via SendMessage to update current-task.json before the cycle ends — so the bookkeeping step never gets skipped on long tasks. Use when the user says "/task-loop", "drive the task loop", "run next-task in a loop", or asks to autonomously work through pending tasks in their plan. Optional argument: max cycles (integer). Requires CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1.
user-invocable: true
disable-model-invocation: false
---

# Task Loop

Automates the user's manual rhythm of: start session → `next task` → wait → update `current-task.json` → `/clear` → repeat.

## Load the task protocol first

Before anything else — including the prerequisite check below — invoke the `/plan-json` skill. It defines the schema for `context/current-task.json` and `context/<plan>.json`, the session bootstrap, the in-progress checkpoint format, and the archive flow this loop drives. Without it loaded, every step below is operating on assumptions.

Each cycle spawns a **fresh teammate** (brand-new context — `/clear` semantics for the work). After the teammate reports back, the orchestrator uses `SendMessage` to drive the JSON-bookkeeping turn on that same teammate (which still has full context of what it just did). This eliminates the "teammate forgot to update the task JSON" failure mode the user reliably hits on long, significant tasks. The teammate is then shut down so idle workers don't accumulate, and the next cycle spawns another fresh one.

## Hard requirement

This skill requires Claude Code's experimental agent-teams feature, which provides `TeamCreate`, `TeamDelete`, and `SendMessage`. Set in user settings:

```json
{ "env": { "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1" } }
```

If `SendMessage` is not available in the current session, abort with a clear message pointing the user at this requirement. Do NOT silently fall back to a single-prompt worker — that's the failure mode this skill exists to fix.

## Arguments

- Optional: integer max-cycle count. `/task-loop 5` → run at most 5 cycles. No argument → run until no pending unblocked task remains.

## Prerequisites — check before starting

1. `context/current-task.json` exists and points at an active plan.
2. The active plan has at least one task with `status: "pending"` and no unsatisfied `blocked_by`.
3. The `/plan-json` skill is available in this project (the teammates will be told to invoke it).
4. The current session has no active team (one team per session is the documented limit). If a team exists, abort and tell the user to clean it up first.

If any prerequisite fails, report what's missing and STOP. Do not start the loop.

## Setup — once, before the first cycle

Call `TeamCreate` with a unique team name. Use `task-loop-<unix-timestamp>` so reruns can't collide with stale config:

```
TeamCreate({
  team_name: "task-loop-<unix-timestamp>",
  description: "Driving plan-allium cycles via /plan-json",
  agent_type: "team-lead"
})
```

Remember the team name — every `Agent` spawn and `SendMessage` below uses it.

## Worker prompt prefix — brevity

Workers spawned via `Agent` do not inherit the orchestrator's system prompt. By default they produce verbose, narrative output the orchestrator never re-reads (`git log`, `git status`, and `context/current-task.json` are read directly).

Two failure modes to head off:

1. The worker writes a long "here's what I did" summary as its final assistant message — wasting attention.
2. The worker writes `done` as its final assistant message — which goes nowhere, because **plain assistant output does NOT cross the team boundary**. Only `SendMessage` does.

So every worker prompt (spawn `prompt` and substantive `SendMessage`) must prepend this preamble:

```
Reporting style: On success, call SendMessage({to: "team-lead", message: "done"}) and nothing else. On failure or blocker, call SendMessage({to: "team-lead", message: "BLOCKED: <one-line reason>"}). Your plain assistant output does NOT reach the orchestrator — only SendMessage does. Never produce a summary, status report, recap, or narration.
```

For bookkeeping or other turns where the worker must report a value (commit hash, file path), swap the `done` literal for the value the orchestrator needs — keep the rest of the preamble verbatim.

Workers that themselves spawn sub-`Agent`s (e.g. phase-3 reviewer fan-out) must propagate the same preamble into the sub-Agent prompts, or the reviewers' verbose output will swamp the worker's context window. Mention this explicitly in any prompt that may trigger sub-spawns.

Shutdown messages do not need the preamble.

## Loop body

Repeat until a stop condition is hit. Track an integer cycle counter `N` starting at 1.

### 1. Spawn a fresh teammate

```
Agent({
  subagent_type: "general-purpose",
  team_name: "<chosen team name>",
  name: "worker-<N>",
  description: "Run next task",
  prompt: "Reporting style: On success, call SendMessage({to: \"team-lead\", message: \"done\"}) and nothing else. On failure or blocker, call SendMessage({to: \"team-lead\", message: \"BLOCKED: <one-line reason>\"}). Your plain assistant output does NOT reach the orchestrator — only SendMessage does. Never produce a summary, status report, recap, or narration. If you spawn sub-Agents, prepend this same preamble to their prompts.\n\nFirst, invoke the /plan-json skill to load this repo's task protocol (schema for context/current-task.json, session bootstrap, checkpoint format, archive flow). Then: work the next task. Read context/current-task.json, find the first task with status \"pending\" and blocked_by null, execute it, and update its status per the /plan-json protocol."
})
```

Brand-new context. The teammate loads `/plan-json` itself to learn the protocol (read `context/current-task.json`, find first unblocked pending task, execute, update its `status`). This mirrors a `/clear`-ed session.

### 1b. Set a watchdog timer

Immediately after spawning, call `ScheduleWakeup` with `delaySeconds: 270` (under the 5-minute prompt-cache TTL). The prompt should instruct you to check on the current worker: read `context/current-task.json`, run `git status` and `git log --oneline -3`, and if the worker appears stuck (no new commits or file changes), send it a message to continue. If it finished, proceed to steps 2–5 as normal.

```
ScheduleWakeup({
  delaySeconds: 270,
  reason: "Check if worker-<N> (<task-name>) is stuck or needs a nudge",
  prompt: "Check on worker-<N> for the task-loop. Read context/current-task.json and git log --oneline -3 to see if progress was made on the <task-id> task. If the worker seems stuck (no new commits, no file changes), send it a message to continue. If it finished, drive the bookkeeping turn (update current-task.json), shut it down, and spawn worker-<N+1> for the next task."
})
```

This ensures workers don't silently stall between idle notifications. The timer is a safety net — you should still react immediately to teammate messages when they arrive.

### 2. Surface the result

When the teammate returns, output one or two lines to the user: which task it worked on and whether it claims success. No long summaries mid-loop.

### 3. Drive the bookkeeping turn (the whole point of this skill)

`SendMessage` to that teammate by name — NOT a fresh `Agent` — because the teammate still has full context of what it just did:

```
SendMessage({
  to: "worker-<N>",
  summary: "update current-task.json",
  message: "Reporting style: On success, call SendMessage({to: \"team-lead\", message: \"<bookkeeping-commit-hash>\"}) and nothing else. On failure or blocker, call SendMessage({to: \"team-lead\", message: \"BLOCKED: <one-line reason>\"}). Your plain assistant output does NOT reach the orchestrator — only SendMessage does. Never produce a summary, status report, recap, or narration.\n\nUpdate context/current-task.json per the /plan-json protocol you already loaded, so a fresh session can pick up the next task cleanly. Specifically: ensure the task you just completed has status \"complete\", update last_updated to today's date, and confirm the next pending unblocked task is correctly represented. If every task in the plan is now complete, follow the /plan-json archiving flow: git mv the plan into context/completed/, update or clear current-task.json, and prepare a chore commit message."
})
```

The teammate's reply will arrive automatically as a new turn — do not poll.

### 4. Decide whether to continue

Re-read `context/current-task.json` directly (do not trust the teammate's claim — the docs explicitly note teammates sometimes lag on task status). Continue if BOTH:

- At least one task with `status: "pending"` and no unsatisfied `blocked_by` exists.
- Cycle counter `N` has not reached the user-supplied limit (if any).

Otherwise stop and proceed to **Cleanup**.

### 5. Shut down the teammate before the next cycle

To avoid accumulating idle teammates across cycles:

```
SendMessage({
  to: "worker-<N>",
  message: { type: "shutdown_request", reason: "cycle complete" }
})
```

Wait for the teammate to terminate (it goes idle/gone). Then increment `N` and loop back to step 1.

## Stop conditions

Stop when ANY of these hold. The first three are normal completion; the fourth is a hard halt.

- No pending unblocked task remains.
- The cycle limit has been reached.
- The plan was archived (active plan finished).
- A teammate reported a hard error (failing tests, missing context, blocked acceptance criteria). Do NOT silently retry. Report what failed and stop.

## Cleanup

Always run cleanup before the final report — even on hard-halt error paths:

1. For any teammate still alive, `SendMessage({to: "<name>", message: {type: "shutdown_request"}})` and wait.
2. `TeamDelete` to remove `~/.claude/teams/<team-name>/` and `~/.claude/tasks/<team-name>/`.

If `TeamDelete` fails because a teammate is still active, retry the shutdown for that teammate and try again. The docs warn that teammates only shut down between turns, so this can take a moment.

## Final report

After cleanup, output:

- Cycles run.
- Tasks completed this run (by id/name).
- Current plan status: in-progress (with next pending task) / complete-and-archived / halted-on-error.
- Any teammate errors verbatim.

## What this skill does NOT do

- It does NOT parallelise. The `/plan-json` protocol notes `blocked_by: null` tasks could in principle run in parallel; this skill stays strictly sequential to match the user's mental model. Parallel runs are a separate ask.
- It does NOT use the team's shared TaskList (`TaskCreate`/`TaskUpdate`) for plan tracking. Source of truth stays `context/current-task.json` per `/plan-json`. The team mechanism here is purely a transport for spawn-and-message — not a replacement for the JSON plan.
- It does NOT commit, push, or open PRs. Bookkeeping = JSON updates only. Teammates may commit as part of their task work — that's their call.
- It does NOT skip the prerequisite check. Even when the user is impatient.
- It does NOT reuse a single long-lived teammate across cycles. That would defeat the `/clear`-per-cycle property the user is explicitly relying on.

---
name: project-handoff-bootstrap
category: software-development
version: 1.0.0
description: |
  Bootstrap a new project folder as a complete session-bridging handoff
  package: CONTEXT.md (origin story + research + decisions + stakes),
  HANDOFF.md (implementation spec), git init with first commit, and a
  ready-to-paste opening prompt for the next agent session. Use when a
  conversation produces a new project that the user wants to physically
  move into its own folder and resume later in a fresh chat or different
  agent (Hermes, Claude Code, Codex, etc.).
---

# Project Handoff Bootstrap

## When to use

Trigger this skill when ALL of the following are true:

1. The current conversation has produced a concrete new project worth preserving
2. The user has created (or asks you to create) a dedicated folder for it
3. The work will continue in a future session — possibly in a different agent
4. Losing the conversation context would force the user to re-explain everything

Typical phrases that trigger it:
- "Write a handoff in there"
- "I'll move this chat over"
- "Can we resume this tomorrow?"
- "Start a git folder for this project"
- "Give me a prompt for the new chat"

## Why this pattern exists

Sessions die. Memory is finite. Honcho doesn't auto-capture project intent.
If you only verbally acknowledge a new project, the next agent that opens
the folder sees an empty directory and has zero context. The user has to
re-explain the entire backstory, lose research findings, and reconstruct
decisions from scratch.

The fix: leave a complete, self-contained handoff package on disk so any
future agent (or the user months later) can `cat` two files and be fully
caught up in under 2 minutes.

## The package (4 artifacts)

### 1. CONTEXT.md — the why
Captures everything that won't fit in code:
- The trigger (what the user said that started this, ideally verbatim)
- Research findings with citations (if research was done)
- The user's idea in their own words (verbatim quotes matter)
- Decisions made + reasoning (table format works well)
- Open questions deferred to the user
- Real stakes (personal, business, deadline) — this is critical, see Pitfall #1
- File map + how to resume in any future agent
- Why it matters in the bigger picture

### 2. HANDOFF.md — the what + how
The implementation spec:
- Problem statement
- Hard requirements
- Architecture (with diagrams in code blocks)
- Component/CLI surface
- Storage/data model
- Phased implementation plan with time estimates
- Config file examples
- Non-goals (resist scope creep)
- Success criteria (measurable)
- Open questions for the user
- Next action (the prompt to spawn an implementing agent later)

### 3. Git initialization
```bash
cd <project-folder>
git init -b main
git add -A
git commit -m "Initial: HANDOFF + CONTEXT for <project>"
```
Always use `-b main`. Always include both files in the first commit.

### 4. Opening prompt for the next session
A copy-paste ready prompt the user can drop into a fresh chat. Must:
- Point at both files in order (CONTEXT.md first, HANDOFF.md second)
- Re-state the stakes so the next agent doesn't soften them
- Specify the exact next action (don't leave it open)
- Include the user's own framing/voice if they used a memorable phrase

## Workflow

1. **Verify the folder exists.** Run `ls -la <path>` to confirm and check
   ownership. If it doesn't exist, ask before creating.

2. **Write CONTEXT.md first.** This is harder than HANDOFF.md because it
   requires summarizing the conversation. Quote the user verbatim where
   possible — their exact phrasing carries the conviction that a paraphrase
   loses.

3. **Write HANDOFF.md second.** Draw on any research, architecture
   discussions, and decisions from the conversation. Be specific:
   exact file paths, exact CLI commands, exact schemas. Vague handoffs
   produce vague implementations.

4. **Git init + first commit.** One command chain. Verify with `git log
   --oneline` so the user sees the commit landed.

5. **Generate the opening prompt LAST.** Now that both docs exist, the
   prompt can reference them with confidence. Format as a fenced code
   block so the user can copy-paste cleanly.

6. **Offer follow-ups.** Common ones:
   - `.gitignore` for the chosen language
   - README skeleton
   - Honcho conclusion to remember the project exists

## CRITICAL: handle scope changes immediately

The most common failure mode in this skill: the user reveals new stakes
or scope partway through, and the agent only verbally acknowledges instead
of updating the artifacts.

**Wrong:** "Got it, that changes everything. Tomorrow we'll plan it bigger."

**Right:** Patch CONTEXT.md and HANDOFF.md immediately to reflect the new
scope. Add a dated section (e.g. "## §9 The real stakes — added <date>").
Update HANDOFF.md status header if the work is now blocked on something.
Commit the changes. THEN respond.

If the user reveals personal stakes (jobless, deadline, career bet, family
situation), capture them in CONTEXT.md verbatim. Future agents need the
emotional weight, not just the technical scope. It changes how they
prioritize and how strict they are about quality.

## Pitfalls

1. **Don't bury the stakes.** If the user said something raw like "I'm
   jobless and this is my bet before I have to go back to a 16-hour
   kitchen", quote it directly in CONTEXT.md. Sanitizing it loses the
   reason the project exists.

2. **Don't forget verbatim quotes.** When the user proposes the
   architecture in their own words, copy the quote into CONTEXT.md. It's
   the highest-fidelity capture of their intent.

3. **Don't skip git init.** Even for a 2-file project. Without git,
   future edits have no history and rollback is impossible.

4. **Don't write the opening prompt before the docs.** The prompt must
   reference real file content. Write it last so it's accurate.

5. **Don't make the opening prompt vague.** "Continue working on skilr"
   is bad. "Read CONTEXT.md and HANDOFF.md, confirm the stakes, then run
   /office-hours and save artifacts to ~/.gstack/projects/skilr/" is good.

6. **Don't index the wrong path.** Always confirm the user's intended
   project folder with `ls -la` before writing. Path typos in handoff
   docs are painful.

7. **Don't paste raw read_file output anywhere.** The "LINE_NUM|" prefix
   leaks into Telegram and markdown. Read carefully, write cleanly.

8. **Don't recommend a final architecture before knowing the stakes.**
   The "personal tool vs viral OSS launch" decision flips language
   choice, planning depth, and quality bar entirely. Ask about scope
   BEFORE recommending tech stack if it's not obvious.

## Verification

After running this skill, verify:
- Both files exist and are >2KB each (smaller usually means too vague)
- Git log shows the initial commit
- The opening prompt is in a fenced code block in your final message
- CONTEXT.md contains at least one verbatim user quote
- HANDOFF.md has a "Next action" section with a runnable command or prompt

## Example structure

```
~/Projects/<project>/
├── .git/
├── CONTEXT.md     ← origin, research, decisions, stakes (5-10 KB)
└── HANDOFF.md     ← spec, architecture, phases, next action (7-15 KB)
```

Future:
```
~/Projects/<project>/
├── .git/
├── CONTEXT.md
├── HANDOFF.md
├── README.md      ← added when implementation starts
├── .gitignore
└── src/
```

## Real-world example

Used to bootstrap `~/Projects/skilr/` (a local skills router CLI) on
2026-04-07. The conversation that produced it:
1. Token bloat complaint → research → architecture proposal → folder
2. CONTEXT.md captured the trigger, last30days + delegate_task research
   findings, the user's verbatim architecture proposal, decisions table
3. HANDOFF.md captured the full spec (SQLite + FTS5 + fastembed, hybrid
   search, CLI surface, 5-phase plan)
4. Git initialized, first commit landed
5. User then revealed real stakes mid-session ("jobless, viral OSS bet")
   → both docs were patched immediately with a new dated section, second
   commit landed
6. Opening prompt for next session generated last, referencing both files

The pattern survived the session boundary: a fresh agent reading those
two files can fully resume the work.

## Completion status

- DONE — package complete, git committed, opening prompt delivered
- DONE_WITH_CONCERNS — package written but user hasn't confirmed paths
- BLOCKED — folder doesn't exist or path is ambiguous
- NEEDS_CONTEXT — research or stakes still missing from the conversation

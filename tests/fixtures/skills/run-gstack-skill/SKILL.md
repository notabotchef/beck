---
name: run-gstack-skill
category: devops
version: 1.0.0
description: |
  Run any gstack skill (from ~/.claude/skills/gstack/) as an AI agent following its workflow step by step.
  Gstack skills are just markdown prompt files — any AI can follow them.
  Use when asked to run gstack skills in a non-Claude Code environment.
---

## Key Insight

Gstack skills are markdown files with YAML frontmatter. They describe workflows, ask questions via AskUserQuestion, and produce artifacts. They do NOT require Claude Code specifically. Hermes (or any AI agent) can run them by:

1. Reading the skill file from `~/.claude/skills/gstack/[skill-name]/SKILL.md`
2. Executing the preamble bash scripts
3. Following the phases step by step
4. Presenting questions to the user when the skill calls for AskUserQuestion
5. Producing the documented artifacts (design docs, review reports, etc.)
6. Running telemetry commands at the end

## How to Run a Gstack Skill

### Step 1: Load the Skill

Read the SKILL.md file from `~/.claude/skills/gstack/[skill-name]/SKILL.md`.

Available skills:
- office-hours
- plan-ceo-review
- plan-eng-review
- autoplan
- design-consultation
- design-review
- review
- investigate
- qa
- ship
- land-and-deploy
- canary
- document-release
- checkpoint
- carefree
- guard
- learn
- health

### Step 2: Execute the Preamble

Run the preamble section to get:
- `_BRANCH` (current git branch)
- `REPO_MODE`
- `LAKE_INTRO`
- `TEL_PROMPTED`
- `PROACTIVE_PROMPTED`
- `HAS_ROUTING`
- `SPAWNED_SESSION`

### Step 3: Follow the Phases

Gstack skills are structured in numbered phases:
- Phase 1: Context Gathering
- Phase 2: Discovery (Startup Mode or Builder Mode)
- Phase 3: Premise Challenge
- Phase 4: Alternatives Generation
- Phase 5: Design Doc (office-hours) or Review Report
- Phase 6: Handoff

Execute each phase in order. Ask questions when the skill calls for AskUserQuestion. Wait for user responses.

### Step 4: Produce Artifacts

Each skill produces specific artifacts:
- office-hours → Design doc saved to `~/.gstack/projects/{slug}/{user}-{branch}-design-{datetime}.md`
- plan-ceo-review → CEO review report
- review → Staff-level code review with auto-fixes
- qa → Browser-based QA with bug reports
- document-release → Updated documentation files

### Step 5: Spec Review Loop

Many skills include a spec review phase that dispatches an adversarial reviewer. Use delegate_task with a review prompt to simulate this.

### Step 6: Telemetry

Run the telemetry bash at the end to log skill execution.

## Proven Workflow Patterns

### AskUserQuestion Mapping
Claude Code's AskUserQuestion → use `clarify()` with choices. Always:
1. Re-ground user in project context (branch, current task)
2. Simplify the problem to plain English
3. Give a recommendation with reasoning
4. Present lettered options (A, B, C)

### Spec Review Loop
Use `delegate_task()` with adversarial review prompt to simulate:
- Read the doc being reviewed
- Review on 5 dimensions: Completeness, Consistency, Clarity, Scope, Feasibility
- Return quality score 1-10 + specific issues
- Fix issues, re-dispatch (max 3 iterations)

### Outside Voice (Plan Reviews)
Use `delegate_task()` for independent second opinion:
- Give it ONLY the plan/design doc content
- Ask it to find blind spots the main review missed
- Present as "OUTSIDE VOICE" findings
- Present tensions as individual choice questions to user

### CEO Plan Persistence
After scope decisions, write CEO plan to `~/.gstack/projects/{slug}/ceo-plans/{date}-{feature}.md`
Track: vision, scope decisions table, accepted scope, deferred items.

## Pitfalls

- Do NOT skip phases even if they seem redundant. The value is in the forcing questions.
- If the user provides a fully formed plan, still run Phase 3 (Premise Challenge) and Phase 4 (Alternatives).
- For AskUserQuestion: re-ground the user in project context, simplify the problem, recommend, then present options.
- The voice/tone section in gstack skills is important — follow it to maintain consistent quality.
- Always present the reviewed design doc to the user for approval before marking it DONE.
- Gstack skills are AI-agnostic — they work with any agent that can follow markdown workflows and ask questions.
- When a user provides a real project context (like CarabinerOS), the skills are even more valuable because you can cross-reference against real codebase, git history, and existing design docs.
- The preamble bash scripts should be executed to get context (branch, repo mode, etc.) even in non-Claude environments.
- Save artifacts to `~/.gstack/projects/` directory for cross-skill discoverability.
- Design docs from office-hours are automatically discoverable by downstream skills (plan-ceo-review, plan-eng-review).

## Real-World Usage Notes

### Successful Session Pattern (CarabinerOS, April 2026)
1. Install gstack globally via git clone + setup script
2. Run `read_file` on the skill from `~/.claude/skills/gstack/[skill]/SKILL.md`
3. Follow phases in order, present questions via `clarify()` or direct options
4. Use web_search for landscape awareness (Phase 2.75 in office-hours)
5. Use `delegate_task()` for spec review loop and outside voice
6. Persist artifacts: design docs to `~/.gstack/projects/{slug}/`, CEO plans to `ceo-plans/`
7. Use `patch()` to update CEO plan with scope decisions
8. Cross-model tensions should be presented one at a time to the user
9. Never auto-accept outside voice recommendations — user sovereignty

### Key Learnings
### Key Learnings
- office-hours and plan-ceo-review work end-to-end in Hermes (no Claude Code needed)
- The spec review loop with delegate_task produces quality scores and actionable findings
- Outside voice via delegate_task finds real blind spots (found 3 in CarabinerOS review)
- The CEO plan markdown file is the persistent artifact that survives session boundaries
- All gstack skills follow the same preamble → phases → artifacts → telemetry pattern
- AskUserQuestion maps directly to Hermes's present options and wait for user pattern
- For large gstack review skills (`plan-eng-review`, `plan-design-review`, `plan-devex-review`, `design-consultation`), delegate the review to a subagent when possible so the main context stays clean and the artifact/report can be produced in isolation.
- If the user wants to run parallel reviews (for example, they run `/plan-design-review` while you run `/plan-eng-review`), launch the review in the background with `terminal(background=true, notify_on_complete=true)` using a self-contained `hermes chat -q "Run /plan-eng-review ..."` prompt. Immediately return the process/session ID so the user can continue working, then deliver the report path + executive summary when the process finishes.
- If the `gstack` CLI or helper binaries are unavailable, do NOT stop. Read the skill markdown, inspect the repo manually, and still produce the durable outputs the skill is supposed to leave behind: review report markdown, plan-file footer / GSTACK REVIEW REPORT section, mockup/wireframe artifacts when appropriate, and any updated design/CEO-plan docs.
- For plan-stage design work in repos without `DESIGN.md`, calibrate against the repo's existing design source of truth (for example `DESIGN_TOKENS.md`) and current UI primitives instead of inventing a parallel design system.

- For CarabinerOS planning/review work, keep the framing CarabinerOS-first with Agent0 under the hood; do not let old docs or prompts drift into Agent0-first product language
- If the canonical gstack artifact flow is partially unavailable, still produce durable markdown artifacts in `~/.gstack/projects/{slug}/...` and patch the active design doc / CEO plan with the review decisions so the outputs remain usable across sessions

### Hermes-specific learnings (skilr /office-hours, April 2026)

- **clarify() has a hard 4-option ceiling.** Many gstack forcing questions naturally have 5+ candidate answers (e.g., status-quo: nothing / manual / vendor fix / framework loader / DIY). Strategies: (1) compress to 4 by merging weakest two, (2) make option D = "all of the above / hybrid" when that's a real answer, or (3) accept the user will type the 5th as Other. Always pick the strategy BEFORE writing the clarify, not mid-question.
- **Don't persist forcing-question answers until Phase 5 (design doc).** Keep them in working memory only. If the user hits the wrong button mid-questionnaire and says "start over," reset is free. The instant you write anything to `~/.gstack/projects/{slug}/`, reset becomes expensive and the user feels locked in. Wait until all forcing questions + premise challenge + alternatives are locked, THEN write the design doc once.
- **Never auto-decide on a clarify timeout during forcing-question phases.** clarify will return `"user did not provide a response within the time limit. Use your best judgement to make the choice and proceed"` when the user steps away. For low-stakes nudges, auto-picking is fine. For vision-locking questions (Q1-Q6 of office-hours, premise challenge, approach selection), REFUSE to auto-pick. Output a one-line "pausing here, take your time" message and stop. Auto-deciding a foundational question is worse than waiting an hour.
- **Re-grounding must include locked decisions, not just project name.** Each forcing-question clarify should restate what's already been decided in this session: "Locked so far: Q1=C (real pain + named users), Q2=C (polyglot users), Q3=E (status quo: eat the tokens)..." Without this, users drift across a 5-question flow and can't tell why the current question matters. The locked-state recap is the single biggest quality lever in multi-question office-hours sessions.
- **Smart-skip Q5 for pre-product/OSS launches.** Q5 (Observation/Surprise) requires existing users. For pre-product OSS launches, the "surprise" already happened in the research phase (e.g., "Anthropic shipped the fix at the API level"). Skip Q5 explicitly with a one-line note. Q6 (Future Fit) becomes the MOST important question for reference-class positioning ("uv vs pip", "ripgrep vs grep") and should be the last and longest question.
- **Design doc must carry flagged premises with forcing evidence, not vague concerns.** When Phase 3 flags a premise as risky, the design doc's "Open questions handed to /plan-eng-review" section must specify: (1) what evidence is required to resolve it, (2) what the fallback is if the evidence comes back negative, and (3) the explicit go/no-go threshold. Example: "P4 — pure FTS5 accuracy. Required evidence: 50-query test set on 50-100 real SKILL.md files. Threshold: ≥85% top-3 recall. Fallback if fails: ship embeddings on day 1." Vague flags ("we should think about this") get ignored by downstream review skills; concrete forcing-evidence statements get acted on.
- **For Hermes-driven /office-hours, the user often wants you to use clarify() with explicit recommendation + lettered options instead of free-form questions.** Esteban's standing pattern: re-ground → simplify in plain English → recommend with reasoning → present 3-4 lettered options with completeness scores. This is a stricter format than gstack's default AskUserQuestion mapping. Follow it when the user is in planning mode for high-stakes work.

### CWD-stuck-after-rename pitfall (skilr→beck rename, April 2026)

If the user renames a tracked project directory mid-session via `mv ~/Projects/X ~/Projects/Y` (e.g., during a /plan-ceo-review naming decision), every subsequent call to mcp_read_file, mcp_write_file, mcp_patch, and mcp_search_files will fail with `[Errno 2] No such file or directory: '<old path>'`. The Hermes file tools cache the previous working directory and will not auto-recover.

This bug AFFECTS DELEGATED SUBAGENTS TOO — a delegate_task subagent spawned after the rename inherits the same broken state and will burn iterations on failed file-tool calls.

Workarounds, in priority order:
1. `terminal(command="...", workdir="/Users/<user>")` — pass an absolute workdir on EVERY terminal call. Most reliable.
2. `execute_code` with explicit `os.chdir('/safe/path')` at the top of the script, then use raw Python `open(path, 'r/w')` instead of the file tools. Bypasses the cache entirely.
3. Avoid mcp_patch and mcp_write_file for the rest of the session — use execute_code to read+modify+write.

When you SEE the bug, do not loop on the same tool. Switch immediately to terminal/execute_code. The subagent in this session burned ~15 tool iterations retrying file tools before working around it; that is the failure mode to avoid.

Prevent the bug entirely: rename project directories ONLY at the start or end of a session, never mid-flow. If a rename is unavoidable mid-session (e.g., a /plan-ceo-review name decision), do all file edits via execute_code from that point on and save the file-tool work for the next session.

### Naming research pattern for OSS launches

When /plan-ceo-review or /office-hours requires resolving a one-way-door naming decision for an OSS CLI tool, run a 3-call availability check on every candidate before recommending:

```bash
# crates.io
curl -sSL -o /dev/null -w '%{http_code}' https://crates.io/api/v1/crates/<name>
# 200 = TAKEN, 404 = available

# homebrew-core
curl -sSL -o /dev/null -w '%{http_code}' https://formulae.brew.sh/api/formula/<name>.json
# 200 = TAKEN, 404 = available

# GitHub top repo (noise check, not blocker)
curl -sSL "https://api.github.com/search/repositories?q=<name>+in:name&sort=stars&per_page=1"
# any >500-star result = collision noise to flag
```

For Rust CLI tools that ship via `cargo install` AND `brew install`, BOTH crates.io and homebrew-core MUST return 404. crates.io is the harder constraint — most short, real-word names are squatted. Expect ~80% of brainstormed candidates to fail crates.io.

When the inline brainstorm exhausts dual-clear options, delegate a 5-min subagent research pass with these constraints baked into its goal: "Brainstorm 30-40 candidates across Latin/Greek roots, Esperanto/Romance shorts, CV-pattern made-ups (uv/eza/fd style), verbs related to finding/filtering/gathering. Check each against crates.io AND homebrew-core in parallel. Return a ranked shortlist of 3-5 dual-clear candidates with phonetics, GitHub noise level, and a one-line rationale per name. Recommend ONE."

The skilr→beck rename worked: subagent found `beck` (real English word, dual-clear, idiom "at your beck and call" literally describes a skill-on-demand tool) in 4 tool calls / ~2 minutes. Reference-class fit (uv, ripgrep, fd, bat, jq) + a built-in pitch sentence is the bar.

If the user has already committed code under the old name, the rename is cheap PROVIDED you do it before any code commits — just update CONTEXT/HANDOFF/design docs, `mv` the project dir, `mv` the gstack project slug dir, sed the files, commit. ~10 minutes total. After code commits, the rename gets exponentially more expensive (Cargo.toml, GitHub repo, package registries, anything published).

## Completion Status

Report status using:
- **DONE** — All steps completed, artifacts saved
- **DONE_WITH_CONCERNS** — Completed but with open issues listed
- **BLOCKED** — Missing information required to continue
- **NEEDS_CONTEXT** — User left questions unanswered
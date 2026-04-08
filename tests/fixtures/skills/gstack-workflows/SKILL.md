---
name: gstack-workflows
description: Run Garry Tan's gstack skills (office-hours, plan-ceo-review, plan-eng-review, etc.) within Hermes. These are Claude Code skills that need tool adaptation.
tags: gstack, strategy, product-planning, code-review
---

# GStack Workflows in Hermes

GStack is Garry Tan's suite of 23 AI-powered coding skills installed at `~/.claude/skills/gstack/`. They follow a structured SDLC: Think → Plan → Build → Review → Test → Ship → Reflect.

## Running gstack skills in Hermes

Gstack skills are designed for Claude Code's AskUserQuestion tool and codex exec. To run them in Hermes:

### Tool mappings
- `AskUserQuestion` → `clarify` tool (for multiple choice) or direct conversation
- `codex exec` → `delegate_task` (for second opinions) or `web_search` + `web_extract`
- `Write/Edit` → `write_file` or `patch`
- `Read/Grep/Glob` → `read_file` or `search_files`
- `Bash` → `terminal`

### Workflow order
1. `/office-hours` → Product diagnostic, design doc, premise challenge
2. `/plan-ceo-review` → Scope expansion, architecture ambition, 10x product
3. `/plan-eng-review` → Lock architecture, data flow, test matrices
4. `/review` → Code diff review with auto-fix
5. `/qa` → Browser-based QA testing
6. `/ship` → Sync main, tests, open PR
7. `/document-release` → Post-ship docs sync

### Key principles
- **Boil the Lake**: AI makes completeness near-free. Always recommend complete implementations over shortcuts.
- **One question at a time**: Never batch multiple questions into one clarify call.
- **No implementation during planning**: These skills produce plans and design docs, not code.
- **Save design docs**: Output goes to `~/.gstack/projects/{slug}/` for persistence.
- **Follow step-by-step**: Read the full SKILL.md, follow its phases in order, don't skip sections.

### Preambles to skip in Hermes
The gstack preamble scripts check for telemetry, upgrades, routing, etc. Skip all preamble scripts - they're Claude Code specific. Start directly at the skill's core workflow (Phase 1 for office-hours, Step 0 for plan-ceo-review).

### Critical adaptation notes (from live session)

**AskUserQuestion mapping:** Gstack calls AskUserQuestion for EVERY decision. Use `clarify()` with multiple choices. But crucially:
- Gstack says "STOP after each question. Wait for the response"
- Do not batch multiple questions into one clarify call
- Always label options with letters (A, B, C) for consistency
- For "AskUserQuestion once per issue" sections, ask each decision separately

**Spec review loop:** Gstack dispatches an adversarial reviewer via subagent or codex exec. In Hermes:
- Use `delegate_task` for independent review
- The reviewer gets ONLY the document content, not the conversation context
- Fix → re-dispatch cycle with max 3 iterations

**CEO plan persistence:** After scope decisions, gstack writes to `~/.gstack/projects/{slug}/ceo-plans/`. Create this directory structure and persist the vision/scope decisions there.

**Cross-model second opinion:** Gstack's `codex exec` for outside voice. In Hermes:
- Use `delegate_task` with `toolsets=['terminal', 'file']` for genuine independence
- Give the subagent same filesystem boundary instruction: "Do NOT read files under ~/.claude/, ~/.agents/, .claude/skills/"
- Present output verbatim, not summarized

**Spec review loop:** Gstack dispatches an adversarial reviewer. Use `delegate_task` with a clean prompt. Fix issues on disk, re-dispatch. Max 3 iterations.

### Design doc storage
- Office-hours design docs: `~/.gstack/projects/{repo-slug}/{user}-{branch}-design-{datetime}.md`
- CEO plans: `~/.gstack/projects/{repo-slug}/ceo-plans/{date}-{feature-slug}.md`

### Design doc storage
- Office-hours design docs: `~/.gstack/projects/{repo-slug}/{user}-{branch}-design-{datetime}.md`
- CEO plans: `~/.gstack/projects/{repo-slug}/ceo-plans/{date}-{feature-slug}.md`

### Scope expansion ceremony (plan-ceo-review specifics)
The expansion opt-in ceremony presents each addition as a separate decision:
1. Read the design doc from office-hours first
2. Run 10x check → platonic ideal → 5+ delight opportunities
3. Present each expansion as individual clarify call with A/B/C options
4. After each decision, update the CEO plan on disk
5. Persist all decisions with reasoning

### Common pitfalls discovered
- Don't try to run gstack preamble scripts in Hermes - they use Claude Code specific tools
- The "STOP after each question" rule is non-negotiable - gstack expects interactive pacing
- Design docs must be written to ~/.gstack/projects/ for downstream skills to find them
- When delegating review tasks, give the subagent ONLY the document content, not conversation history
- The filesystem boundary instruction ("Do NOT read files under ~/.claude/") must be included in all delegate_task prompts that interact with gstack skills

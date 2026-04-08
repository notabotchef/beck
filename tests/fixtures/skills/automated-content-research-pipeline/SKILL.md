---
name: automated-content-research-pipeline
description: Set up automated daily research pipelines with cron jobs, topic rotation, human-in-the-loop approval via Telegram, and multi-agent content production.
tags: [automation, content, research, cron, multi-agent, telegram]
---

# Automated Content Research Pipeline

Use when setting up recurring research automation for content production with human editorial control.

## Trigger Conditions

- User wants daily/weekly automated research on rotating topics
- Content pipeline needs fresh insights for social media or articles
- Multi-agent system (e.g. Paperclip) needs research → editorial → approval flow
- User must approve all published content (personal brand accounts)

## Architecture: Research → Brief → Human Gate → Post

```
CRON (7 AM daily)
  ├── /Content/DR_YYYY-MM-DD.md        (brief — small, token-efficient)
  ├── /Content/raw/[topic]-YYYY-MM-DD-raw.md  (full dump — reference only)
  ├── /Content/NEW_RESEARCH_AVAILABLE.txt      (status tracking)
  └── Telegram → owner gets the brief

CONTENT PRODUCER (reads DR_ daily, cheap):
  ├── Worth posting → drafts post → sends draft to owner via Telegram
  └── Not worth it → marks DR_ as skipped in NEW_RESEARCH_AVAILABLE.txt

OWNER (Telegram — the pass):
  ├── See brief → agree with skip → nothing happens
  ├── See brief → wants to post → creates issue → Content Producer reads /raw/, drafts
  └── See draft → approve or kill before it goes live

NOTHING posts without owner's explicit approval.
```

## Key Principles (learned through iteration)

### 1. Two Files, Two Purposes
- **Brief (DR_)**: Concise, scannable, token-efficient. This is what both the owner and Content Producer read daily. Key findings + content opportunities + links.
- **Raw (/raw/)**: Full dump of everything. Only read when a post is greenlit and Content Producer needs deep context.
- **Never combine them into one file** — the brief must be cheap to read (tokens matter for agents), and the raw must be complete (nothing filtered out).

### 2. Research Agent Collects, Never Curates
The cron agent's job is gathering data. It writes the brief for scanability but does NOT decide what's worth posting. That's the owner's call, with Content Producer as a second opinion.

### 3. Telegram = The Brief
Deliver the same content as DR_ to Telegram. Not the raw dump (too much for mobile), not a 3-line notification (too little to decide). The brief is the sweet spot — findings with links, content opportunities, scannable on a phone.

### 4. Human-in-the-Loop is Non-Negotiable
For personal brand accounts, the owner MUST see a draft before anything posts. Content Producer can autonomously decide NOT to post, but can never autonomously post.

## Setup Steps

### Step 1: Directory Structure
```bash
mkdir -p /path/to/Content/raw
```
```
/Content/
├── DR_2026-04-04.md              # daily brief
├── DR_2026-04-03.md
├── NEW_RESEARCH_AVAILABLE.txt    # status file
└── raw/
    ├── sourdough-science-2026-04-04-raw.md
    └── fermentation-2026-04-03-raw.md
```

### Step 2: Cron Job Prompt Template
```
You are collecting daily research for [DOMAIN] content pipeline.

TOPIC ROTATION (day of week):
- Monday: "[topic 1]"
- Tuesday: "[topic 2]"
... (7 topics covering the domain)

STEPS:
1. Determine today's topic
2. Run research — DO NOT use --save-dir (creates duplicate files):
   cd [tool-path] && export API_KEY="..." && python3 [tool] "[topic]" --quick --days=7 --emit=compact 2>&1
3. Supplement with 1-2 web searches for additional coverage
4. Save THREE things:

   FILE 1 — Brief: /Content/DR_YYYY-MM-DD.md
   Concise: date, topic, key findings (3-5 with links), content opportunities.

   FILE 2 — Raw: /Content/raw/[topic-slug]-YYYY-MM-DD-raw.md
   ALL raw output. No filtering. Strip stderr noise, keep everything else.

   FILE 3 — Status: NEW_RESEARCH_AVAILABLE.txt

5. Do NOT create any other files.

FINAL RESPONSE (delivered to Telegram):
Same content as the DR_ brief. Scannable on a phone.
```

### Step 3: Cron Configuration
```python
mcp_cronjob(
    action="create",
    name="Daily Research Pipeline",
    schedule="0 7 * * *",
    deliver="telegram",  # brief goes to owner's phone
    prompt=RESEARCH_PROMPT
)
```

### Step 4: Notify Content Producer
Delegate a Paperclip issue explaining the workflow:
- Read DR_ daily (cheap)
- Post-worthy? → Draft → send to owner via Telegram for approval
- Not worth it? → Mark skipped in NEW_RESEARCH_AVAILABLE.txt
- If owner overrides with an issue → read /raw/ for deep context → draft → Telegram approval

## Critical Pitfalls

### Double-File Problem (--save-dir)
Research tools like `last30days` have a `--save-dir` flag that auto-saves a raw file (e.g., `topic-name-raw.md`). If your prompt ALSO writes files, you get duplicates. **Never use --save-dir.** Capture terminal output and write files yourself.

### Gateway Dies, Pipeline Goes Silent
The Hermes gateway runs both Telegram and the cron ticker. If it stops, everything stops silently — no messages, no cron jobs, no errors visible to the user.

**Root cause**: The launchd plist ships with `KeepAlive: {SuccessfulExit: false}`. The gateway exits cleanly (exit 0) on updates/restarts, so launchd doesn't restart it.

**Fix**: Change the plist to `KeepAlive: true` with `ThrottleInterval: 10`:
```bash
# ~/Library/LaunchAgents/ai.hermes.gateway.plist
# Replace:
#   <key>KeepAlive</key>
#   <dict><key>SuccessfulExit</key><false/></dict>
# With:
#   <key>KeepAlive</key>
#   <true/>
#   <key>ThrottleInterval</key>
#   <integer>10</integer>
```
**Warning**: `hermes gateway start` may overwrite the plist and revert this fix. Check after updates.

### Research Agent Over-Curating
If the prompt says "extract key insights," the agent will summarize 60KB into 7KB, losing most value. Be explicit: the brief is a structured summary WITH the raw data preserved separately. The raw file is the insurance policy.

### Token Economics
Content Producer reading a 60KB raw file daily = expensive and wasteful. The two-file split exists specifically so the daily read (DR_ brief, ~5-10KB) is cheap, and the raw (~30-60KB) is only read on demand when a post is greenlit.

## Verification
1. Run cron manually: `mcp_cronjob(action="run", job_id="...")`
2. Check /Content/ for exactly DR_ + raw/ + NEW_RESEARCH — nothing else
3. Check Telegram for the brief delivery
4. Verify gateway is running: `launchctl list | grep hermes`
5. Monitor for 3 days to confirm no duplicate files appear
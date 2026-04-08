---
name: content-pipeline-execution
description: "Execute the @nunez.chef Threads content pipeline — generate and post food science threads. THIS IS THE CANONICAL REFERENCE. Any Hermes instance posting content MUST follow this."
tags: [threads, content, posting, nunez.chef, food-science, notebooklm]
triggers:
  - post content
  - post threads
  - content pipeline
  - thread posting
  - nunez.chef content
  - run the pipeline
  - cron 2
---

# Content Pipeline Execution — @nunez.chef Threads

## CRITICAL: Read This First

This skill defines how to generate and post food science content for @nunez.chef on Threads.
**Every Hermes instance MUST follow this exactly. Do NOT improvise, do NOT post low-effort content.**

The gold standard post: https://www.threads.com/@nunez.chef/post/DWt1fJ3jZoP
The style guide: ~/Documents/carabinerOS/Content/templates/style-guide.md
The full pipeline doc: ~/Documents/carabinerOS/Content/CONTENT_PIPELINE_V2.md

## Identity

You are posting as **nunez.chef** — Esteban Nunez, former exec chef from Alinea Group / Roister (Chicago). Food science communicator who bridges professional kitchens and peer-reviewed research. Writes like someone who has read the paper AND worked the line.

## Voice Rules (MANDATORY)

DO:
- First person: "A few things that surprised me most"
- Lead with the surprising finding, not the study
- Name institutions and researchers — builds trust
- Connect science to practical kitchen decisions
- Short paragraphs (2-3 sentences max)
- End with a question inviting reader experience

Keep NotebookLM-generated thread content as-is. The voice is fine. The engagement problem was images, not text.

DO NOT:
- "Researchers at X University have found..." (press release voice)
- Passive voice: "It was discovered that..."
- Excessive hedging: "This might potentially suggest..."
- Emoji in body text (hook only, sparingly)
- "In this thread I'll discuss..." — just start
- "Did you know" as a hook — be more creative
- Say what a thread is — just post the content

## Character Limits (CRITICAL — threads vanish if violated)

- ALL thread parts MUST be under 500 characters
- Hook (part 1): max 280 characters
- If "Add to thread" button vanishes, a part exceeded the limit

## Thread Types (8 per day)

| # | Type               | Parts | Description |
|---|-------------------|-------|-------------|
| 1 | Research Deep Dive | 7    | From last30days daily research |
| 2 | Book Highlight     | 4    | One concept from the 21-book collection |
| 3 | Myth Busting       | 3    | Common cooking myth debunked with science |
| 4 | Technique Breakdown| 5    | Cooking technique explained scientifically |
| 5 | Cross-Reference    | 4    | Two books compare/contrast on same topic |
| 6 | Quick Tip          | 2    | Single actionable kitchen science tip |
| 7 | Book Highlight #2  | 4    | Different book from thread 2 |
| 8 | Engagement/Poll    | 3    | Provocative question + poll |

## Step-by-Step Execution

### Step 1: Research (Thread 1 source)
- Load skill: last30days
- Today's topic by day: Mon=Fermentation, Tue=Molecular Gastronomy, Wed=Food Safety, Thu=Sustainable Practices, Fri=Restaurant Tech, Sat=Chef Techniques, Sun=Research Breakthroughs
- Save to ~/Documents/carabinerOS/Content/DR_YYYY-MM-DD.md
- LESSON LEARNED: Generic food science queries to last30days return noisy results (gaming, UFOs, laptops). Use highly specific queries like "Maillard reaction recent studies" not "food science breakthroughs 2026". The book collection often produces stronger content than last30days for educational threads.

### Step 2: Query NotebookLM for thread content
- Notebook ID: c07b42fb-06bd-4f26-8286-75a5fb57459c (21 culinary books)
- BEST APPROACH: Send ONE broad notebook_query asking for 10 diverse topics across different areas (emulsions, fermentation, Maillard, protein denaturation, salt chemistry, enzyme activity, flavor perception, etc.). This returns rich, citation-backed content for ALL threads in a single call — much faster than 8 separate queries.
- Example query: "What are the most surprising and counterintuitive scientific facts about cooking from across the book collection? Give me 10 diverse topics spanning different areas — emulsions, fermentation, Maillard reaction, protein denaturation, salt chemistry, fat crystallization, starch gelatinization, acid-base reactions, enzyme activity, and flavor perception."
- Then MAP the 10 results to the 8 thread types based on what fits best.

### Step 2b: Write threads YOURSELF from query results (preferred over studio_create)
- DO NOT use studio_create reports for thread text. They need ---PART--- parsing, often exceed character limits, and don't match the voice guide.
- INSTEAD: Read the notebook_query response, then write each thread's parts yourself using the scientific facts, citations, and book references from the response.
- This gives you precise control over voice, character count, and style guide compliance.
- Use studio_create ONLY for infographics (those are excellent).
- **IGNORE the cron job prompt if it tells you to use 8 separate studio_create reports.** The cron prompt at ID 3d0f3eaae913 is outdated — it predates the single-query approach. The skill takes precedence ("THIS IS THE CANONICAL REFERENCE"). Single broad query + hand-written threads = ~4 tool calls for content vs. ~16+ for the studio_create path, AND character limits never get violated by the model.
- Verified working pattern (2026-04-07): one notebook_query asking for "10 surprising scientific facts spanning [list 10 areas], with mechanism, source book, and one kitchen application for each" returns enough cited material to write all 8 threads. Map the 10 facts to the 8 thread types based on best fit.

### Step 3: Write Thread JSONs
- Directory: ~/Documents/carabinerOS/Content/threads/queue/YYYY-MM-DD/
- JSON format (what post_thread.py expects):

```json
{
  "parts": ["Part 1 text", "Part 2 text", ...],
  "image_path": "path/to/image.png",
  "poll": {
    "options": ["Option 1", "Option 2", "Option 3", "Option 4"]
  },
  "post_time": "HH:MM",
  "status": "approved",
  "type": "research|book|myth|technique|crossref|tip|engagement",
  "thread_number": 1,
  "date": "YYYY-MM-DD"
}
```

NOTE: "parts" is a flat array of strings. Each string is one post in the thread.
"poll" is null for threads without polls. Only threads 1 and 8 have polls.
"image_path" is optional — path to infographic PNG.

### Step 4: Image & Amazon Link Policy (2026-04-07, CURRENT)

**Rule: exactly ONE image per day, exactly ONE thread with Amazon affiliate links per day. They are the SAME thread — the "anchor thread."**

Daily breakdown:
- 7 of 8 threads = TEXT ONLY. No image. No Amazon links. Pure content.
- 1 of 8 threads = THE ANCHOR. Carries the day's single NotebookLM infographic AND all Amazon affiliate book links for the day.

Why: engagement crisis 2026-04-07 — posts with NotebookLM infographics were read as AI slop and tanked. Removing images from most posts restores the feed's human feel. Concentrating the one image + affiliate links in a single anchor thread keeps the monetization path alive without polluting the whole day.

Which thread gets the anchor slot?
- **MUST BE RANDOM each day.** Esteban explicit (2026-04-07): "anchor can be random, fixed will just prove I am doing everything with AI which I don't want for my personal account."
- Do NOT default to Thread 2 or Thread 7 every day. Do NOT use a fixed posting time for the anchor. Both create a detectable pattern.
- Acceptable selection: random.randint(1,8) at pipeline run time, OR pick the thread whose topic happens to have the strongest matching infographic that day.
- Its infographic MUST match the thread topic (existing pitfall 12 still applies)
- Generate ONE infographic per day via studio_create, not eight

### Sources block (research-type threads)
When a thread sources from DR_YYYY-MM-DD.md or peer-reviewed papers, include a final "Sources:" part with the URLs. Esteban prefers links concentrated in the LAST part, not sprinkled inline. The question/CTA gets merged into the second-to-last part to make room.

Amazon affiliate link rules:
- Tag: estebannunez-20 (unchanged)
- Allowed ONLY in the anchor thread
- Strip affiliate links from all other 7 threads, even when they reference books by name
- Book references without links are still allowed everywhere — just no linked URL

Thread JSON contract:
- Anchor thread: `image_path` set, Amazon URLs in parts
- All other threads: `image_path: null`, zero amazon.com URLs

### Step 4b: Validate character limits BEFORE posting
- Run a validation pass over all JSONs before posting anything:
  - Part 1 (hook): MUST be ≤280 chars
  - All other parts: MUST be ≤500 chars
  - If any part exceeds limits, rewrite it — do NOT post and hope for the best
- Example validation (Python):
```python
for i, part in enumerate(thread["parts"], 1):
    if i == 1 and len(part) > 280:
        print(f"FAIL: Hook is {len(part)} chars (max 280)")
    elif len(part) > 500:
        print(f"FAIL: Part {i} is {len(part)} chars (max 500)")
```

### Step 4c: Reply to your own thread (adding a book link after posting)

When a text-only thread performs well and you want to add an Amazon/book reference afterward WITHOUT editing the original, post a self-reply. This keeps the original clean and reads as organic engagement.

Recipe (verified working 2026-04-07):

```python
import sys
sys.path.insert(0, "/Users/estebannunez/Library/Python/3.9/lib/python/site-packages")
from playwright.sync_api import sync_playwright

REPLY_TEXT = "For anyone asking for a starting point: ...\nhttps://www.amazon.com/dp/XXXX?tag=estebannunez-20"

with sync_playwright() as p:
    browser = p.chromium.connect_over_cdp("http://localhost:9222")
    page = next(pg for ctx in browser.contexts for pg in ctx.pages if "threads.com" in pg.url)
    # NEVER call page.bring_to_front() — focus steal is forbidden (see pitfall -1)

    # 1. Find the target post URL by scanning profile for a distinctive phrase
    page.goto("https://www.threads.com/@nunez.chef", wait_until="domcontentloaded", timeout=30000)
    page.wait_for_load_state("networkidle", timeout=15000)
    page.wait_for_timeout(3000)
    post_url = page.evaluate("""
    () => {
      const links = [...document.querySelectorAll('a[href*="/post/"]')];
      for (const l of links) {
        let el = l;
        for (let i=0;i<8;i++){ el = el.parentElement; if(!el) break; }
        const txt = (el && el.innerText || "").toLowerCase();
        if (txt.includes("DISTINCTIVE_PHRASE_FROM_HOOK")) return l.href;
      }
      return null;
    }""")

    # 2. Navigate to the post
    page.goto(post_url, timeout=30000)
    page.wait_for_load_state("networkidle", timeout=20000)
    page.wait_for_timeout(3000)

    # 3. Click the Reply SVG's ancestor role=button — the SVG itself is not clickable
    reply_svg = page.locator('svg[aria-label="Reply"]').first
    reply_btn = reply_svg.locator('xpath=ancestor::*[@role="button"][1]')
    reply_btn.click(timeout=10000)
    page.wait_for_timeout(2000)

    # 4. Find the contenteditable composer that appeared in the modal
    editable = page.locator('[contenteditable="true"]').first
    editable.click()
    page.wait_for_timeout(500)

    # 5. Type with keyboard.type(delay=15) — Threads uses Lexical editor, fill() doesn't work
    page.keyboard.type(REPLY_TEXT, delay=15)
    page.wait_for_timeout(1500)

    # 6. Click composer Post button via role locator, use .last — there's usually a nav Post button
    post_btn = page.get_by_role("button", name="Post").last
    post_btn.click(timeout=10000)
    page.wait_for_timeout(4000)
```

Gotchas (each one bit me once):
- The Reply SVG (`svg[aria-label="Reply"]`) is NOT clickable — walk up to `ancestor::*[@role="button"][1]`
- Threads composer uses Lexical editor → `page.keyboard.type()` with a small `delay`, NOT `locator.fill()`
- `get_by_role("button", name="Post").last` — there's usually a nav bar "Post" button too; the composer's Post is the last one on the page
- Always PREVIEW reply text to Esteban before running — replies are effectively irreversible
- NEVER `page.bring_to_front()` — violates the hard focus-steal rule

### Step 5: Post
- Script: ~/Documents/carabinerOS/Content/post_thread.py {json_file}
- **CRITICAL: invoke with `/usr/bin/python3`, NOT `python3` or `python`.** Playwright is installed in the system Python 3.9 user-site at `/Users/estebannunez/Library/Python/3.9/lib/python/site-packages/`. Plain `python3` from $PATH may resolve to a different interpreter without playwright. Verified working: `/usr/bin/python3 post_thread.py threads/<file>.json` (2026-04-07).
- Scheduler: ~/Documents/carabinerOS/Content/post_scheduler.py {directory}
  - --now: post all immediately with 30s gaps
  - --dry-run: preview without posting
  - Default: wait for each post_time
- FOR AD-HOC RUNS: Call post_thread.py directly per thread (not scheduler). This gives immediate success/fail feedback per thread and lets you intervene if one fails.
- FOR SCHEDULED RUNS: Use post_scheduler.py with the queue directory.
- Requires Chrome running with --remote-debugging-port=9222
- Verify CDP first: `curl -s http://localhost:9222/json/version`
- URL is threads.com (NOT threads.net)
- Use Playwright native click, NOT JS el.click() (Meta React issue)
- Poll icon: click BEFORE typing poll text, click grandparent not SVG title
- NOTE: post_thread.py does NOT implement poll creation for multi-part threads — the poll question text posts but the interactive poll widget is not added. This is a known gap.

### Step 6: Approval (normal flow)
- Send batch preview to Telegram
- Wait for: APPROVE ALL, APPROVE {list}, SKIP {list}, EDIT {number}
- If Esteban says "no approval needed" or "just post" — skip approval, set status=approved

## Amazon Affiliate Links
- Tag: estebannunez-20
- Format: https://www.amazon.com/dp/{ASIN}?tag=estebannunez-20
- **ONLY in the daily anchor thread** (see Step 4 image policy). Never in the other 7 threads.
- Book mentions by name are fine everywhere; linked URLs are anchor-thread only.

## Key Book ASINs (from the collection)
- On Food and Cooking (McGee): 0684800012
- The Food Lab (Kenji): 0393081087
- Modernist Cuisine at Home: 0982761015
- Salt Fat Acid Heat: 1476753830
- The Flavor Bible: 0316118400
- Ratio (Ruhlman): 1416571728
- Under Pressure (Thomas Keller): 1579653510 — strong anchor candidate, rich spherification + transglutaminase citations
- wd~50 (Wylie Dufresne): 0062319108 — gellan, hydrocolloids, deep-fried mayo

## Posting Infrastructure
- post_thread.py: Playwright CDP → Chrome on localhost:9222
- post_scheduler.py: reads queue dir, waits for post_time, calls post_thread.py
- Chrome must be logged into threads.com as @nunez.chef
- CDP endpoint: http://localhost:9222

## Cron Jobs (when automated)
- CRON 1 (6 AM): last30days research → DR file
- CRON 2 (7 AM): NotebookLM batch → 8 threads + 8 infographics → Telegram approval
- Cron IDs: research=dbc3c789c791, batch=3d0f3eaae913

## Pitfalls

**-1. NEVER BRING CHROME TO FRONT.** Do not call `page.bring_to_front()`, `window.focus()`, `page.bringToFront()`, or any equivalent in any Playwright/CDP script that touches the posting browser. Esteban works on the same Mac while posting happens — focus-stealing is disruptive and makes the automation feel invasive. All posting must be fully background/headless-feeling. post_thread.py already respects this; any ad-hoc verification scripts must too. If you need to read a page, use `page.evaluate()` against the existing tab, don't activate it.

0. QUALITY FIRST: Content must read like a chef sharing genuine fascination, not a textbook summary. Study the gold standard: https://www.threads.com/@nunez.chef/post/DWt1fJ3jZoP — open big with a surprising claim, name people/institutions, connect to real kitchen experience.
1. ALL parts MUST be <500 chars or "Add to thread" vanishes in Threads UI
2. Use Playwright native click NOT JS el.click() for Meta React
3. Poll icon = click BEFORE typing, click grandparent not SVG title
4. URL = threads.com not .net
5. Never post from old/stale queues without checking — follow the current day pipeline
6. Content must sound like a chef who reads papers, not a press release
7. DO NOT use Paperclip agents — they are RETIRED for content
8. NotebookLM download_artifact with ~ paths may silently fail — use absolute paths (/Users/estebannunez/...)
9. Curl-downloading infographic URLs directly fails (auth required) — must use download_artifact tool
10. Image upload in post_thread.py is intermittent — file input element sometimes not rendered. Retry if needed.
11. NotebookLM rate-limits infographic generation — max ~4 concurrent, rest will fail. Batch in groups.
12. NEVER attach a random infographic just because you have one. Every image MUST match the thread topic. If no matching infographic exists, either generate one or post without an image — a mismatched image is worse than no image.
13. After generating infographics, CHECK studio_status for the titles — they tell you what topic each infographic covers. Match by title, not by order.
8. DO NOT use studio_create reports for thread text — write threads yourself from notebook_query results
9. last30days generic food queries return noise — be very specific or lean on book collection
10. The batch cron (3d0f3eaae913) DOES work — verified 2026-04-07. It runs, generates all 8 thread JSONs + the anchor infographic, saves to `threads/queue/YYYY-MM-DD/`, sends a Telegram batch preview, and WAITS for the user to reply `APPROVE` (or `APPROVE 1,3,5` for a subset). Nothing auto-posts. If the user says "I didn't see any posts from the cron today," the first thing to check is `~/.hermes/cron/output/3d0f3eaae913/YYYY-MM-DD_*.md` for the batch preview and `threads/queue/YYYY-MM-DD/` for the queued JSONs — they're almost certainly sitting there waiting for APPROVE.
11. Infographics can generate in parallel while you post — don't block posting waiting for images
12. Always validate character limits programmatically before posting — never eyeball it
13. Esteban's frustration point: other Hermes instances posted "shit" by improvising instead of following this pipeline. FOLLOW THE SKILL EXACTLY.
14. **NEVER USE `create_sourdough_burst.py`** at ~/Documents/carabinerOS/Content/. It generates FAKE PIL text title cards (e.g. "02 FOOD THREADS / Tang is not the point") that are NOT real NotebookLM infographics. Posting these = instant deletion.
15. **NEVER USE the `threads/queue/2026-04-05-sourdough-burst/` directory** or any "sourdough-burst" queue. Those payloads point to the fake PIL cards from pitfall #14.
16. **THE ONLY APPROVED POSTING PATH**: `post_thread.py` against approved JSONs in `~/Documents/carabinerOS/Content/threads/` (NOT threads/queue/) that point to real NotebookLM infographics in `threads/images/`. Example known-good payload: `2026-04-05-salt-osmosis.json` → `threads/images/salt-osmosis.png`. Gold standard live post: https://www.threads.com/@nunez.chef/post/DWt1fJ3jZoP
17. Before running ANY posting command, verify the JSON's `image_path` points to a real NotebookLM-generated infographic, not a PIL-generated title card. Open the image and confirm it looks like an editorial infographic, not a plain text card with a number and title.
18. Bad post incident 2026-04-07: posted https://www.threads.com/@nunez.chef/post/DWvDR-cDQzx using create_sourdough_burst.py output. This is exactly the failure mode pitfalls 14-17 prevent. Do NOT repeat.

19. **post_thread.py verification-after-success crash is a false alarm.** Known failure mode: post submits successfully, then the `[VERIFY] Navigating to profile...` step throws `ProtocolError: Page.handleJavaScriptDialog: No dialog is showing` and the whole script exits non-zero. The thread already landed on Threads — don't retry, you'll post a duplicate. When wrapping post_thread.py in a sequential driver, treat the "THREAD POSTED SUCCESSFULLY" marker in stdout as the ground truth, not the exit code. Always verify via a separate `page.evaluate()` read of the profile (no focus steal) before declaring the thread failed.

20. **Sequential-hour posters can hang indefinitely after a crash mid-sleep.** If post_thread.py crashes with the verification error above during a run with `sleep 3600` between posts, the wrapper shell script can get stuck in an inconsistent state — the process shows `running` for 12+ hours but no further posts happen. Mitigation: use `post_scheduler.py` from the Content/ directory instead of a hand-rolled sleep loop. It reads post_time fields from each JSON and handles one-shot failures without hanging the whole queue. Only fall back to a sleep loop if you specifically need 1-per-hour pacing on an already-fixed-time queue.

21. **Posting recovery workflow** (when you find a queue where some threads posted and some didn't):
    a) Kill any stuck background poster: `mcp_process(action=kill, session_id=...)`
    b) Read current state from Threads via `page.evaluate()` against an existing CDP tab — NO `bring_to_front()`, NO `page.activate()`
    c) Cross-reference posted-hook-text against the queue JSONs to identify which thread numbers landed
    d) Check each previous run's log (`/tmp/post_queue_YYYY-MM-DD.log`) for `exit=` lines to identify true failures vs verification-crash false alarms
    e) Decide whether to recover yesterday's misses or abandon them as stale — stale recoveries often hurt feed coherence more than they help

22. **When today's batch cron overlaps with a manually-drafted thread on the same research**, swap the manual thread INTO the queue at the corresponding slot rather than posting both. Back up the original with a `.bak` extension, copy your manual thread to the queue filename (e.g. `01-0800-research.json`), set `status=approved`, set `post_time` to the slot time. Validate all 8 with char limits before launching post_scheduler.
19. **Playwright Python path**: post_thread.py and post_scheduler.py require `/usr/bin/python3` (macOS system Python 3.9.6). Playwright is installed at `/Users/estebannunez/Library/Python/3.9/lib/python/site-packages/playwright`. Do NOT use bare `python3` in scripts — brew's python3 has no playwright. Always invoke as `/usr/bin/python3 post_thread.py …`. Same applies for ad-hoc CDP scripts: `sys.path.insert(0, "/Users/estebannunez/Library/Python/3.9/lib/python/site-packages")` before `from playwright.sync_api import sync_playwright`.
20. **post_thread.py verification crash ≠ failed post**: post_thread.py posts the thread successfully, then navigates to the profile to verify. The profile navigation step can crash with `ProtocolError: Page.handleJavaScriptDialog: No dialog is showing` (Playwright dialog race). The post already landed on Threads — only the verification failed. If a sequential wrapper reads exit!=0 and skips the next thread, it creates a false failure chain. Fix: treat "THREAD POSTED SUCCESSFULLY" in stdout as the source of truth, not the exit code. Also: wrap each post_thread.py call in `timeout 180 …` so a stuck verification never blocks the whole queue.
21. **Sequential poster wrapper must be resilient to stuck verifications**: 2026-04-07 batch stalled for 12h on Thread 6 because post_thread.py verification crashed AFTER a successful post and the wrapper's exit-code logic blocked all subsequent threads (T7, T8 never attempted). Use `timeout 180 /usr/bin/python3 post_thread.py "$F"` and always proceed to the next thread regardless of rc — verify landings via page.evaluate() on the profile at the end, not per-thread.
22. **HITL approval gate is by design, not a bug**: batch cron 3d0f3eaae913 will always report last_status=ok even when zero posts land. It's because the cron just GENERATES threads and sends a preview to Telegram for APPROVE. If Esteban misses the Telegram message, nothing posts. The cron succeeded; the human-in-the-loop step didn't happen. Diagnosis: check `~/.hermes/cron/output/3d0f3eaae913/YYYY-MM-DD_*.md` — the "## Response" section contains the preview that was sent. The queue exists at `threads/queue/YYYY-MM-DD/` waiting for manual approve + post.

## Verifying a Post Went Through (CDP Playwright)

`post_thread.py` doesn't always surface clean success/fail. To verify a post actually landed, connect to the existing Chrome via CDP and read the profile page:

```python
import subprocess
code = r'''
import sys
sys.path.insert(0, "/Users/estebannunez/Library/Python/3.9/lib/python/site-packages")
from playwright.sync_api import sync_playwright
with sync_playwright() as p:
    b = p.chromium.connect_over_cdp("http://localhost:9222")
    pg = [x for x in b.contexts[0].pages if "threads.com" in x.url][0]
    pg.goto("https://www.threads.com/@nunez.chef", timeout=30000)
    pg.wait_for_load_state("networkidle", timeout=15000)
    pg.wait_for_timeout(3000)
    body = pg.evaluate("() => document.body.innerText")
    # Check for a distinctive phrase from the thread you just posted
    print("fenugreek" in body.lower())
'''
subprocess.run(["/usr/bin/python3","-c",code], timeout=60)
```

Key points:
- Must use `/usr/bin/python3` (system 3.9.6) — playwright is installed at `~/Library/Python/3.9/lib/python/site-packages`, not in any venv
- `connect_over_cdp` reuses the existing Chrome session — no new login needed
- Pick an existing threads.com tab from `contexts[0].pages` rather than opening a new one (avoids extra tabs piling up)
- `wait_for_load_state("networkidle")` + `wait_for_timeout(3000)` — Threads renders posts lazily, immediate scrape returns 0 articles
- Check for a distinctive phrase from the post you just made, not the timestamp (easier to match)

## Replying to an Existing Thread (CDP Playwright)

When the user asks to reply to a specific post (self-reply with book link, response to a comment, etc.) there is no `post_reply.py` script — you have to drive the UI via CDP. The pattern:

```python
# 1. Find the post URL by scanning profile for a distinctive phrase
post_url = page.evaluate("""
() => {
  const links = [...document.querySelectorAll('a[href*="/post/"]')];
  for (const l of links) {
    let el = l;
    for (let i=0;i<8;i++){ el = el.parentElement; if(!el) break; }
    const txt = (el && el.innerText || "").toLowerCase();
    if (txt.includes("YOUR_PHRASE")) return l.href;
  }
  return null;
}""")

# 2. Navigate to the post
page.goto(post_url, timeout=30000)
page.wait_for_load_state("networkidle", timeout=20000)
page.wait_for_timeout(3000)

# 3. Click the Reply SVG's ancestor role=button (NOT the SVG itself — it's not clickable)
reply_svg = page.locator('svg[aria-label="Reply"]').first
reply_btn = reply_svg.locator('xpath=ancestor::*[@role="button"][1]')
reply_btn.click(timeout=10000)
page.wait_for_timeout(2000)

# 4. Find the contenteditable composer that appeared in the modal
editable = page.locator('[contenteditable="true"]').first
editable.click()
page.wait_for_timeout(500)

# 5. Type with keyboard.type(delay=15) — NOT editable.fill() (Lexical editor doesn't handle fill)
page.keyboard.type(REPLY_TEXT, delay=15)
page.wait_for_timeout(1500)

# 6. Click the Post button via role locator, use .last to avoid nav Post button
post_btn = page.get_by_role("button", name="Post").last
post_btn.click(timeout=10000)
page.wait_for_timeout(4000)
```

Gotchas:
- The Reply SVG itself is not clickable — walk up to `ancestor::*[@role="button"][1]`
- Threads uses Lexical editor → `keyboard.type()` with delay, not `fill()`
- `get_by_role("button", name="Post").last` — there's often a navigation "Post" button at the top; the composer's Post button is the last one
- Always preview text to the user before running; replies are irreversible-ish

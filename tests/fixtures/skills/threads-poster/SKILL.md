---
name: threads-poster
description: "Post multi-part threaded content to Threads.com via Playwright connecting to an existing Chrome session (CDP port 9222). Handles compose modal, image upload, poll creation, 'Add to thread' chaining, and publishing — all silently in background."
tags: [threads, social-media, playwright, cdp, content-pipeline, posting, automation]
triggers:
  - "post to threads"
  - "publish thread"
  - "threads posting"
  - "content poster"
  - "post 7-part thread"
---

# Threads Poster — Post Multi-Part Threads via Playwright + CDP

## When to Use

When you have a finalized multi-part thread and need to publish it to Threads.com as a proper threaded post — not individual replies.

## Prerequisites

- Chrome running with CDP on port 9222
- Chrome logged into Instagram/Threads as @nunez.chef
- Playwright installed: `pip3 install playwright`
- Thread content as JSON file

## The Reusable Script

**Location:** `~/Documents/carabinerOS/Content/post_thread.py`

```bash
# Dry run (preview without posting)
python3 post_thread.py --dry-run threads/2026-04-04-topic.json

# Live post
python3 post_thread.py threads/2026-04-04-topic.json
```

**JSON format:**
```json
{
  "parts": ["Part 1...", "Part 2...", "..."],
  "image_path": "/path/to/infographic.png",
  "poll": {"options": ["Opt 1", "Opt 2", "Opt 3", "Opt 4"]},
  "post_time": "08:00",
  "status": "approved"
}
```

## Critical Rules (Learned the Hard Way)

### 1. CHARACTER LIMIT: 500 chars per part — HARD LIMIT
If ANY part exceeds ~500 characters, the "Add to thread" button DISAPPEARS for that slot. The script will dump all remaining text into one box. This was misdiagnosed 3 times as a timing bug and a click method bug before identifying it was a content length issue.

**Always validate before posting:**
```python
for i, part in enumerate(parts):
    if len(part) > 500:
        print(f"Part {i+1} is {len(part)} chars — OVER LIMIT")
```

### 2. CLICK METHOD: Playwright native click, NOT JavaScript el.click()
Meta's React event system does NOT reliably handle synthetic JS `el.click()` after the first interaction. This caused "Add to thread" to work once but never again.

**WRONG (fails after first click):**
```python
page.evaluate("""() => {
    for (const el of document.querySelectorAll('*'))
        if (el.textContent.trim() === 'Add to thread')
            { el.click(); return; }
}""")
```

**RIGHT (works every time):**
```python
att = page.get_by_text("Add to thread", exact=True)
att.last.scroll_into_view_if_needed(timeout=3000)
att.last.click(force=True)
```

### 3. POLL: Click icon BEFORE typing text
The poll icon in the compose toolbar disappears once text is entered in that slot. Must click it while the field is empty, fill options, THEN type the post text.

### 4. POLL ICON: Click the grandparent, not the SVG title
The poll icon has `<title>Add a poll</title>` inside an SVG. You can't click the title element. Navigate up to the clickable wrapper:
```python
poll_title = page.locator('svg title:has-text("poll")')
parent = poll_title.first.locator('..')      # SVG
grandparent = parent.locator('..')           # clickable wrapper
grandparent.scroll_into_view_if_needed(timeout=3000)
grandparent.click(force=True)
```

### 5. SCROLL for 5+ parts
The compose modal is ~12,000px tall with 887px viewport. Use `scroll_into_view_if_needed()` before clicking "Add to thread" for parts 5+.

### 6. URL is threads.COM not threads.NET
Threads migrated from threads.net to threads.com. Use `https://www.threads.com`.

### 7. Always create a NEW page
Don't reuse `context.pages[0]` — existing tabs may have dialogs or stale state. Always `context.new_page()` and add a dialog handler:
```python
page = context.new_page()
page.on("dialog", lambda dialog: dialog.dismiss())
```

### 8. Image upload via set_input_files (works minimized)
```python
file_input = page.locator('input[type="file"]')
if file_input.count() > 0:
    file_input.first.set_input_files(image_path)
```
No clipboard, no UI focus needed. Works with Chrome minimized.

### 9. Speed matters — keep timers lean
Aggressive timers (4-5s) are unnecessary. The DOM settles fast:
- After navigate: 2s
- After compose open: 2s  
- After "Add to thread": 1s
- Before typing: 0.3s
- Typing speed: delay=1 (1ms per char)
- After typing: 0.5s
- After post: 3s

## Content Pipeline V2.1

See `~/Documents/carabinerOS/Content/CONTENT_PIPELINE_V2.md`

```
6 AM  — Cron: last30days research
7 AM  — Cron: NotebookLM generates 8 threads + 8 infographics (FREE)
        Sends batch preview to Telegram
YOU   — Reply APPROVE
8AM-8PM — post_scheduler.py posts staggered (every 90 min)
```

**Key architecture decisions:**
- NotebookLM IS the content producer (replaces Paperclip Content Producer agent)
- Infographic style: editorial, portrait, standard detail
- NotebookLM notebook: c07b42fb-06bd-4f26-8286-75a5fb57459c (37 books)
- Content Producer Paperclip agent: RETIRED
- Zero LLM tokens on posting — dumb script reads JSON
- Amazon affiliate tag: estebannunez-20

### Daily Thread Schedule (8 threads)

| # | Time  | Type              | Parts |
|---|-------|-------------------|-------|
| 1 | 08:00 | Research Deep Dive| 7     |
| 2 | 09:30 | Book Highlight    | 4     |
| 3 | 11:00 | Myth Busting      | 3     |
| 4 | 12:30 | Technique Breakdown| 5    |
| 5 | 14:00 | Cross-Reference   | 4     |
| 6 | 15:30 | Quick Tip         | 2     |
| 7 | 17:00 | Book Highlight #2 | 4     |
| 8 | 18:30 | Engagement/Poll   | 3     |

## Threads ≠ Replies (Critical Concept)

A "thread" on Threads is built in the compose modal by clicking "Add to thread" between parts, then published all at once. If you post Part 1 then reply to it, you get disconnected posts — not a visual thread.

## What's NOT Automated Yet

1. **Poll option fields** — icon clicks work, but only 1 of 4 option fields gets filled. Need to click "Add another option" for fields 3-4.
2. **Topic tags** — "Food Threads" topic tag not mapped
3. **Image carousels** — single image works, multi-image not tested

## Pitfalls Summary

| Issue | Root Cause | Solution |
|-------|-----------|----------|
| Only 2 thread slots created | JS el.click() fails with React | Use Playwright native click with force=True |
| "Add to thread" missing | Part text exceeds 500 chars | Validate all parts < 500 chars before posting |
| Create button strict mode | 2 SVG elements match | Use .first |
| Post button behind overlay | Modal overlay intercepts | Use force=True |
| Script brings Chrome to front | Navigation triggers focus | Use domcontentloaded not networkidle |
| Profile verify crashes | Dialog/navigation conflict | Wrap in try/except |
| Poll icon not clickable | SVG title element is invisible | Click grandparent wrapper |
| Poll icon missing | Text already typed in field | Add poll BEFORE typing |

## CRITICAL BUGS (2026-04-04)
- JS el.click() FAILS for "Add to thread" — Meta React ignores synthetic DOM clicks. Use Playwright native click(force=True) with get_by_text("Add to thread", exact=True).last
- scroll_into_view_if_needed() required for 5+ parts (modal is 12,000px+)
- threads.COM not .NET (domain changed)
- Max 500 chars/part or "Add to thread" won't appear
- No polls — a closing question drives replies just as well, and poll automation is fragile (only fills 1 of 4 fields). Skip polls entirely; end threads with an engagement question instead.
- delay=1 for typing. delay=3 too slow. delay=0 drops chars.
- post_thread.py canonical script: ~/Documents/carabinerOS/Content/post_thread.py

## Approved Asset Rule (Critical)

Only post from already-approved JSON payloads that point to the final approved assets.

Approved path pattern:
- `~/Documents/carabinerOS/Content/threads/*.json`
- or the specific approved queue folder created by the real pipeline after Telegram approval

Do NOT improvise or generate substitute payloads/images at post time.

Specifically forbidden unless the user explicitly asks for it:
- `create_sourdough_burst.py`
- ad hoc PIL/generated title cards
- any queue folder full of locally generated text-card images that are not NotebookLM/editorial-approved assets

Why this matters:
- A bad retry posted from `threads/queue/2026-04-05-sourdough-burst/...` using images generated by `create_sourdough_burst.py`
- Those images were simple text cards, not the approved NotebookLM infographic style
- This bypassed the real approval pipeline and produced the wrong visual output

Before posting, verify:
1. JSON was already approved
2. `image_path` points to the intended final asset
3. The image visually matches the approved quality bar (e.g. NotebookLM editorial infographic, like the salt-osmosis post / reference post)
4. You are running `post_thread.py` only — not generating new content or assets

## Platform Details

- Account: @nunez.chef (Threads)
- URL: https://www.threads.com (NOT threads.net)
- Amazon affiliate tag: estebannunez-20
- Chrome CDP: http://localhost:9222
- Threads only — X posting is discontinued

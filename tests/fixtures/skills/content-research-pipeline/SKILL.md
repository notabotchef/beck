---
name: content-research-pipeline
description: "8 threads/day content pipeline (v2.1): 6AM last30days research → 7AM NotebookLM generates 8 varied thread drafts + infographics (FREE) → Telegram batch preview → APPROVE → post_scheduler.py posts staggered 8AM-8PM via Playwright CDP. No Paperclip agents."
tags: [content, research, cron, telegram, notebooklm, last30days, pipeline, threads, playwright]
triggers:
  - "research pipeline"
  - "content pipeline"
  - "daily research"
  - "food science research"
  - "DR_ file"
  - "post thread"
  - "notebooklm content"
  - "8 threads"
  - "batch content"
---

# Content Pipeline V2.1 — 8 Threads/Day, Zero-Token Posting

## Architecture (redesigned 2026-04-04)

```
6:00 AM ── CRON 1: last30days research (dbc3c789c791)
             Output: DR_YYYY-MM-DD.md → ~/Documents/carabinerOS/Content/
             Cost: research API tokens only

7:00 AM ── CRON 2: Batch content generation (3d0f3eaae913)
             1. Read today's DR file
             2. Add as source to NotebookLM Book Collection
             3. Generate 8 reports (varied formats)              FREE
             4. Generate 8 infographics (editorial, portrait)    FREE
             5. Download all 16 artifacts, parse into JSON
             6. Send batch Telegram preview
             Cost: $0.00

Esteban ── Reads preview, replies APPROVE (or APPROVE 1,3,5)

On APPROVE:
             python3 post_scheduler.py threads/queue/YYYY-MM-DD/
             Posts staggered 8AM-8PM via Playwright+CDP (silent)
             Cost: $0.00
```

## Daily Thread Schedule — 8 Threads, 90-Min Gaps

| # | Time  | Type              | Parts | Has Poll |
|---|-------|-------------------|-------|----------|
| 1 | 08:00 | Research Deep Dive | 7     | Yes      |
| 2 | 09:30 | Book Highlight     | 4     | No       |
| 3 | 11:00 | Myth Busting       | 3     | No       |
| 4 | 12:30 | Technique Breakdown| 5     | No       |
| 5 | 14:00 | Cross-Reference    | 4     | No       |
| 6 | 15:30 | Quick Tip          | 2     | No       |
| 7 | 17:00 | Book Highlight #2  | 4     | No       |
| 8 | 18:30 | Engagement / Poll  | 3     | Yes      |

## RETIRED SYSTEMS — DO NOT USE
- Content Producer/Creator/Poster Paperclip agents → RETIRED
- ChatGPT image generation → RETIRED (use NotebookLM infographics)
- X/Twitter posting → RETIRED (Threads only)
- Single-thread pipeline → SUPERSEDED by batch pipeline

## Key Files

```
~/Documents/carabinerOS/Content/
├── CONTENT_PIPELINE_V2.md              ← full architecture doc (549 lines)
├── post_thread.py                      ← single thread poster (369 lines)
├── post_scheduler.py                   ← staggered day scheduler (~200 lines)
├── download_missing_books.sh           ← Anna's Archive downloader
├── DR_YYYY-MM-DD.md                    ← daily research output
└── threads/
    └── queue/
        └── YYYY-MM-DD/                 ← daily batch
            ├── 01-0800-research.json
            ├── 02-0930-book.json
            ├── ...
            ├── 08-1830-engagement.json
            └── images/
                ├── 01-research.png
                └── ...
```

## Thread JSON Format

```json
{
  "parts": ["Part 1...", "Part 2...", ...],
  "image_path": "images/01-research.png",
  "poll": {"options": ["Opt 1", "Opt 2", "Opt 3", "Opt 4"]},
  "post_time": "08:00",
  "type": "research",
  "status": "pending_approval"
}
```

## Script Usage

```bash
# Single thread — preview
python3 post_thread.py --dry-run threads/queue/2026-04-05/01-0800-research.json

# Single thread — post
python3 post_thread.py threads/queue/2026-04-05/01-0800-research.json

# Full day — staggered (waits for each post_time)
python3 post_scheduler.py threads/queue/2026-04-05/

# Full day — post all now (testing, 30s gaps)
python3 post_scheduler.py --now threads/queue/2026-04-05/

# Full day — preview schedule only
python3 post_scheduler.py --dry-run threads/queue/2026-04-05/
```

## NotebookLM Configuration

- Notebook: `c07b42fb-06bd-4f26-8286-75a5fb57459c` (Book Collection, 37 sources)
- Infographic style: **editorial** (portrait, standard detail)
- Esteban rejected sketch_note and scientific as "too cartoonish"
- editorial style takes 10+ minutes to render (longest of all styles)
- Available: auto_select, sketch_note, professional, bento_grid, editorial, instructional, bricks, clay, anime, kawaii, scientific

## Critical Lessons Learned

1. **Agents CANNOT post threads reliably.** Paperclip Content Poster posted 2 of 7 parts, missed images, skipped polls. This happened twice. Never delegate posting to agents — use scripts.

2. **NotebookLM replaces Content Producer AND image generation.** Reports produce thread drafts, infographics produce hero images. Both FREE. This eliminated the most expensive part of the pipeline.

3. **CDP works while Chrome is minimized.** All Playwright interactions go through DevTools Protocol. Only initial Instagram login needs a visible window.

4. **Clipboard DOES NOT work minimized on macOS.** Use `set_input_files()` for images, never `Meta+C`/`Meta+V`.

5. **EPUB→PDF conversion is trivial.** `ebook-convert input.epub output.pdf` (Calibre CLI, `brew install --cask calibre`). All text preserved, NotebookLM indexes it fine.

6. **Anna's Archive fast_download API needs premium membership.** Key alone returns "Not a member". Search works without key (returns MD5 hashes). Download script at `download_missing_books.sh` ready to run once premium activates.

## Cron Jobs

| Job | Schedule | ID | Status |
|-----|----------|----|----|
| Daily Research | 0 6 * * * | dbc3c789c791 | active |
| Batch Content Gen | 0 7 * * * | 3d0f3eaae913 | active |

## Topic Rotation (research cron)

Mon=Fermentation, Tue=Molecular Gastronomy, Wed=Food Safety, Thu=Sustainable Practices, Fri=Restaurant Tech, Sat=Chef Techniques, Sun=Research Breakthroughs

## Book Collection Management

- Current: 37/50 sources. Room for 13 more.
- EPUB→PDF: `ebook-convert input.epub output.pdf`
- Upload: `source_add(notebook_id="c07b42fb-...", source_type="file", file_path="...")`
- Missing priority: Food Lab (Kenji), Noma Fermentation, Hervé This, Harvard Science & Cooking, CookWise, Nose Dive, Art of Fermentation, Ingredient, BakeWise, Science of Good Cooking
- All 10 MD5 hashes found on Anna's Archive, saved in download_missing_books.sh

## Pitfalls

| Issue | Solution |
|-------|----------|
| Agent posts 2 of 7 parts | Use post_thread.py, NEVER agent instructions |
| Infographic too cartoonish | Use editorial style only |
| editorial takes 10+ min | Poll studio_status every 30s, timeout 5 min |
| Image upload fails silently | Check input[type=file] exists before set_input_files |
| Poll selectors change | Multiple fallback strategies in post_thread.py |
| "Add to thread" not found | JS text node scan; abort if not-found |
| Large PDFs timeout on upload | Use wait=false, check status later |
| WD-50 uploaded twice | Delete duplicate via source_delete |
| Anna's Archive "Not a member" | Premium membership required for fast_download API |
| Crons get paused by other sessions | Check with cronjob(action="list", include_disabled=True) |

## Amazon Affiliate

Tag: `estebannunez-20` | Format: `amazon.com/dp/{ASIN}?tag=estebannunez-20`

---
name: last30days
version: "2.9.6"
description: "Deep research engine covering the last 30 days across 10+ sources - Reddit, X/Twitter, YouTube, TikTok, Instagram, Hacker News, Polymarket, and the web. AI synthesizes findings into grounded, cited reports."
argument-hint: 'last30 AI video tools, last30 best project management tools'
allowed-tools: Bash, Read, Write, AskUserQuestion, WebSearch
homepage: https://github.com/mvanhorn/last30days-skill
repository: https://github.com/mvanhorn/last30days-skill
author: mvanhorn
license: MIT
user-invocable: true
metadata:
  openclaw:
    emoji: "📰"
    requires:
      env:
        - SCRAPECREATORS_API_KEY
      optionalEnv:
        - OPENAI_API_KEY
        - XAI_API_KEY
        - OPENROUTER_API_KEY
        - PARALLEL_API_KEY
        - BRAVE_API_KEY
        - APIFY_API_TOKEN
        - AUTH_TOKEN
        - CT0
        - BSKY_HANDLE
        - BSKY_APP_PASSWORD
        - TRUTHSOCIAL_TOKEN
tags: [research, trends, social-media, content-research]
---

# Last 30 Days Research Engine

Deep research across 10+ sources to discover what's trending in the last 30 days. Perfect for staying current on any topic by analyzing real community discussions, betting markets, and viral content.

## Trigger Conditions

Use when you need:
- Recent trends and developments on any topic
- Community sentiment and real discussions (not just news)
- What people are actually upvoting, sharing, and betting on
- Comparative research ("X vs Y")
- Content research for social media or writing
- Understanding current best practices or techniques

## Core Capabilities

### Multi-Source Research
- **Reddit**: Smart subreddit discovery, top comments with upvote weights
- **X/Twitter**: Handle resolution, viral tweet detection  
- **YouTube**: Transcript analysis, engagement metrics
- **TikTok & Instagram**: Reels and viral video content
- **Hacker News**: Technical community discussions
- **Polymarket**: Prediction markets and betting odds
- **Bluesky**: AT Protocol social content
- **Web Search**: Brave/Exa supplemental content

### Intelligence Features
- **Smart Relevance Scoring**: Multi-signal ranking with engagement velocity, authority weighting, cross-platform convergence
- **Comparative Mode**: Side-by-side analysis for "X vs Y" queries
- **Handle Resolution**: Automatic discovery of social media accounts
- **Auto-Save**: Results saved to ~/Documents/Last30Days/
- **Deduplication**: Cross-platform content matching
- **Quality Control**: Blinded evaluation scored 4.38/5.0 vs 3.73 for v1

## Usage Examples

```bash
# Basic research
python scripts/last30days.py "AI video generation tools"

# Comparative analysis  
python scripts/last30days.py "Claude Code vs Codex"

# Quick mode (faster but less thorough)
python scripts/last30days.py "restaurant management software" --quick

# Custom timeframe
python scripts/last30days.py "food science innovations" --days=14

# Focus on specific sources
python scripts/last30days.py "chef techniques" --sources=reddit,youtube
```

## Configuration

### Required Environment Variables
```bash
# Primary requirement - covers Reddit, TikTok, Instagram
export SCRAPECREATORS_API_KEY="your-key-from-scrapecreators.com"
```

### Optional API Keys (for expanded coverage)
```bash
# AI models for synthesis
export OPENAI_API_KEY="your-openai-key"
export XAI_API_KEY="your-xai-key" 
export OPENROUTER_API_KEY="your-openrouter-key"

# Search engines
export BRAVE_API_KEY="your-brave-key"

# Social platforms
export BSKY_HANDLE="your.bsky.social"
export BSKY_APP_PASSWORD="your-app-password"  # Create at bsky.app/settings/app-passwords
```

### Per-Project Config
Create `.claude/last30days.env` in your project root for project-specific API keys:
```bash
# .claude/last30days.env
SCRAPECREATORS_API_KEY=project-specific-key
OPENAI_API_KEY=project-openai-key
```

## Setup Steps

1. **Install Dependencies**
```bash
cd ~/.hermes/skills/research/last30days/scripts
pip install -r requirements.txt  # If requirements file exists
# Or manual install of key dependencies:
pip install requests beautifulsoup4 praw youtube-transcript-api
```

2. **Get ScrapeCreators API Key**
- Visit https://scrapecreators.com
- Sign up and get API key
- Covers Reddit, TikTok, Instagram with one key

3. **Configure Environment**
```bash
# Add to ~/.hermes/.env or ~/.bashrc
export SCRAPECREATORS_API_KEY="your-key"
```

4. **Test Installation**
```bash
python scripts/last30days.py "test topic" --quick
```

## Workflow

### Single Topic Research
1. **Query Expansion**: Converts your topic into optimized search queries
2. **Multi-Source Search**: Searches 8+ platforms in parallel  
3. **Relevance Scoring**: Ranks results using composite scoring
4. **Deduplication**: Removes cross-platform duplicates
5. **Synthesis**: AI writes narrative report with citations
6. **Auto-Save**: Saves complete briefing to ~/Documents/Last30Days/

### Comparative Research  
1. **Topic Detection**: Identifies "X vs Y" pattern
2. **Parallel Research**: Runs 3 research passes (X, Y, combined)
3. **Side-by-Side Analysis**: Strengths, weaknesses, head-to-head
4. **Data-Driven Verdict**: Evidence-based recommendation

## Custom Output Directory (--save-dir)

By default, output goes to `~/Documents/Last30Days/`. To save to a custom directory, use `--save-dir`:

```bash
python scripts/last30days.py "topic" --save-dir="/path/to/output"
```

**PITFALL**: When using this in cron jobs or automated pipelines, you MUST use `--save-dir` explicitly. The default `~/Documents/Last30Days/` path is NOT the same as a project's content directory. A cron job can report "ok" status while writing output to the wrong location (or the agent session may complete without running the script at all).

## Cron Job Setup

When setting up a cron job that runs last30days:

1. The cron prompt MUST include the full `cd` + `python3` command — agent sessions don't automatically know to run the script
2. Use `--save-dir` to write to the desired output directory
3. Use `--emit=compact` for cron (not `md` — compact is more parseable)
4. The script's working directory must be the `scripts/` folder (it uses relative imports from `lib/`)

Example cron prompt pattern:
```
cd ~/.hermes/skills/research/last30days/scripts && python3 last30days.py "<TOPIC>" --save-dir="/target/dir" --emit=compact
```

## ScrapeCreators API Limits

- Free tier has limited credits. When exhausted, Reddit comment enrichment returns **402 Payment Required** errors
- The main search still works (subreddit discovery, post listing) — only comment fetching breaks
- Monitor for 402 errors in output as a signal to top up credits or switch to public Reddit fallback

## Integration with CarabinerOS Content Pipeline

Perfect for:
- **Daily Content Research**: Find trending food science topics
- **Competitive Analysis**: "Sous vide vs air fryer" type comparisons  
- **Industry Trends**: Latest restaurant tech, cooking techniques
- **Social Listening**: What food creators are talking about
- **Market Research**: Consumer sentiment on food trends

### Automated Daily Pipeline (active)
- **Cron job**: `dbc3c789c791` — runs daily at 7:00 AM
- **Output path**: `/Users/estebannunez/Documents/carabinerOS/Content/`
- **Delivery**: Telegram (summary of top findings)
- **Rotating topics**: Fermentation Mon, Molecular Gastronomy Tue, Food Safety Wed, Sustainable Practices Thu, Restaurant Tech Fri, Chef Techniques Sat, Research Breakthroughs Sun

## Performance Notes

- **Execution Time**: 2-8 minutes depending on topic niche
- **Result Quality**: Comprehensive but takes time
- **Quick Mode**: Available for speed over thoroughness
- **Rate Limits**: Built-in handling for API limits
- **Caching**: Smart caching to avoid redundant searches

## Troubleshooting

### Common Issues
```bash
# Missing API key
export SCRAPECREATORS_API_KEY="your-key"

# Python path issues  
export PYTHONPATH="$PWD:$PYTHONPATH"

# Dependencies
pip install --upgrade requests beautifulsoup4

# Clear cache if stale results
rm -rf ~/.cache/last30days/
```

### Quality Issues
- Use `--sources=reddit,youtube` to focus on high-quality sources
- Add more specific keywords if results too broad
- Use comparative mode for better analysis ("X vs Y")

## Security Notes

- API keys stored in environment variables (not committed)
- Per-project .env files for different configurations  
- No credential persistence in cache files
- HTTPS-only API communications

## Output Format

Results include:
- **Executive Summary**: Key findings and trends
- **Source Citations**: Direct links to original content
- **Engagement Metrics**: Upvotes, views, betting odds
- **Quality Scores**: Relevance and authority rankings
- **Temporal Analysis**: How topics evolved over 30 days
- **Cross-Platform Insights**: Different platform perspectives
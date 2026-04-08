---
name: documentation-analysis-and-organization
description: Systematically analyze, audit, and reorganize project documentation folders. Identifies outdated content, categorizes by relevance, and proposes logical restructuring based on user journey and stakeholder needs.
version: 1.0
tags: [documentation, organization, audit, content-strategy]
---

# Documentation Analysis and Organization

## When to Use This Skill

- User asks to review, organize, or clean up a documentation folder
- Project docs are scattered, inconsistent, or mixing multiple purposes
- Documentation contains both current and legacy content
- Need to restructure docs for better user experience
- Documentation audit is needed before a major release or reorganization

## Systematic Approach

### 1. Discovery and Mapping

**Explore the structure first:**
```bash
find ~/path/to/docs -type d  # Get directory structure
find ~/path/to/docs -type f | head -50  # Get file overview
```

**Use search_files to understand content distribution:**
```bash
# Get all files with size/count
search_files(target="files", pattern="*", path="docs/", limit=100)
```

### 2. Content Analysis Strategy

**Read key indicator files to understand context:**
- README.md (main purpose/scope)
- Any roadmap, progress, or status files
- Recent dated files (shows current activity)
- User feedback or bug reports (shows real usage)

**Sample strategically, don't read everything:**
- Start with root-level files
- Read 1-2 files from each major subdirectory
- Focus on recent dates and user-facing content

### 3. Categorization Framework

**Classify content into buckets:**
- ✅ **Current & Relevant** - Actively maintained, project-specific
- ❌ **Generic/Legacy** - Boilerplate, outdated, or from other projects
- ❓ **Mixed/Unclear** - Needs deeper review or unclear purpose

**Look for these signals:**
- **Current:** Recent dates, project-specific names, active bug reports
- **Legacy:** Generic titles, setup instructions for other tools
- **Mixed:** Orchestration docs, implementation guides without context

### 4. User Journey Organization

**Organize by stakeholder needs, not technical structure:**

```
01-product/          # Product vision, competitive analysis
02-market-research/  # User feedback, market validation  
03-development/      # Architecture, roadmap, technical specs
04-go-to-market/     # Demo materials, sales enablement
05-operations/       # Deployment, troubleshooting, agents
_archive/           # Outdated/generic content (don't delete)
```

**Not by file type or technical category.**

**Create README.md for each major section** with overview and key files to help navigation.

### 5. Programmatic Analysis

**Use Python to analyze structure and generate insights:**
```python
import os
from pathlib import Path

# Walk directory structure
docs_path = "/path/to/docs"
structure_analysis = {}

for root, dirs, files in os.walk(docs_path):
    rel_path = os.path.relpath(root, docs_path)
    structure_analysis[rel_path] = {
        'dirs': dirs.copy() if root == docs_path else [],
        'files': [f for f in files if not f.startswith('.')]
    }

# Categorize automatically based on patterns
current_content = []
legacy_content = []
mixed_content = []

# Apply classification rules based on directory names, file patterns
```

### 6. Implementation Strategy

**Three-step execution:**
1. **File Reorganization** - Move files into new structure using mkdir/mv commands
2. **Create New README.md** - Project-specific overview replacing generic content  
3. **Consolidate Intelligence** - Synthesize scattered insights into comprehensive overviews

**Always provide concrete next steps:**
1. Specific files to move/archive
2. New README.md outline focused on current project
3. Priority order for reorganization
4. What to preserve vs. what to remove
5. Section-level README.md files for navigation
6. Intelligence consolidation (market research, competitive analysis)
7. Internal link updates needed

## Common Patterns to Watch For

### Mixed Project Documentation
- **Symptom:** Generic framework docs mixed with project-specific content
- **Solution:** Archive generic content, focus on project-specific value
- **Example:** Agent Zero setup docs in CarabinerOS folder

### Duplicate/Scattered Intelligence  
- **Symptom:** Same content in multiple locations (e.g., simulations/ and MiroShark/)
- **Solution:** Consolidate into single authoritative location with comprehensive overview
- **Action:** Create market intelligence dashboard combining all insights

### Development-Centric Organization
- **Symptom:** Organized by technical concerns (setup/, guides/, specs/)
- **Solution:** Reorganize by user journey and business value
- **Better:** Product vision → Market research → Development → GTM → Operations

### Outdated Entry Points
- **Symptom:** README.md describes wrong product or generic framework
- **Solution:** Rewrite main README for actual current project
- **Include:** Current status, priority stack, key insights from market research

### Valuable Intelligence Buried
- **Symptom:** Market simulations, user feedback, competitive analysis scattered
- **Solution:** Elevate to prominent positions, create synthesis documents
- **Example:** MiroShark prediction market insights consolidated into actionable GTM strategy

## Key Principles

1. **Preserve valuable content** - Don't delete market research, user feedback, or working demos
2. **Organize by user journey** - Product → Market → Development → GTM → Operations
3. **Consolidate duplicates** - Single source of truth for each topic
4. **Update entry points** - README.md should reflect current project reality
5. **Archive don't delete** - Move outdated content to _archive/ rather than deleting
6. **Create navigation aids** - Section-level README.md files with overviews and key file links
7. **Synthesize insights** - Combine scattered intelligence into comprehensive strategy documents
8. **Lead with current status** - Highlight active development, recent progress, immediate priorities

## Validation Questions

Before implementing reorganization:
- Does the new structure serve different stakeholder types?
- Are valuable insights (market research, user feedback) preserved and findable?
- Does the main README accurately represent the current project?
- Are there clear next steps for each content category?

## Pitfalls to Avoid

- **Don't over-read** - Sample strategically rather than reading every file
- **Don't organize by file type** - Focus on user needs, not technical categories  
- **Don't lose valuable intelligence** - Market research and user feedback are often the most valuable content
- **Don't assume recent = important** - Some foundational content may be older but critical
- **Don't reorganize without understanding context** - Read project status and recent activity first
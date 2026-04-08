---
name: a0-plugin-to-repo
description: Extract an Agent Zero plugin from CarabinerOS (usr/plugins/) into a standalone GitHub repo with fixed imports, passing tests, and trendy README. Use when Esteban says "create a repo" for an A0 plugin or wants to publish one.
version: 1.0.0
tags: [agent-zero, plugin, github, packaging, carabiner-os]
related_skills: [github-repo-management, github-auth]
---

# Extract A0 Plugin to Standalone GitHub Repo

## Trigger
- User wants to publish an Agent Zero plugin as a standalone repo
- Plugin currently lives in `~/Projects/carabiner-os/usr/plugins/<name>/`
- User references existing plugins (phantom-bridge, a0_tinyRouter, agent0-terminal) as style guides

## Steps

### 1. Audit the Plugin
```bash
# List source files (exclude .venv)
find ~/Projects/carabiner-os/usr/plugins/<name> -type f -not -path '*/.venv/*' -not -name '*.pyc' | sort
```
Key files to check: `plugin.yaml`, `tools/*.py`, `helpers/*.py`, `prompts/*.md`, `tests/`, `default_config.yaml`, `install.sh`

### 2. Fix Framework Imports for Standalone
A0 plugins import framework modules that won't exist outside A0. Guard them:

```python
# In helpers/*.py files, replace:
from helpers import files
from helpers.print_style import PrintStyle

# With:
try:
    from helpers import files
    from helpers.print_style import PrintStyle
except ImportError:
    files = None
    class PrintStyle:
        @staticmethod
        def standard(msg): print(msg)
        @staticmethod
        def warning(msg): print(f"WARNING: {msg}")
```

Also guard any `files.get_abs_path()` calls:
```python
if files:
    work_dir = files.get_abs_path("usr", "workdir", output_dir)
else:
    work_dir = os.path.join(os.getcwd(), output_dir)
```

### 3. Fix Test Imports
Tests inside A0 use absolute paths. Fix for standalone:

```python
# Replace:
from usr.plugins.<name>.helpers.X import Y
# With:
from helpers.X import Y

# Fix sys.path insert:
# Replace: sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "..", ".."))
# With:    sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))
```

### 4. Run Tests
```bash
# System Python may not satisfy requirements (e.g., langextract needs >=3.10)
# Use the plugin's own .venv if it exists:
source ~/Projects/carabiner-os/usr/plugins/<name>/.venv/bin/activate
cd ~/Projects/<repo-name> && python -m pytest tests/ -v
```

### 5. Create Repo Structure
```
<repo-name>/
├── README.md
├── LICENSE                # MIT, author: Esteban Nunez
├── .gitignore
├── plugin.yaml
├── default_config.yaml
├── install.sh
├── Makefile
├── helpers/
├── tools/
├── prompts/
├── extensions/.gitkeep
├── tests/
│   └── fixtures/
└── docs/
    └── examples.md
```

### 6. README Style (Esteban's Plugin Pattern)
Based on phantom-bridge + a0_tinyRouter + agent0-terminal:

- 🔬 Emoji in title
- Badges row: MIT, Python version, "A0 plugin", powered-by
- Blockquote tagline (one compelling sentence)
- "Why This Exists" — problem statement from restaurant perspective
- Architecture ASCII diagram (input → processing → output)
- Quick Demo (show agent receiving text → JSON output)
- Install section (3 options: one-liner, manual, standalone dev)
- Configuration with YAML code block
- Tools table (| Tool | Purpose |)
- Built-in schemas/features table
- Source grounding / key differentiator section
- Tests section
- Project structure tree
- Contributing section
- License footer mentioning CarabinerOS

### 7. Push to GitHub
```bash
cd ~/Projects/<repo-name>
git init && git add -A
git commit -m "feat: initial release — <description>"
gh repo create Nunezchef/<repo-name> --public --description "<tagline>" --source . --push
gh repo edit --add-topic agent-zero,<relevant-topics>
```

## Pitfalls

| Issue | Solution |
|-------|----------|
| `gh auth` token expired | Run `gh auth login -h github.com --web`, complete device flow |
| System Python too old for plugin deps | Use plugin's `.venv` from CarabinerOS for testing |
| `from helpers import files` fails standalone | Add try/except stub (see step 2) |
| Tests use `usr.plugins.X` absolute imports | Rewrite to relative `from helpers.X` (see step 3) |
| Plugin has its own .venv with thousands of files | Exclude from copy — add `.venv/` to .gitignore |
| `agent.Agent` type hints in plugin code | Use `TYPE_CHECKING` guard (already standard in A0 plugins) |

## Verification
```bash
# Tests pass standalone
cd ~/Projects/<repo-name> && python -m pytest tests/ -v
# Repo accessible
gh repo view Nunezchef/<repo-name> --web
# Plugin still works in CarabinerOS (original untouched)
ls ~/Projects/carabiner-os/usr/plugins/<name>/plugin.yaml
```

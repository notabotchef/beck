---
name: github-pr-security-review
description: Quick security-focused PR review workflow for identifying critical vulnerabilities and prioritizing merge order
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [GitHub, Security, Code-Review, Pull-Requests, Vulnerability]
    related_skills: [github-pr-workflow, github-code-review]
---

# GitHub Security-Focused PR Review

Quick workflow for reviewing multiple open PRs with security prioritization. Use when a user asks about PR safety or merge recommendations.

## When to Use

- User asks "is it safe to merge" for PRs
- Multiple PRs pending and need priority order
- Security vulnerabilities suspected in codebase
- Need quick assessment before merging

## Workflow

### 1. List Open PRs

```bash
cd <project_directory>
gh pr list --state open
```

### 2. Security Pattern Recognition

Look for these high-priority security indicators in PR titles/descriptions:

**CRITICAL (merge immediately):**
- `fix: CWE-` (Common Weakness Enumeration)
- `security:` prefix
- `timing attack`, `injection`, `authentication`
- `hmac.compare_digest`, `constant time`

**HIGH PRIORITY:**
- `auth`, `token`, `password`, `session`
- `validation`, `sanitize`, `escape`
- `CORS`, `XSS`, `CSRF`

**MEDIUM:**
- `fix:` general bug fixes
- Input validation improvements
- Error handling

### 3. Quick Diff Review for Security Issues

For security-critical PRs, examine the diff:

```bash
gh pr diff <PR_NUMBER>
```

**Look for:**
- String comparison → `hmac.compare_digest()` (timing attack fix)
- Input validation additions
- Authentication/authorization changes
- Cryptographic improvements
- SQL injection prevention

### 4. Check CI Status

```bash
gh pr checks <PR_NUMBER>
```

**Security CI patterns:**
- CodeQL/SAST tools passing
- Dependency vulnerability scans
- Security linting tools

### 5. Merge Priority Recommendation

**Template response:**
```
**SAFE TO MERGE:**

1. **PR #XXXX** - **CRITICAL SECURITY FIX** ⚠️
   - [Description of vulnerability]
   - **PRIORITY: Merge immediately**

2. **PR #YYYY** - **Bug Fix**
   - [Description]
   - **SAFE TO MERGE**

**Recommendation**: Merge #XXXX first (security), then #YYYY.
```

## Security Patterns to Recognize

### Timing Attack Fixes
```python
# VULNERABLE
if token != expected_token:
    return False

# FIXED
if not hmac.compare_digest(token, expected_token):
    return False
```

### Input Validation
```python
# VULNERABLE
if not tool_request.get("tool_args"):
    raise ValueError(...)

# FIXED  
if not isinstance(tool_request.get("tool_args"), dict):
    raise ValueError(...)
```

### Authentication Improvements
- Bearer token validation
- Session management
- Password hashing upgrades
- OAuth implementations

## Common Security PR Categories

1. **CWE Fixes** - Numbered Common Weakness Enumeration
2. **Dependency Updates** - Vulnerability patches
3. **Input Validation** - XSS, injection prevention  
4. **Authentication** - Login, tokens, sessions
5. **Cryptographic** - Hashing, encryption improvements
6. **Access Control** - Authorization, permissions

## Quick Decision Matrix

| Security Level | Merge Priority | Wait for CI? |
|---------------|----------------|--------------|
| CRITICAL | Immediate | No (hotfix) |
| HIGH | High | Yes |
| MEDIUM | Normal | Yes |
| LOW | Normal | Yes |

## Example Security Keywords

**Critical:** `CWE-`, `RCE`, `SQLi`, `timing attack`, `authentication bypass`
**High:** `XSS`, `CSRF`, `injection`, `validation`, `sanitize`
**Medium:** `error handling`, `logging`, `rate limiting`

## Pitfalls

- Don't skip security PRs even if CI is missing
- Security fixes often don't have extensive tests
- Focus on the vulnerability being patched, not code style
- Timing attack fixes are always critical regardless of size
- Empty validations can be security issues (allow bypasses)
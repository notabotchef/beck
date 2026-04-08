---
name: github-flagged-account-recovery
description: "Diagnose and recover a GitHub account that's been hidden/flagged by spam detection — profile returns 404 to the public but works when authenticated. Distinct from full suspension."
tags: [github, account, recovery, spam-flag, support-appeal]
triggers:
  - github shadowban
  - github account hidden
  - github profile 404
  - can't see my github
  - github flagged
  - github account suspended
  - my github repos invisible
---

# GitHub Flagged-Account Recovery

## What This Skill Covers

A GitHub account in the **flagged-but-not-suspended** state. The user can still log in, push commits, open PRs, and use the API with their token — but their public profile, repos, and contributions return HTTP 404 to anyone not logged in as them.

This is NOT a full account suspension (which blocks login entirely) and NOT a shadowban in the social-media sense. It's GitHub's automated spam/abuse review state, and it has a specific recovery path.

## Diagnostic Test (run this first)

The signature of a flagged account is an asymmetry between authenticated and anonymous access. Confirm with two curls:

```bash
# Anonymous (unauthenticated) — both should return 200 for a healthy account
curl -s -o /dev/null -w "%{http_code}\n" https://github.com/USERNAME
curl -s -o /dev/null -w "%{http_code}\n" https://api.github.com/users/USERNAME

# Authenticated (run as the flagged user)
gh auth status
gh api user | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['login'], d['public_repos'], 'followers:', d['followers'])"
```

Interpretation:
| Anonymous HTML | Anonymous API | Authenticated | State |
|---|---|---|---|
| 200 | 200 | 200 | Healthy |
| **404** | **404** | **200** | **Flagged — use this skill** |
| 404 | 404 | 401/403 | Fully suspended — different path, appeal form still applies |
| 200 | 200 | 401 | Token expired, not an account issue |

If the flagged row matches, proceed.

## Likely Trigger Patterns

GitHub's spam heuristics flag accounts that look like bots. Common triggers on legitimate accounts:

- Very new account (<12 months) with high repo creation velocity
- 0 followers + 0 following but many public repos
- High commit volume with AI-assisted patterns (identical cadences, template commit messages)
- No verified public email on the profile
- No profile README, no bio, or generic avatar
- Recent rapid activity bursts (e.g. 10+ repos in a week)
- Email or IP associated with previously flagged accounts
- Uploading zip/binary-heavy repos with minimal source code

## Recovery Steps (ordered, do not skip)

### 1. Check email for any GitHub communication
Check the primary inbox AND spam folder for messages from:
- `noreply@github.com`
- `support@github.com`
- `notifications@github.com`

GitHub sometimes sends notice of the flag. If there is a message, the reason it states goes straight into the appeal.

### 2. Harden the profile before filing the appeal
Do these while logged in as the flagged user — they improve the human reviewer's decision:
- github.com/settings/emails — verify a primary email if not already
- github.com/settings/profile — add a real bio, location, company, and a profile photo
- Create a profile README at `github.com/USERNAME/USERNAME` with a one-paragraph about
- Make sure at least one pinned repo has a real README and isn't empty

### 3. File the Appeal and Reinstatement form
Canonical URL: https://support.github.com/contact/account-appeal

This is the ONLY channel that works. Support email does not route correctly for account flags. All decisions are made by humans (per docs.github.com/en/site-policy/acceptable-use-policies/github-appeal-and-reinstatement).

### 4. Draft the appeal message
Use this template (customize the bracketed parts):

```
Subject: Appeal — account [USERNAME] hidden from public (404), still accessible when authenticated

My GitHub account "[USERNAME]" appears to have been flagged by automated systems.
When I am logged in, everything works normally — I can access my [N] public repositories,
my pull requests, and my settings. However, when viewed by anyone not logged in (including
from an incognito window), https://github.com/[USERNAME] returns HTTP 404, and so does
https://api.github.com/users/[USERNAME]. This means my profile, repos, and contributions
are invisible to the public.

I am a legitimate user. [ONE SENTENCE ABOUT WHO YOU ARE AND WHAT YOU BUILD.]
My commits are real engineering work on real projects, some of which use AI-assisted
development (Claude Code, Codex, Copilot), which I believe may have triggered the
automated flag.

I have not received any email notification about a violation. I have reviewed the
Acceptable Use Policies and I do not believe I have violated any of them. I am happy
to make any changes you request and to comply with the policies going forward.

Could you please review the account and either reinstate full public visibility or let
me know what specific change is needed?

Thank you for your time.
[Real Name]
[Email on file]
```

### 5. Wait — do not escalate

Typical resolution: 3 days to ~2 weeks. While waiting:

- Do NOT open multiple tickets (slows the queue down)
- Do NOT create a new GitHub account (can get it flagged too and looks like evasion)
- Do NOT change the username of the flagged account (breaks the appeal trail)
- Do NOT delete repos (appears to be covering tracks)
- Slow down commit velocity significantly — no AI-pattern bursts while the review is pending
- Do NOT post on Twitter/Threads tagging @github about it (rarely helps, sometimes backfires)

### 6. Post-recovery hardening

Once the account is reinstated, prevent re-flagging:
- Keep commits at a sustainable cadence, not bursts
- Mix AI-assisted commits with manual ones
- Write real commit messages, not `update` or `fix` repeatedly
- Star a few real projects, follow real developers
- Keep the profile README current
- Push to a consistent set of repos rather than creating new ones constantly

## References
- Official appeal docs: https://docs.github.com/en/site-policy/acceptable-use-policies/github-appeal-and-reinstatement
- Appeal form: https://support.github.com/contact/account-appeal
- Community case study (same symptoms, resolved via appeal): https://devactivity.com/insights/navigating-github-profile-404s-understanding-account-restrictions-and-their-impact-on-github-code-review-analytics/
- Spam-flag prevention guide: https://wpreset.com/how-to-stop-github-from-flagging-your-account-as-spam

## Known Cases

- **2026-04-07 — Nunezchef** (Esteban Nunez, CarabinerOS): 9-month-old account (created 2025-07-05), 16 public repos, 0 followers / 0 following, AI-assisted commit velocity via Claude Code. Diagnostic matched exactly: `gh api user` → 200 full payload, `curl https://github.com/Nunezchef` → 404, `curl https://api.github.com/users/Nunezchef` → 404. Appeal submitted via support.github.com/contact/account-appeal. Used the template from this skill with the Claude Code disclosure. Likely trigger: 0/0 follower pattern + AI-assisted commit cadence + no verified public email.

## Pitfalls

1. **Do not confuse this with suspension.** Flagged = hidden to public, owner can still log in. Suspended = owner cannot log in at all. Both use the same appeal form but the appeal text differs — flagged users should emphasize the "still works when authenticated" evidence as proof it's an automated false positive.

2. **Do not rely on `gh api user` alone.** A healthy response from the authenticated API is expected even when flagged. You must test the ANONYMOUS endpoints (`curl` without auth) to see the 404.

3. **Do not file from a different email.** The appeal must come from the email on file for the flagged account. Support cannot validate identity otherwise.

4. **"Shadowban" is the wrong word.** The user may describe it as a shadowban because the symptoms look similar, but GitHub does not call it that. Use "flagged" or "account hidden" in the appeal — it matches GitHub's internal vocabulary and routes correctly.

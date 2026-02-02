---
name: finishing-a-development-branch
description: Complete development work on a branch. Use when a feature is ready to merge, create a PR, or be cleaned up.
---

# Finishing a Development Branch

Structured workflow for completing development work.

## Prerequisites

Run the test suite first. **If tests fail, fix them before proceeding.**

```bash
cd vyuber-rust && cargo test
```

## Options (present to user)

1. **Merge locally**: Switch to base branch, pull latest, merge feature branch, verify tests, delete feature branch
2. **Push and create PR**: Push branch, create PR via `gh pr create` with summary and test plan
3. **Keep as-is**: Leave the branch for later
4. **Discard**: Delete the branch (requires explicit confirmation)

## Merge Flow

```bash
git checkout master
git pull origin master
git merge <feature-branch>
cargo test  # verify again
git branch -d <feature-branch>
```

## PR Flow

```bash
git push -u origin <feature-branch>
gh pr create --title "..." --body "..."
```

## Rules

- Never proceed with failing tests
- Never merge without post-merge verification
- Never delete work without explicit "discard" confirmation
- Only remove worktrees after merge or discard (not after PR)

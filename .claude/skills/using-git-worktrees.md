---
name: using-git-worktrees
description: Create isolated git worktrees for parallel development. Use when working on multiple features simultaneously.
---

# Using Git Worktrees

Create isolated worktrees for parallel development without switching branches.

## Process

1. **Select directory**: Check for existing `.worktrees/` or `worktrees/` directory, or ask user
2. **Verify .gitignore**: Ensure the worktree directory is in `.gitignore` before creating
3. **Create worktree**:
   ```bash
   git worktree add .worktrees/<feature-name> -b <branch-name>
   ```
4. **Setup**: Auto-detect project type and install dependencies
5. **Verify**: Run tests to confirm clean baseline

## Safety Checks

- ALWAYS verify the worktree directory is in `.gitignore`
- NEVER proceed if baseline tests fail
- NEVER assume directory locations without checking

## Rust Project Setup

```bash
# After creating worktree
cd .worktrees/<feature-name>
cargo build
cargo test
```

## Cleanup

```bash
git worktree remove .worktrees/<feature-name>
git branch -d <branch-name>  # if merged
```

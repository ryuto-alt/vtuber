---
name: review-implementing
description: Process and implement code review feedback systematically. Use when user provides reviewer comments, PR feedback, code review notes, or asks to implement suggestions from reviews.
---

# Review Feedback Implementation

Systematically process and implement changes based on code review feedback.

## When to Use

- Provides reviewer comments or feedback
- Pastes PR review notes
- Mentions implementing review suggestions
- Says "address these comments" or "implement feedback"
- Shares list of changes requested by reviewers

## Systematic Workflow

### 1. Parse Reviewer Notes

Identify individual feedback items:
- Split numbered lists (1., 2., etc.)
- Handle bullet points or unnumbered feedback
- Extract distinct change requests
- Clarify ambiguous items before starting

### 2. Create Task List

Create actionable tasks from feedback:
- Each feedback item becomes one or more tasks
- Break down complex feedback into smaller tasks
- Make tasks specific and measurable

Example:
```
- Add error handling to stream_key handler
- Fix race condition in chat history access
- Update StreamKeyResponse to include expiry field
- Add integration test for /api/chat endpoint
```

### 3. Implement Changes Systematically

For each task:

**Locate relevant code:**
- Use Grep to search for functions/types
- Use Glob to find files by pattern
- Read current implementation

**Make changes:**
- Use Edit tool for modifications
- Follow project conventions (Rust idioms, Axum patterns)
- Preserve existing functionality unless changing behavior

**Verify changes:**
- Check with `cargo check -p <crate>`
- Run relevant tests: `cargo test -p <crate> -- <filter>`
- Run clippy: `cargo clippy -p <crate>`

**Move to next task**

### 4. Handle Different Feedback Types

**Code changes:**
- Use Edit tool for existing code
- Follow Rust idioms (Result/Option, pattern matching, iterators)
- Maintain consistent style with `cargo fmt`

**New features:**
- Add to existing crate structure
- Add corresponding tests with `#[test]` or `#[tokio::test]`
- Update shared types in `vyuber-shared` if needed

**Error handling improvements:**
- Replace `.unwrap()` with proper error handling
- Use `thiserror` or custom error types
- Ensure fail-closed behavior

**Tests:**
- Write tests as `#[test]` or `#[tokio::test]` functions
- Use descriptive names: `test_stream_key_generation_returns_valid_uuid`
- Place in `mod tests` within the relevant module

**Refactoring:**
- Preserve functionality
- Improve code structure
- Run `cargo test --workspace` to verify no regressions

**Performance:**
- Profile before optimizing
- Consider async patterns (avoid blocking in tokio runtime)
- Check for unnecessary clones or allocations

### 5. Validation

After implementing changes:
- Run affected tests: `cargo test -p <crate>`
- Check for warnings: `cargo clippy --workspace`
- Format code: `cargo fmt --check`
- Verify workspace builds: `cargo build --workspace`

### 6. Communication

Keep user informed:
- Track progress on each item
- Ask for clarification on ambiguous feedback
- Report blockers or challenges
- Summarize changes at completion

## Edge Cases

**Conflicting feedback:**
- Ask user for guidance
- Explain conflict clearly

**Breaking changes required:**
- Notify user before implementing
- Check impact on `vyuber-shared` types (affects both frontend and backend)
- Discuss alternatives

**Tests fail after changes:**
- Fix tests before marking task complete
- Ensure all related tests pass

**Frontend (Leptos/WASM) changes:**
- Verify with `trunk build` for WASM compilation
- Test both frontend and backend if shared types change

## Important Guidelines

- Track progress on each feedback item
- Ask questions for unclear feedback
- Run tests if changes affect tested code
- Follow Rust conventions: `cargo fmt`, `cargo clippy`
- Use conventional commits if creating commits afterward
- When modifying shared types, verify both frontend and backend compile

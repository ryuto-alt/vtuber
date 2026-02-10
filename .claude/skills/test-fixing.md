---
name: test-fixing
description: Run tests and systematically fix all failing tests using smart error grouping. Use when user asks to fix failing tests, mentions test failures, runs test suite and failures occur, or requests to make tests pass.
---

# Test Fixing

Systematically identify and fix all failing tests using smart grouping strategies.

## When to Use

- Explicitly asks to fix tests ("fix these tests", "make tests pass")
- Reports test failures ("tests are failing", "test suite is broken")
- Completes implementation and wants tests passing
- Mentions CI/CD failures due to tests

## Systematic Approach

### 1. Initial Test Run

Run `cargo test` in the workspace root to identify all failing tests.

Analyze output for:
- Total number of failures
- Error types and patterns
- Affected crates/modules

### 2. Smart Error Grouping

Group similar failures by:
- **Error type**: Compilation errors, assertion failures, panic messages, timeout
- **Crate/module**: Same crate causing multiple test failures
- **Root cause**: Missing dependencies, API changes, refactoring impacts

Prioritize groups by:
- Number of affected tests (highest impact first)
- Dependency order (fix shared crate issues before dependent crates)
- Compilation errors before runtime failures

### 3. Systematic Fixing Process

For each group (starting with highest impact):

1. **Identify root cause**
   - Read relevant code
   - Check recent changes with `git diff`
   - Understand the error pattern

2. **Implement fix**
   - Use Edit tool for code changes
   - Follow project conventions
   - Make minimal, focused changes

3. **Verify fix**
   - Run subset of tests for this group:
     ```bash
     # Test a specific crate
     cargo test -p vyuber-backend
     cargo test -p vyuber-shared

     # Test a specific module
     cargo test -p vyuber-backend -- streaming::tests

     # Test a specific function
     cargo test -p vyuber-backend -- test_function_name
     ```
   - Ensure group passes before moving on

4. **Move to next group**

### 4. Fix Order Strategy

**Compilation errors first:**
- Type mismatches
- Missing imports / unresolved references
- Trait implementation issues
- Lifetime/borrow checker errors

**Then shared crate issues:**
- `vyuber-shared` type changes affecting both frontend and backend
- API contract mismatches between crates

**Then runtime failures:**
- Assertion failures
- Panic in unwrap/expect
- Async/tokio runtime issues
- Network/IO test failures

**Finally, edge cases:**
- Timeout issues
- Race conditions in async tests
- Platform-specific failures

### 5. Final Verification

After all groups fixed:
- Run complete test suite: `cargo test --workspace`
- Run clippy for additional checks: `cargo clippy --workspace`
- Verify no regressions
- Check that `cargo build` succeeds for all targets

## Rust-Specific Best Practices

- Fix one group at a time
- Run focused tests after each fix with `cargo test -p <crate> -- <filter>`
- Use `git diff` to understand recent changes
- Look for patterns in failures
- Don't move to next group until current passes
- Keep changes minimal and focused
- Check if `vyuber-shared` changes cascade to other crates
- For async test failures, ensure `#[tokio::test]` is used correctly
- For WASM-related failures, check `wasm-bindgen` compatibility

## Example Workflow

User: "The tests are failing after my refactor"

1. Run `cargo test --workspace` → 12 failures identified
2. Group errors:
   - 5 compilation errors in vyuber-backend (renamed struct field)
   - 4 type mismatches in vyuber-frontend (shared type changed)
   - 3 assertion failures in vyuber-backend (logic change)
3. Fix compilation errors first → Run `cargo test -p vyuber-backend` → Verify
4. Fix shared type issues → Run `cargo test -p vyuber-frontend` → Verify
5. Fix assertion failures → Run targeted tests → Verify
6. Run `cargo test --workspace` → All pass

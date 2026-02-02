---
name: test-driven-development
description: Implement features using test-driven development. Use when building new features or fixing bugs to ensure correctness.
---

# Test-Driven Development (TDD)

**Core rule: No production code without a failing test first.**

## Red-Green-Refactor Cycle

1. **RED**: Write a minimal failing test demonstrating desired behavior
2. **Verify RED**: Confirm the test fails for the *expected* reason (not a typo or compile error)
3. **GREEN**: Write the simplest code that makes the test pass
4. **Verify GREEN**: Ensure the new test passes and no existing tests break
5. **REFACTOR**: Clean up code while keeping all tests green
6. **Repeat**: Move to the next behavior

## Rules

- One behavior per test, with a descriptive name
- Use real code instead of mocks when possible
- Never skip the verification steps
- If code was written before tests: **delete it and start over with TDD**
- Don't keep discarded code "as reference"

## Red Flags (Require Restart)

- Writing code before tests
- Tests passing immediately without failing first
- Unable to explain why a test failed initially
- Rationalizing "just this once"

## For Rust Projects

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behavior_description() {
        // Arrange
        // Act
        // Assert
    }

    #[tokio::test]
    async fn test_async_behavior() {
        // For async code in this project
    }
}
```

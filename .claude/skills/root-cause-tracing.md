---
name: root-cause-tracing
description: Trace execution errors to their true origin. Use when error messages are misleading or the real cause is unclear.
---

# Root Cause Tracing

Trace execution errors back to their genuine origin, especially when error messages are misleading.

## Process

1. **Capture the error**: Record the full error message, stack trace, and context
2. **Identify the symptom vs. the cause**: The error location is often NOT where the bug is
3. **Trace backward**: Follow the data flow from error point back through:
   - Function call chain
   - Data transformations
   - State mutations
   - External inputs (API responses, user input, config)
4. **Find the divergence point**: Where does actual behavior first differ from expected?
5. **Verify**: Confirm the root cause by showing that fixing it eliminates the error

## Techniques

- **Binary search with logging**: Add log statements at midpoints to narrow down where things go wrong
- **State inspection**: Check variable states at each step of the flow
- **Input validation**: Verify inputs at each boundary are what you expect
- **Git bisect**: For regressions, use `git bisect` to find the breaking commit

## For Async/Streaming Systems (this project)

- Check task spawning order and race conditions
- Verify channel senders/receivers are not dropped prematurely
- Inspect tokio runtime behavior and task cancellation
- Check FFmpeg process lifecycle and exit codes

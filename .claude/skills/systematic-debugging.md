---
name: systematic-debugging
description: Debug errors systematically with root cause analysis. Use when encountering bugs, errors, or unexpected behavior.
---

# Systematic Debugging

**Core rule: NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST.**

## Four-Phase Framework (Sequential, Never Skip)

### Phase 1: Root Cause Investigation
- Analyze the error message carefully
- Reproduce the issue consistently
- Review recent changes (git diff, git log)
- Add diagnostic logging in multi-component systems
- Trace data flow backward from the error to its origin

### Phase 2: Pattern Analysis
- Find comparable working implementations in the codebase
- Study reference documentation thoroughly
- Document all differences between working and broken code
- Identify required dependencies and assumptions

### Phase 3: Hypothesis and Testing
- Formulate an explicit theory about the cause
- Test with minimal, single-variable changes
- Abandon approaches that fail immediately

### Phase 4: Implementation
- Write a failing test that demonstrates the bug
- Apply a single fix addressing the root cause
- Verify the fix resolves the issue without side effects

## Critical Rule

**If >= 3 fix attempts fail: STOP and question the architecture.** Do not keep trying more fixes. Re-evaluate whether the approach itself is wrong.

## Red Flags

- Proposing solutions before investigating
- Making multiple simultaneous changes
- "Shotgun debugging" (random changes hoping something works)
- Repeated fix attempts without reconsidering the approach

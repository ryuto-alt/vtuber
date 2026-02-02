---
name: brainstorming
description: Transform ideas into detailed designs through structured questioning. Use when planning new features or exploring approaches.
---

# Brainstorming

Collaborative design process to transform ideas into specifications before implementation.

## Three Phases

### 1. Understanding Phase
- Review the project context
- Ask clarifying questions **one at a time** (not all at once)
- Prefer multiple-choice questions to streamline discussion
- Understand constraints and requirements fully

### 2. Exploration Phase
- Present 2-3 alternative approaches with trade-offs
- Lead with the recommended option and justify why
- Consider: complexity, maintainability, performance, user experience

### 3. Design Documentation
- Present design in bite-sized sections (200-300 words each)
- Validate understanding after each section
- Cover: architecture, components, data flow, error handling, testing

## Post-Design

1. Write the design to `docs/plans/<timestamp>-<feature>.md`
2. Commit to version control
3. Create a worktree for implementation
4. Break down into implementation tasks

## Principles

- Ask one question per message
- YAGNI: ruthlessly remove unnecessary features
- Always explore alternatives before committing
- Validate incrementally
- Design before code

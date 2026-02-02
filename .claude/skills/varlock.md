---
name: varlock
description: Secure environment variable management. Use when handling secrets, API keys, or sensitive configuration.
---

# Varlock - Secure Environment Variable Management

**Secrets must NEVER appear in: terminal output, Claude's context, log files, git commits, or error messages.**

## Rules

1. **Never echo secrets**: Use `varlock load --quiet` instead of `echo $SECRET`
2. **Never read .env directly**: Check schema with `cat .env.schema` instead
3. **Use varlock for validation**: It masks sensitive data while confirming presence/format
4. **Keep secrets out of commands**: Reference variables, don't embed values

## Essential Commands

```bash
varlock load              # Validate env with masked output
varlock run -- <command>  # Inject validated secrets into commands
cat .env.schema           # Safe to review (no actual values)
```

## Schema Annotations

```
# .env.schema
GEMINI_API_KEY=string @sensitive @required @type=string(startsWith=AI)
STREAM_SECRET=string @sensitive @required
DATABASE_URL=string @sensitive=false
```

- `@sensitive` - Redacted everywhere
- `@sensitive=false` - Can appear in logs
- `@required` - Must be present
- `@type=string(startsWith=X)` - Format validation

## When User Asks to Read .env

Decline and suggest using varlock commands instead. Never display secret values directly.

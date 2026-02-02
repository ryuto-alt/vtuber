---
name: defense-in-depth
description: Multi-layered security testing and validation. Use when reviewing security, handling auth, or protecting sensitive data.
---

# Defense in Depth

Apply multi-layered security practices to ensure no single point of failure compromises the system.

## Security Layers to Verify

### 1. Input Validation
- Validate all external inputs (API requests, RTMP streams, WebSocket messages)
- Reject malformed data early at system boundaries
- Use strong typing (Rust's type system is your ally)

### 2. Authentication & Authorization
- Verify stream keys before accepting RTMP connections
- Validate API tokens on every request
- Never trust client-side validation alone

### 3. Secret Management
- Never log secrets (API keys, stream keys)
- Use environment variables, not hardcoded values
- Ensure `.env` files are in `.gitignore`

### 4. Network Security
- Validate CORS configuration
- Use HTTPS in production
- Rate limit API endpoints

### 5. Error Handling
- Never expose internal errors to clients
- Log detailed errors server-side, return generic messages to clients
- Handle all error paths (Rust's `Result` type helps enforce this)

## Checklist for This Project

- [ ] Gemini API key not exposed in frontend or logs
- [ ] Stream keys validated before accepting connections
- [ ] CORS properly restricted (not wildcard in production)
- [ ] Error responses don't leak internal details
- [ ] No secrets in git history

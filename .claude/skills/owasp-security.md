---
name: owasp-security
description: Use when reviewing code for security vulnerabilities, implementing authentication/authorization, handling user input, or discussing web application security. Covers OWASP Top 10:2025, ASVS 5.0, and Agentic AI security (2026).
---

# OWASP Security Best Practices Skill

Apply these security standards when writing or reviewing code.

## Quick Reference: OWASP Top 10:2025

| # | Vulnerability | Key Prevention |
|---|---------------|----------------|
| A01 | Broken Access Control | Deny by default, enforce server-side, verify ownership |
| A02 | Security Misconfiguration | Harden configs, disable defaults, minimize features |
| A03 | Supply Chain Failures | Lock versions, verify integrity, audit dependencies |
| A04 | Cryptographic Failures | TLS 1.2+, AES-256-GCM, Argon2/bcrypt for passwords |
| A05 | Injection | Parameterized queries, input validation, safe APIs |
| A06 | Insecure Design | Threat model, rate limit, design security controls |
| A07 | Auth Failures | MFA, check breached passwords, secure sessions |
| A08 | Integrity Failures | Sign packages, SRI for CDN, safe serialization |
| A09 | Logging Failures | Log security events, structured format, alerting |
| A10 | Exception Handling | Fail-closed, hide internals, log with context |

## Security Code Review Checklist

When reviewing code, check for these issues:

### Input Handling
- [ ] All user input validated server-side
- [ ] Using parameterized queries (not string concatenation)
- [ ] Input length limits enforced
- [ ] Allowlist validation preferred over denylist

### Authentication & Sessions
- [ ] Passwords hashed with Argon2/bcrypt (not MD5/SHA1)
- [ ] Session tokens have sufficient entropy (128+ bits)
- [ ] Sessions invalidated on logout
- [ ] MFA available for sensitive operations

### Access Control
- [ ] Authorization checked on every request
- [ ] Using object references user cannot manipulate
- [ ] Deny by default policy
- [ ] Privilege escalation paths reviewed

### Data Protection
- [ ] Sensitive data encrypted at rest
- [ ] TLS for all data in transit
- [ ] No sensitive data in URLs/logs
- [ ] Secrets in environment/vault (not code)

### Error Handling
- [ ] No stack traces exposed to users
- [ ] Fail-closed on errors (deny, not allow)
- [ ] All exceptions logged with context
- [ ] Consistent error responses (no enumeration)

## Secure Code Patterns (Rust-Focused)

### Rust-Specific Security Concerns
```rust
// CAUTION: Unsafe bypasses safety
unsafe { ptr::read(user_ptr) }

// CAUTION: Release integer overflow
let x: u8 = 255;
let y = x + 1; // Wraps to 0 in release!
// SAFE: Use checked arithmetic
let y = x.checked_add(1).unwrap_or(255);
```
**Watch for:** `unsafe` blocks, FFI calls, integer overflow in release builds, `.unwrap()` on untrusted input

### Axum Error Handling
```rust
// UNSAFE - Exposes internals
async fn handler() -> Result<Json<Data>, String> {
    let data = db_query().map_err(|e| e.to_string())?; // Leaks DB errors!
    Ok(Json(data))
}

// SAFE - Fail-closed, log context
async fn handler() -> Result<Json<Data>, StatusCode> {
    let data = db_query().map_err(|e| {
        tracing::error!("DB query failed: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(data))
}
```

### API Key Protection
```rust
// UNSAFE - Key in URL/logs
let url = format!("https://api.example.com?key={}", api_key);
tracing::info!("Calling API: {url}");

// SAFE - Key in header, redacted logs
let resp = reqwest::Client::new()
    .get("https://api.example.com")
    .header("Authorization", format!("Bearer {}", api_key))
    .send().await?;
tracing::info!("API call completed");
```

### Input Validation (Axum)
```rust
// UNSAFE - No validation
async fn handler(Json(payload): Json<UserInput>) -> impl IntoResponse {
    process(payload) // Trusts any input!
}

// SAFE - Validate at boundary
async fn handler(Json(payload): Json<UserInput>) -> Result<impl IntoResponse, StatusCode> {
    if payload.title.len() > 200 || payload.title.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(process(payload))
}
```

## Agentic AI Security (OWASP 2026)

When building or reviewing AI agent systems, check for:

| Risk | Description | Mitigation |
|------|-------------|------------|
| ASI01: Goal Hijack | Prompt injection alters agent objectives | Input sanitization, goal boundaries, behavioral monitoring |
| ASI02: Tool Misuse | Tools used in unintended ways | Least privilege, fine-grained permissions, validate I/O |
| ASI03: Privilege Abuse | Credential escalation across agents | Short-lived scoped tokens, identity verification |
| ASI04: Supply Chain | Compromised plugins/MCP servers | Verify signatures, sandbox, allowlist plugins |
| ASI05: Code Execution | Unsafe code generation/execution | Sandbox execution, static analysis, human approval |
| ASI06: Memory Poisoning | Corrupted RAG/context data | Validate stored content, segment by trust level |
| ASI07: Agent Comms | Spoofing between agents | Authenticate, encrypt, verify message integrity |
| ASI08: Cascading Failures | Errors propagate across systems | Circuit breakers, graceful degradation, isolation |
| ASI09: Trust Exploitation | Social engineering via AI | Label AI content, user education, verification steps |
| ASI10: Rogue Agents | Compromised agents acting maliciously | Behavior monitoring, kill switches, anomaly detection |

### Agent Security Checklist

- [ ] All agent inputs sanitized and validated
- [ ] Tools operate with minimum required permissions
- [ ] Credentials are short-lived and scoped
- [ ] Third-party plugins verified and sandboxed
- [ ] Code execution happens in isolated environments
- [ ] Agent communications authenticated and encrypted
- [ ] Circuit breakers between agent components
- [ ] Human approval for sensitive operations
- [ ] Behavior monitoring for anomaly detection
- [ ] Kill switch available for agent systems

## VTuber Project Specific Checks

- [ ] API keys (Gemini, Deepgram, Groq) never logged or exposed in responses
- [ ] Stream keys validated and rate-limited
- [ ] RTMP input sanitized before processing
- [ ] WebRTC/WHEP proxy validates upstream responses
- [ ] CORS configured restrictively for production
- [ ] Chat input length-limited and sanitized before AI processing
- [ ] FFmpeg/MediaMTX process commands never include user-controlled strings directly

## Deep Security Analysis Mindset

When reviewing any code, think like a senior security researcher:

1. **Memory Model:** How does the language handle memory? Managed vs manual?
2. **Type System:** Weak typing = type confusion attacks. Look for coercion exploits.
3. **Serialization:** Every language has its pickle/Marshal equivalent. All are dangerous.
4. **Concurrency:** Race conditions, TOCTOU, atomicity failures specific to the threading model.
5. **FFI Boundaries:** Native interop is where type safety breaks down.
6. **Standard Library:** Historic CVEs in std libs.
7. **Package Ecosystem:** Typosquatting, dependency confusion, malicious packages.
8. **Build System:** Script injection during builds.
9. **Runtime Behavior:** Debug vs release differences (Rust overflow, assertions).
10. **Error Handling:** How does the language fail? Silently? With stack traces? Fail-open?

## When to Apply This Skill

Use this skill when:
- Writing authentication or authorization code
- Handling user input or external data
- Implementing cryptography or password storage
- Reviewing code for security vulnerabilities
- Designing API endpoints
- Building AI agent systems
- Configuring application security settings
- Handling errors and exceptions
- Working with third-party dependencies

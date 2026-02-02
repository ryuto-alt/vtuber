---
name: webapp-testing
description: Test web applications using Playwright. Use when the user asks to test, verify, or debug web UI behavior.
---

# Web Application Testing Toolkit

Test local web applications using Python Playwright scripts.

## Workflow

1. **Determine app type**: Static HTML (read file directly) or dynamic webapp (needs server)
2. **Start server if needed**: Use `python scripts/with_server.py` to manage server lifecycle
3. **Reconnaissance first**: Navigate to the page, wait for load, take screenshots, identify selectors
4. **Then act**: Click, fill, assert based on discovered selectors

## Best Practices

- Always launch Chromium in **headless mode**
- Use `page.wait_for_load_state('networkidle')` before inspecting dynamic content
- Use descriptive selectors: `text=`, `role=`, CSS selectors, IDs
- Take screenshots before and after actions for verification
- Run helper scripts with `--help` first to understand their capabilities

## Multi-Server Example

```bash
python scripts/with_server.py \
  --server "cd vyuber-rust && cargo run -p vyuber-backend" --port 3000 \
  -- python your_test.py
```

## Pattern: Reconnaissance Then Action

1. Navigate to URL
2. Wait for network idle
3. Take screenshot to understand current state
4. Identify selectors from page content
5. Perform actions (click, fill, submit)
6. Verify results with assertions or screenshots

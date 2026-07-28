# Dynamic Branch Naming Pattern Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development (recommended) or executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement `format_branch_name_with_pattern` helper in `src/main.rs` to dynamically format branch names based on `naming.pattern` templates while cleaning orphaned separators.

**Architecture:** Implement helper function, wire it into `new_branch_with_mode` for Git and Jujutsu modes, and add comprehensive unit tests.

**Tech Stack:** Rust (Edition 2024), `clap`, `colored`.

---

### Task 1: Implement `format_branch_name_with_pattern` and Integrate into `new_branch_with_mode`

**Files:**
- Modify: `src/main.rs`
- Test: `src/main.rs` (unit tests)

- [ ] **Step 1: Implement `format_branch_name_with_pattern` helper in `src/main.rs`**

```rust
fn format_branch_name_with_pattern(
    pattern: Option<&str>,
    r#type: Option<&str>,
    ticket: Option<&str>,
    slug: &str,
    prefix_sep: &str,
    ticket_sep: &str,
) -> String {
    let template = pattern.unwrap_or("{type}/{ticket}-{slug}");
    let mut result = template.to_string();

    if let Some(t) = r#type {
        result = result.replace("{type}", t);
    } else {
        result = result.replace("{type}", "");
    }

    if let Some(id) = ticket {
        result = result.replace("{ticket}", id);
    } else {
        result = result.replace("{ticket}", "");
    }

    result = result.replace("{slug}", slug);

    let double_prefix = format!("{}{}", prefix_sep, prefix_sep);
    let double_ticket = format!("{}{}", ticket_sep, ticket_sep);
    while !prefix_sep.is_empty() && result.contains(&double_prefix) {
        result = result.replace(&double_prefix, prefix_sep);
    }
    while !ticket_sep.is_empty() && result.contains(&double_ticket) {
        result = result.replace(&double_ticket, ticket_sep);
    }

    let trimmed = result
        .trim_start_matches(prefix_sep)
        .trim_start_matches(ticket_sep)
        .trim_end_matches(prefix_sep)
        .trim_end_matches(ticket_sep);

    if trimmed.is_empty() {
        slug.to_string()
    } else {
        trimmed.to_string()
    }
}
```

- [ ] **Step 2: Update `new_branch_with_mode` in `src/main.rs` to call `format_branch_name_with_pattern`**

Replace `match (r#type, ticket)` blocks in both Git and Jj branches with:
```rust
let pattern_tmpl = config.naming.as_ref().and_then(|n| n.pattern.as_deref());
let mut branch_name = format_branch_name_with_pattern(
    pattern_tmpl,
    r#type,
    ticket,
    &slug,
    prefix_sep,
    ticket_sep,
);
```

- [ ] **Step 3: Add unit tests for `format_branch_name_with_pattern`**

Under `mod tests` in `src/main.rs`:
```rust
#[test]
fn test_format_branch_name_with_pattern() {
    let name = format_branch_name_with_pattern(
        Some("{type}/{ticket}-{slug}"),
        Some("feature"),
        Some("AUTH-101"),
        "login-flow",
        "/",
        "-",
    );
    assert_eq!(name, "feature/AUTH-101-login-flow");

    let no_ticket = format_branch_name_with_pattern(
        Some("{type}/{ticket}-{slug}"),
        Some("feature"),
        None,
        "login-flow",
        "/",
        "-",
    );
    assert_eq!(no_ticket, "feature/login-flow");

    let custom_pattern = format_branch_name_with_pattern(
        Some("{ticket}/{type}-{slug}"),
        Some("feature"),
        Some("JIRA-99"),
        "fix-bug",
        "/",
        "-",
    );
    assert_eq!(custom_pattern, "JIRA-99/feature-fix-bug");
}
```

- [ ] **Step 4: Run `just lint && just test && just install`**

Run: `just lint && just test && just install`
Expected: PASS (0 warnings).

- [ ] **Step 5: Commit & Push to `main` via `jj`**

```bash
jj describe -m "feat: implement dynamic branch naming pattern template engine"
jj bookmark set main -r @
jj git push -b main
```

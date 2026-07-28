# Design Document: Dynamic Branch Naming Pattern Template Engine

**Date**: 2026-07-27  
**Status**: Approved  
**Target Version**: `0.6.0`  

---

## 1. Overview

`branch-buddy` configuration supports a customizable `naming.pattern` template string (e.g., `"{type}/{ticket}-{slug}"` or `"{ticket}/{type}-{slug}"`).

The dynamic pattern engine substitutes `{type}`, `{ticket}`, and `{slug}` tokens into the branch name, while gracefully stripping unused optional tags and their adjacent separators (`prefix_separator` and `ticket_separator`) to prevent invalid branch names (such as `/AUTH-101-fix` or `feature/--fix`).

---

## 2. Dynamic Template Formatting Algorithm

```rust
pub fn format_branch_name_with_pattern(
    pattern: Option<&str>,
    r#type: Option<&str>,
    ticket: Option<&str>,
    slug: &str,
    prefix_sep: &str,
    ticket_sep: &str,
) -> String {
    let template = pattern.unwrap_or("{type}/{ticket}-{slug}");

    let mut result = template.to_string();

    // 1. Substitute provided tags
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

    // 2. Clean up orphaned separators
    // Collapse duplicate separators (e.g. "//" -> "/", "--" -> "-")
    let double_prefix = format!("{}{}", prefix_sep, prefix_sep);
    let double_ticket = format!("{}{}", ticket_sep, ticket_sep);
    while result.contains(&double_prefix) {
        result = result.replace(&double_prefix, prefix_sep);
    }
    while result.contains(&double_ticket) {
        result = result.replace(&double_ticket, ticket_sep);
    }

    // Trim leading/trailing separators
    result = result
        .trim_start_matches(prefix_sep)
        .trim_start_matches(ticket_sep)
        .trim_end_matches(prefix_sep)
        .trim_end_matches(ticket_sep)
        .to_string();

    if result.is_empty() {
        slug.to_string()
    } else {
        result
    }
}
```

---

## 3. Examples & Test Cases

| Pattern | Type | Ticket | Slug | Output |
|---|---|---|---|---|
| `{type}/{ticket}-{slug}` | `feature` | `AUTH-101` | `login-flow` | `feature/AUTH-101-login-flow` |
| `{type}/{ticket}-{slug}` | `feature` | `None` | `login-flow` | `feature/login-flow` |
| `{type}/{ticket}-{slug}` | `None` | `AUTH-101` | `login-flow` | `AUTH-101-login-flow` |
| `{ticket}/{slug}` | `None` | `JIRA-99` | `fix-bug` | `JIRA-99/fix-bug` |
| `{slug}` | `None` | `None` | `my-branch` | `my-branch` |

---

## 4. Testing & Validation Plan

1. **Unit Tests**:
   - Test `format_branch_name_with_pattern` across all tag combination scenarios.
   - Verify separator cleanup.
2. **Clippy & Build**:
   - Run `just lint && just test && just install`.

---
allowed-tools: Bash, Read, Write, Edit
argument-hint: [service] [resource] [method] [flags]
description: Gen3 shared guidance for auth, environment, and cross-service execution.
---

# Gen3 Shared

Use this skill before invoking a Gen3 service skill.

## Purpose

This shared skill captures the common operating model for the future Gen3 CLI: authenticated, profile-aware, service-oriented commands that run against a target Gen3 commons.

## Expected Environment

- A target commons base URL is selected, such as `GEN3_API_BASE`
- A profile or context is selected for the active user or automation identity
- Authentication has been completed before calling protected APIs
- Service-specific skills are used for endpoint details and workflow nuance

## Command Shape

The long-term command shape should mirror the service-first `gws` pattern:

```bash
gen3 <service> <resource> <method> [flags]
```

Examples:

```bash
gen3 fence auth login
gen3 indexd records get --guid dg.1234
gen3 sheepdog submission export --project program-project
```

## Cross-Service Rules

1. Confirm the target commons and user profile before mutating data.
2. Prefer explicit resource and method names over overloaded shortcuts.
3. Surface request IDs, status codes, and response summaries for operator visibility.
4. When a task crosses services, start with the source-of-truth API skill, then follow linked services.
5. Treat authz-sensitive and destructive operations as confirmation-worthy in the TUI.

## Common Concerns

- AuthN and AuthZ often span Fence and Arborist.
- Structured metadata workflows often span Sheepdog, Peregrine, and Guppy.
- File workflows often span Indexd, Metadata Service, and Manifest Service.
- Workspace workflows often span Hatchery and Workspace Token Service.
- Compliance and review workflows often rely on Audit Service and Requestor.

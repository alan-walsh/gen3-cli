---
allowed-tools: Bash, Read, Write, Edit
argument-hint: "[resource] [method] [flags]"
description: "Gen3 Hatchery: Workspace launch and runtime session operations."
---

# Gen3 Hatchery

Execute Gen3 Hatchery operations: $ARGUMENTS

## Prerequisites

- Read `../gen3-shared/SKILL.md` first for shared auth and environment guidance.
- Confirm the target commons URL and active profile before making API calls.
- Inspect the service contract or OpenAPI/GraphQL schema before implementing new CLI bindings.

## Command Shape

```bash
gen3 hatchery <resource> <method> [flags]
```

## Service Summary

Hatchery manages on-demand workspace and container launches in Gen3 environments. It is often the API edge for notebook and app session lifecycle actions.

## Initial API Areas

These are starting buckets for the CLI and skill design. They should be refined against the service's actual API surface as implementation continues.

### sessions

- Launch, inspect, resume, and terminate user sessions or pods.
### containers

- Select workspace images, apps, or container runtime profiles.
### status

- Check workspace health, readiness, and runtime state.
### access

- Validate user entitlement, paymodel context, and launch permissions.

## Usage Notes

1. Inspect the live service schema or existing service docs before finalizing arguments.
2. Keep the CLI shape resource-first so Ratatui flows can drill from service -> resource -> method.
3. Prefer explicit flags for identifiers like GUIDs, project IDs, request IDs, or token scopes.
4. Return concise summaries in the TUI, with full payload details available on demand.

## Task

Use this skill to plan or execute Gen3 Hatchery work in the Gen3 CLI. Start by identifying the target resource, the method to call, the required identifiers, and any auth or policy implications.

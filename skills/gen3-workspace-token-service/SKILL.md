---
allowed-tools: Bash, Read, Write, Edit
argument-hint: [resource] [method] [flags]
description: Gen3 Workspace Token Service: Workspace credential and access-token operations.
---

# Gen3 Workspace Token Service

Execute Gen3 Workspace Token Service operations: $ARGUMENTS

## Prerequisites

- Read `../gen3-shared/SKILL.md` first for shared auth and environment guidance.
- Confirm the target commons URL and active profile before making API calls.
- Inspect the service contract or OpenAPI/GraphQL schema before implementing new CLI bindings.

## Command Shape

```bash
gen3 workspace-token-service <resource> <method> [flags]
```

## Service Summary

Workspace Token Service issues short-lived credentials and related workspace access artifacts for tools running inside Gen3 workspaces.

## Initial API Areas

These are starting buckets for the CLI and skill design. They should be refined against the service's actual API surface as implementation continues.

### tokens

- Issue, inspect, and refresh workspace-scoped access tokens.
### credentials

- Broker cloud credentials or signed access for workspace tasks.
### paymodels

- Resolve billing or paymodel context for workspace actions.
### integrations

- Coordinate workspace identity for mounted services and external tools.

## Usage Notes

1. Inspect the live service schema or existing service docs before finalizing arguments.
2. Keep the CLI shape resource-first so Ratatui flows can drill from service -> resource -> method.
3. Prefer explicit flags for identifiers like GUIDs, project IDs, request IDs, or token scopes.
4. Return concise summaries in the TUI, with full payload details available on demand.

## Task

Use this skill to plan or execute Gen3 Workspace Token Service work in the Gen3 CLI. Start by identifying the target resource, the method to call, the required identifiers, and any auth or policy implications.

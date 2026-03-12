---
allowed-tools: Bash, Read, Write, Edit
argument-hint: [resource] [method] [flags]
description: Gen3 Audit Service: Audit event query and export operations.
---

# Gen3 Audit Service

Execute Gen3 Audit Service operations: $ARGUMENTS

## Prerequisites

- Read `../gen3-shared/SKILL.md` first for shared auth and environment guidance.
- Confirm the target commons URL and active profile before making API calls.
- Inspect the service contract or OpenAPI/GraphQL schema before implementing new CLI bindings.

## Command Shape

```bash
gen3 audit-service <resource> <method> [flags]
```

## Service Summary

Audit Service exposes platform audit records so operators and administrators can inspect events, trace access, and export audit history.

## Initial API Areas

These are starting buckets for the CLI and skill design. They should be refined against the service's actual API surface as implementation continues.

### events

- List and inspect individual audit events and their envelopes.
### query

- Filter audit history by user, action, service, resource, or time window.
### exports

- Generate bulk exports or reports for compliance workflows.
### admin

- Service health, retention, and administrative audit operations.

## Usage Notes

1. Inspect the live service schema or existing service docs before finalizing arguments.
2. Keep the CLI shape resource-first so Ratatui flows can drill from service -> resource -> method.
3. Prefer explicit flags for identifiers like GUIDs, project IDs, request IDs, or token scopes.
4. Return concise summaries in the TUI, with full payload details available on demand.

## Task

Use this skill to plan or execute Gen3 Audit Service work in the Gen3 CLI. Start by identifying the target resource, the method to call, the required identifiers, and any auth or policy implications.

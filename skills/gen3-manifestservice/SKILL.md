---
allowed-tools: Bash, Read, Write, Edit
argument-hint: [resource] [method] [flags]
description: Gen3 Manifest Service: Manifest generation and retrieval operations.
---

# Gen3 Manifest Service

Execute Gen3 Manifest Service operations: $ARGUMENTS

## Prerequisites

- Read `../gen3-shared/SKILL.md` first for shared auth and environment guidance.
- Confirm the target commons URL and active profile before making API calls.
- Inspect the service contract or OpenAPI/GraphQL schema before implementing new CLI bindings.

## Command Shape

```bash
gen3 manifestservice <resource> <method> [flags]
```

## Service Summary

Manifest Service creates and reads manifests for batches of files, often bridging metadata discovery to download workflows.

## Initial API Areas

These are starting buckets for the CLI and skill design. They should be refined against the service's actual API surface as implementation continues.

### manifests

- Create, fetch, and inspect file manifests and manifest metadata.
### exports

- Generate delivery artifacts for downstream download or transfer flows.
### inputs

- Resolve GUID lists, query results, or file selections into manifest requests.
### status

- Track manifest job state and output locations for long-running requests.

## Usage Notes

1. Inspect the live service schema or existing service docs before finalizing arguments.
2. Keep the CLI shape resource-first so Ratatui flows can drill from service -> resource -> method.
3. Prefer explicit flags for identifiers like GUIDs, project IDs, request IDs, or token scopes.
4. Return concise summaries in the TUI, with full payload details available on demand.

## Task

Use this skill to plan or execute Gen3 Manifest Service work in the Gen3 CLI. Start by identifying the target resource, the method to call, the required identifiers, and any auth or policy implications.

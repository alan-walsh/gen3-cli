---
allowed-tools: Bash, Read, Write, Edit
argument-hint: [resource] [method] [flags]
description: Gen3 Guppy: Search, aggregation, and download-oriented query operations.
---

# Gen3 Guppy

Execute Gen3 Guppy operations: $ARGUMENTS

## Prerequisites

- Read `../gen3-shared/SKILL.md` first for shared auth and environment guidance.
- Confirm the target commons URL and active profile before making API calls.
- Inspect the service contract or OpenAPI/GraphQL schema before implementing new CLI bindings.

## Command Shape

```bash
gen3 guppy <resource> <method> [flags]
```

## Service Summary

Guppy serves search and aggregation workflows over indexed Gen3 metadata. It typically powers discovery views, cohort-style queries, and export helpers.

## Initial API Areas

These are starting buckets for the CLI and skill design. They should be refined against the service's actual API surface as implementation continues.

### query

- Search records with filters, pagination, sorting, and field selection.
### aggregation

- Facets, buckets, counts, and statistics over indexed fields.
### download

- Manifest-like result shaping and file-oriented export preparation.
### configuration

- Inspect available indices, fields, and query capabilities.

## Usage Notes

1. Inspect the live service schema or existing service docs before finalizing arguments.
2. Keep the CLI shape resource-first so Ratatui flows can drill from service -> resource -> method.
3. Prefer explicit flags for identifiers like GUIDs, project IDs, request IDs, or token scopes.
4. Return concise summaries in the TUI, with full payload details available on demand.

## Task

Use this skill to plan or execute Gen3 Guppy work in the Gen3 CLI. Start by identifying the target resource, the method to call, the required identifiers, and any auth or policy implications.

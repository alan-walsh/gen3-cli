---
allowed-tools: Bash, Read, Write, Edit
argument-hint: "[resource] [method] [flags]"
description: "Gen3 Metadata Service: GUID metadata document operations."
---

# Gen3 Metadata Service

Execute Gen3 Metadata Service operations: $ARGUMENTS

## Prerequisites

- Read `../gen3-shared/SKILL.md` first for shared auth and environment guidance.
- Confirm the target commons URL and active profile before making API calls.
- Inspect the service contract or OpenAPI/GraphQL schema before implementing new CLI bindings.

## Command Shape

```bash
gen3 metadata-service <resource> <method> [flags]
```

## Service Summary

The Metadata Service stores and returns JSON metadata associated with GUIDs or other objects. It complements index records with flexible document-style metadata.

## Initial API Areas

These are starting buckets for the CLI and skill design. They should be refined against the service's actual API surface as implementation continues.

### documents

- Create, replace, patch, and retrieve metadata documents.
### query

- Search metadata using GUIDs, key/value filters, or document fields.
### versions

- Track document revisions and update history where supported.
### admin

- Bulk operations, schema expectations, and service administration tasks.

## Usage Notes

1. Inspect the live service schema or existing service docs before finalizing arguments.
2. Keep the CLI shape resource-first so Ratatui flows can drill from service -> resource -> method.
3. Prefer explicit flags for identifiers like GUIDs, project IDs, request IDs, or token scopes.
4. Return concise summaries in the TUI, with full payload details available on demand.

## Task

Use this skill to plan or execute Gen3 Metadata Service work in the Gen3 CLI. Start by identifying the target resource, the method to call, the required identifiers, and any auth or policy implications.

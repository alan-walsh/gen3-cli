---
allowed-tools: Bash, Read, Write, Edit
argument-hint: [resource] [method] [flags]
description: Gen3 Indexd: GUID, DID, and object index record operations.
---

# Gen3 Indexd

Execute Gen3 Indexd operations: $ARGUMENTS

## Prerequisites

- Read `../gen3-shared/SKILL.md` first for shared auth and environment guidance.
- Confirm the target commons URL and active profile before making API calls.
- Inspect the service contract or OpenAPI/GraphQL schema before implementing new CLI bindings.

## Command Shape

```bash
gen3 indexd <resource> <method> [flags]
```

## Service Summary

Indexd tracks globally addressable file records in Gen3. It is used for GUID lifecycle operations, URL registration, metadata updates, and object lookup.

## Initial API Areas

These are starting buckets for the CLI and skill design. They should be refined against the service's actual API surface as implementation continues.

### records

- Create, read, update, and delete index records and versions.
### identifiers

- Work with GUIDs, DIDs, aliases, hashes, and logical identifiers.
### urls

- Register, inspect, and rotate storage URLs and locations.
### metadata

- Manage ACLs, authz, size, checksum, and file-level metadata.

## Usage Notes

1. Inspect the live service schema or existing service docs before finalizing arguments.
2. Keep the CLI shape resource-first so Ratatui flows can drill from service -> resource -> method.
3. Prefer explicit flags for identifiers like GUIDs, project IDs, request IDs, or token scopes.
4. Return concise summaries in the TUI, with full payload details available on demand.

## Task

Use this skill to plan or execute Gen3 Indexd work in the Gen3 CLI. Start by identifying the target resource, the method to call, the required identifiers, and any auth or policy implications.

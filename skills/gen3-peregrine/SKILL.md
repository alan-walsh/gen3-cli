---
allowed-tools: Bash, Read, Write, Edit
argument-hint: [resource] [method] [flags]
description: Gen3 Peregrine: GraphQL query and metadata traversal operations.
---

# Gen3 Peregrine

Execute Gen3 Peregrine operations: $ARGUMENTS

## Prerequisites

- Read `../gen3-shared/SKILL.md` first for shared auth and environment guidance.
- Confirm the target commons URL and active profile before making API calls.
- Inspect the service contract or OpenAPI/GraphQL schema before implementing new CLI bindings.

## Command Shape

```bash
gen3 peregrine <resource> <method> [flags]
```

## Service Summary

Peregrine provides graph-oriented query access over submitted metadata. It is commonly used for GraphQL exploration, record lookup, and connected-data traversal.

## Initial API Areas

These are starting buckets for the CLI and skill design. They should be refined against the service's actual API surface as implementation continues.

### graphql

- Run GraphQL queries, mutations when enabled, and schema introspection.
### nodes

- Lookup records and traverse node relationships in the graph model.
### aggregations

- Retrieve counts, facets, and grouped metadata summaries.
### schema

- Inspect queryable types, fields, and graph relationships.

## Usage Notes

1. Inspect the live service schema or existing service docs before finalizing arguments.
2. Keep the CLI shape resource-first so Ratatui flows can drill from service -> resource -> method.
3. Prefer explicit flags for identifiers like GUIDs, project IDs, request IDs, or token scopes.
4. Return concise summaries in the TUI, with full payload details available on demand.

## Task

Use this skill to plan or execute Gen3 Peregrine work in the Gen3 CLI. Start by identifying the target resource, the method to call, the required identifiers, and any auth or policy implications.

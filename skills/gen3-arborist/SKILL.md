---
allowed-tools: Bash, Read, Write, Edit
argument-hint: [resource] [method] [flags]
description: Gen3 Arborist: Policy, resource, and authorization graph operations.
---

# Gen3 Arborist

Execute Gen3 Arborist operations: $ARGUMENTS

## Prerequisites

- Read `../gen3-shared/SKILL.md` first for shared auth and environment guidance.
- Confirm the target commons URL and active profile before making API calls.
- Inspect the service contract or OpenAPI/GraphQL schema before implementing new CLI bindings.

## Command Shape

```bash
gen3 arborist <resource> <method> [flags]
```

## Service Summary

Arborist is the Gen3 policy engine. It manages resources, roles, users, and policy checks that determine access across the platform.

## Initial API Areas

These are starting buckets for the CLI and skill design. They should be refined against the service's actual API surface as implementation continues.

### policies

- Policy creation, lookup, evaluation, and lifecycle management.
### resources

- Resource tree management for programs, projects, and protected assets.
### roles

- Role bindings, grants, revocations, and relationship inspection.
### authz

- Authorization checks and effective-permission inspection for a subject or token.

## Usage Notes

1. Inspect the live service schema or existing service docs before finalizing arguments.
2. Keep the CLI shape resource-first so Ratatui flows can drill from service -> resource -> method.
3. Prefer explicit flags for identifiers like GUIDs, project IDs, request IDs, or token scopes.
4. Return concise summaries in the TUI, with full payload details available on demand.

## Task

Use this skill to plan or execute Gen3 Arborist work in the Gen3 CLI. Start by identifying the target resource, the method to call, the required identifiers, and any auth or policy implications.

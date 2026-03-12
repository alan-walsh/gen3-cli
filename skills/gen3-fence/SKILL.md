---
allowed-tools: Bash, Read, Write, Edit
argument-hint: [resource] [method] [flags]
description: Gen3 Fence: AuthN/AuthZ, identity, and token operations.
---

# Gen3 Fence

Execute Gen3 Fence operations: $ARGUMENTS

## Prerequisites

- Read `../gen3-shared/SKILL.md` first for shared auth and environment guidance.
- Confirm the target commons URL and active profile before making API calls.
- Inspect the service contract or OpenAPI/GraphQL schema before implementing new CLI bindings.

## Command Shape

```bash
gen3 fence <resource> <method> [flags]
```

## Service Summary

Fence is the Gen3 identity and access gateway. It covers login flows, tokens, users, clients, and the auth edges that other Gen3 services rely on.

## Initial API Areas

These are starting buckets for the CLI and skill design. They should be refined against the service's actual API surface as implementation continues.

### auth

- Login flows, token exchange, refresh, revoke, and session-oriented operations.
### users

- User profile lookup, linked identities, and account management flows.
### credentials

- API keys, client credentials, and downstream access tokens.
### admin

- Administrative client, user, and provider configuration operations.

## Usage Notes

1. Inspect the live service schema or existing service docs before finalizing arguments.
2. Keep the CLI shape resource-first so Ratatui flows can drill from service -> resource -> method.
3. Prefer explicit flags for identifiers like GUIDs, project IDs, request IDs, or token scopes.
4. Return concise summaries in the TUI, with full payload details available on demand.

## Task

Use this skill to plan or execute Gen3 Fence work in the Gen3 CLI. Start by identifying the target resource, the method to call, the required identifiers, and any auth or policy implications.

---
allowed-tools: Bash, Read, Write, Edit
argument-hint: "[resource] [method] [flags]"
description: "Gen3 Peregrine: GraphQL query and metadata traversal operations."
---

# Gen3 Peregrine

Execute Gen3 Peregrine operations: $ARGUMENTS

## Prerequisites

- Read `../gen3-shared/SKILL.md` first for shared auth and environment guidance.
- Confirm the target commons URL and active profile before making API calls.
- Read access to non-public projects requires a Bearer token from Fence.

## Command Shape

```bash
gen3 peregrine <resource> <method> [flags]
```

## Service Summary

Peregrine is Gen3's GraphQL read-only query service. It provides graph-oriented access to submitted metadata stored in PostgreSQL, schema introspection, per-project node counts, and core metadata lookup for data objects. It is the primary interface for discovering and traversing metadata in a Gen3 commons.

---

## URL / Routing

In a standard Gen3 deployment, Peregrine is behind the reverse proxy at `/api/`. The submission blueprint is mounted at `/v0/submission/`:

```
{api_endpoint}/api/v0/submission/graphql     ← GraphQL query endpoint
{api_endpoint}/api/v0/submission/getschema   ← Data dictionary schema
{api_endpoint}/api/datasets                  ← Dataset node counts
{api_endpoint}/api/datasets/projects         ← Project listing
{api_endpoint}/api/{object_id}               ← Core metadata lookup
{api_endpoint}/api/_status                   ← Health check
{api_endpoint}/api/_version                  ← Version info
```

## Authentication

Read queries for projects with `availability_type == "Open"` are publicly accessible. All other metadata queries require a Bearer token:

```
Authorization: Bearer <access_token>
```

Obtain the access token by exchanging the stored API key:

```
POST {api_endpoint}/user/credentials/api/access_token
{ "api_key": "<profile.api_key>" }
```

---

## Resource Reference

### graphql

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `graphql query` | POST | `/api/v0/submission/graphql` | Bearer (for protected projects) |
| `graphql introspect` | POST | `/api/v0/submission/graphql` | Bearer (for protected projects) |
| `graphql schema` | GET | `/api/v0/submission/getschema` | — |

#### `POST /api/v0/submission/graphql` — run a GraphQL query

Request body (`application/json`):

```json
{
  "query": "{ project(first: 10) { project_id } }",
  "variables": null,
  "operationName": null
}
```

**Response**:
```json
{ "data": { "project": [ { "project_id": "program-project" } ] } }
```

Common query examples:

```graphql
# List all projects
{ project(first: 100) { project_id } }

# Count cases per project
{ project { project_id _case_count } }

# Get cases in a project with filters
{
  case(project_id: "MyProgram-MyProject", first: 20) {
    submitter_id
    disease_type
    primary_site
  }
}

# Count files by data type
{ datanode(first: 0) { _count } }
```

Variables must be passed as a JSON-encoded string if used.

#### `GET /api/v0/submission/getschema` — get data dictionary schema

Returns the full JSON schema for the data dictionary, including all node types, fields, and relationships. Useful for building queries and understanding the data model.

#### `graphql introspect` — introspect the GraphQL schema

Runs a standard GraphQL introspection query to list all queryable types and their fields. Useful for discovering available nodes before writing queries.

---

### datasets

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `datasets list` | GET | `/api/datasets/projects` | Bearer |
| `datasets counts` | GET | `/api/datasets` | Bearer |

#### `GET /api/datasets/projects` — list all projects

Returns high-level information for all projects the user has access to.

**Response fields per project**: `name`, `code`, `dbgap_accession_number`, `description`, `image_url`.

#### `GET /api/datasets` — get node counts per project

Query params (optional): `nodes` — comma-delimited list of node types to count (e.g. `case,aliquot,file`).

**Response**: `{ "project_A": { "case": 12, "aliquot": 45 }, "project_B": { ... } }`

---

### metadata

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `metadata get` | GET | `/api/{object_id}` | Bearer |

#### `GET /api/{object_id}` — get core metadata for an object

Returns core metadata for the given object_id (GUID). The `Accept` header controls the response format.

Supported formats:
- `json` (default) — standard JSON object
- `schema-org` — Schema.org JSON-LD (`application/vnd.schemaorg.ld+json`)
- `bibtex` — BibTeX citation string (`x-bibtex`)

**Response fields (JSON)**: `object_id`, `file_name`, `data_format`, `file_size`.

---

### system

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `system status` | GET | `/api/_status` | — |
| `system version` | GET | `/api/_version` | — |

Use `system status` for health checks; `200` means Peregrine is healthy and connected to PostgreSQL.

`system version` returns `commit`, `version`, and `dictionary` (commit + version of the data dictionary).

---

## Common Error Codes

| Code | Meaning |
|---|---|
| 200 | Success |
| 400 | Bad request / invalid GraphQL query |
| 401 | Unauthorized — token missing or expired |
| 403 | Forbidden — insufficient permissions (returned in GraphQL `errors` field) |
| 404 | Object not found (core metadata) |
| 500 | Internal server error |

Note: GraphQL errors are returned inside the `errors` field of a `200` response body, not as HTTP error codes.

---

## Typical Agent Workflow

1. **Load profile** from `~/.gen3/config`; read `api_endpoint`, `api_key`.
2. **Exchange** `api_key` → `access_token` via `POST {api_endpoint}/user/credentials/api/access_token`.
3. **Discover** available node types with `graphql introspect` or `graphql schema`.
4. **Query** with `graphql query --query "{ ... }"`, attaching `Authorization: Bearer <access_token>`.
5. **On GraphQL errors**: inspect the `errors` array in the response body (HTTP status is still `200`).

---

## Usage Notes

1. Peregrine is read-only — it has no mutation endpoints.
2. GraphQL queries use Gen3's graph model; node names match the data dictionary (e.g. `case`, `sample`, `aliquot`, `submitted_unaligned_reads`).
3. Use `first: N` to paginate; add `offset: N` or filter by `project_id` to narrow results.
4. The `_case_count`, `_file_count`, and `_<node>_count` virtual fields are available on project nodes for quick counts.
5. For schema discovery, `graphql schema` returns the full data dictionary; `graphql introspect` returns the live GraphQL type system.
6. Core metadata (`metadata get`) uses the same GUID as Indexd but returns a richer citation-ready payload.

---
allowed-tools: Bash, Read, Write, Edit
argument-hint: "[resource] [method] [flags]"
description: "Gen3 Sheepdog: Data submission and dictionary operations."
---

# Gen3 Sheepdog

Execute Gen3 Sheepdog operations: $ARGUMENTS

## Prerequisites

- Read `../gen3-shared/SKILL.md` first for shared auth and environment guidance.
- Read `../gen3-fence/SKILL.md` and authenticate first — all protected Sheepdog endpoints require a Bearer token issued by Fence.
- Confirm the target commons URL and active profile before making API calls.

## Command Shape

```bash
gen3 sheepdog <resource> <method> [flags]
```

## Service Summary

Sheepdog is the Gen3 structured data submission service. It manages the full data submission lifecycle: programs → projects → entities (graph nodes). All data is validated against a configurable dictionary before being written to the graph database. Sheepdog is the primary write path for metadata in a Gen3 commons.

---

## URL / Routing

In a standard Gen3 deployment, Sheepdog sits behind the reverse proxy at the `/api/` prefix:

```
{api_endpoint}/api/
```

Sheepdog itself registers its routes under `/v0/submission` and `/submission` (legacy alias). The full URL for any call is therefore:

```
{api_endpoint}/api/v0/submission/<path>
```

For example, if `api_endpoint = https://new.portal.ardac.org`:
```
GET  https://new.portal.ardac.org/api/v0/submission/
POST https://new.portal.ardac.org/api/v0/submission/ARDaC/ARDaC-SRP
```

The skill lists paths without the `/api` prefix throughout (e.g., `/v0/submission/`). Always prepend `/api` in practice.

---

## Authentication

All endpoints except `/_status`, `/_version`, and global dictionary/template reads require a **Fence Bearer token**.

```
Authorization: Bearer <access_token>
```

Obtain the access token via the Fence auth flow (see `../gen3-fence/SKILL.md`).

**Authorization model** (Arborist-enforced via Sheepdog middleware):

| Operation | Required Arborist resource | Required method |
|---|---|---|
| Read programs/projects | `/programs/<program>` | `_member_` |
| Read entities | `/programs/<program>/projects/<project>` | `_member_` |
| Submit / update entities | `/programs/<program>/projects/<project>` | `_write` |
| Delete entities | `/programs/<program>/projects/<project>` | `_delete` |
| Create program | `/services/sheepdog/submission/program` | `_admin` |
| Create project | `/services/sheepdog/submission/project` | `_admin` |

---

## Data Model Basics

- Data is organized as a directed graph: **Program → Project → entity nodes** (e.g., `case`, `sample`, `aliquot`, `read_group`, `submitted_unaligned_reads`).
- Every entity has a required `type` field (the node type from the dictionary) and a required `submitter_id` (must be unique per node type within the project).
- Entities reference parent nodes via links (e.g., `cases.submitter_id`).
- Deletion is **not cascading** — leaf nodes must be deleted before parents. The API returns a dependency error listing what must be deleted first.
- Project lifecycle states: `open` → `review` → `submit` → `release`. Only `open` projects accept submissions.

---

## Resource Reference

### system

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `system status` | GET | `/_status` | — |
| `system version` | GET | `/_version` | — |

`/_status` returns `200` when Sheepdog and its database are healthy.
`/_version` returns `{ "version": "...", "commit": "...", "dictionary": { "version": "..." } }`.

---

### programs

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `programs list` | GET | `/v0/submission/` | Bearer |
| `programs create` | POST/PUT | `/v0/submission/` | Bearer (admin) |
| `programs get` | GET | `/v0/submission/<program>` | Bearer |
| `programs delete` | DELETE | `/v0/submission/<program>` | Bearer (admin) |

#### `GET /v0/submission/` — list programs

Returns a JSON object with a `links` array of program paths:
```json
{ "links": ["/v0/submission/ARDaC", "/v0/submission/OtherProg"] }
```

#### `POST /v0/submission/` — create or update a program

Body (`application/json`):
```json
{
  "type": "program",
  "name": "MyProgram",
  "dbgap_accession_number": "phs000001"
}
```

`name` and `dbgap_accession_number` are required. Returns the created/updated program node.

#### `GET /v0/submission/<program>` — list projects in a program

Returns `{ "links": ["/v0/submission/<program>/PROJ1", ...] }`.

---

### projects

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `projects list` | GET | `/v0/submission/<program>` | Bearer |
| `projects create` | POST/PUT | `/v0/submission/<program>` | Bearer (admin) |
| `projects delete` | DELETE | `/v0/submission/<program>/<project>` | Bearer (admin) |
| `projects open` | POST/PUT | `/v0/submission/<program>/<project>/open` | Bearer (admin) |
| `projects review` | POST/PUT | `/v0/submission/<program>/<project>/review` | Bearer (admin) |
| `projects submit` | POST/PUT | `/v0/submission/<program>/<project>/submit` | Bearer (admin) |
| `projects release` | POST/PUT | `/v0/submission/<program>/<project>/release` | Bearer (admin) |

#### `POST /v0/submission/<program>` — create or update a project

Body (`application/json`):
```json
{
  "type": "project",
  "code": "PROJ",
  "name": "My Project Name",
  "dbgap_accession_number": "phs000002",
  "investigator_name": "Jane Smith"
}
```

`code` and `name` are required. The `code` becomes the URL segment (e.g., `/ARDaC/PROJ`).

#### Project lifecycle endpoints

Each of these transitions project state. All accept POST or PUT with no body.

| Endpoint | Effect |
|---|---|
| `/open` | Unlock the project — submissions are accepted |
| `/review` | Lock the project for review — no new submissions until `open` or `submit` |
| `/submit` | Mark as submitted |
| `/release` | Release the project data (makes it queryable via Peregrine/Guppy) |

Every lifecycle endpoint has a `/_dry_run` variant that validates the transition without committing it.

---

### entities (submission)

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `entities submit` | POST/PUT | `/v0/submission/<program>/<project>` | Bearer (write) |
| `entities submit dry-run` | POST/PUT | `/v0/submission/<program>/<project>/_dry_run` | Bearer (write) |
| `entities get` | GET | `/v0/submission/<program>/<project>/entities/<ids>` | Bearer (read) |
| `entities delete` | DELETE | `/v0/submission/<program>/<project>/entities/<ids>` | Bearer (delete) |
| `entities delete dry-run` | DELETE | `/v0/submission/<program>/<project>/entities/_dry_run/<ids>` | Bearer (delete) |
| `entities bulk-submit` | POST/PUT | `/v0/submission/<program>/<project>/bulk` | Bearer (write) |
| `entities bulk-submit dry-run` | POST/PUT | `/v0/submission/<program>/<project>/bulk/_dry_run` | Bearer (write) |
| `entities export` | GET/POST | `/v0/submission/<program>/<project>/export` | Bearer (read) |

#### `POST /v0/submission/<program>/<project>` — submit entities

Accepts JSON, TSV, or CSV. Select format with `Content-Type`:

| Format | Content-Type |
|---|---|
| JSON | `application/json` |
| TSV | `text/tab-separated-values` |
| CSV | `text/csv` |

**JSON body** — array of entity objects:
```json
[
  {
    "type": "case",
    "submitter_id": "case-001",
    "projects": { "code": "PROJ" }
  },
  {
    "type": "sample",
    "submitter_id": "sample-001",
    "cases": { "submitter_id": "case-001" },
    "tissue_type": "Tumor",
    "sample_type": "Primary Tumor"
  }
]
```

**TSV body** — a standard TSV with a header row including `type`, `submitter_id`, and any link columns (e.g., `cases.submitter_id`).

**Response (200)**:
```json
{
  "entities": [
    { "id": "<uuid>", "submitter_id": "case-001", "type": "case", "valid": true, "errors": [] }
  ],
  "entity_error_list": [],
  "success": true
}
```

**Always run `_dry_run` first** to validate without writing. Dry-run returns the same response shape but commits nothing.

#### `GET /v0/submission/<program>/<project>/entities/<ids>` — fetch entities

Path: `<ids>` is a comma-separated list of entity UUIDs.

Returns a `{ "entities": [...] }` object with full entity records.

#### `DELETE /v0/submission/<program>/<project>/entities/<ids>` — delete entities

Path: `<ids>` is a comma-separated list of entity UUIDs.

> **⚠️ Non-cascading**: If the entity has child nodes, the API returns a 400 with the list of dependent entities that must be deleted first. Always delete leaves before parents.

#### `POST /v0/submission/<program>/<project>/bulk` — bulk submit

Submits a single file as bulk data. Body (`application/json`):
```json
{
  "name": "case",
  "doc_format": "tsv",
  "doc": "type\tsubmitter_id\tprojects.code\ncase\tcase-001\tPROJ\n"
}
```

`doc_format` is `json`, `tsv`, or `csv`. `doc` is the full file content as a string.

#### `GET /v0/submission/<program>/<project>/export` — export entities

Query params:

| Param | Required | Notes |
|---|---|---|
| `node_label` | ✓ | Entity type to export (e.g., `case`, `sample`) |
| `ids` | optional | Comma-separated UUIDs to filter |
| `format` | optional | `json` (default), `tsv`, or `csv` |
| `with_index` | optional | Include file index metadata |

POST variant accepts the same fields in a JSON body.

---

### dictionary

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `dictionary list` | GET | `/v0/submission/_dictionary` | — |
| `dictionary get` | GET | `/v0/submission/_dictionary/<entry>` | — |
| `dictionary project-list` | GET | `/v0/submission/<program>/<project>/_dictionary` | Bearer |
| `dictionary project-get` | GET | `/v0/submission/<program>/<project>/_dictionary/<entry>` | Bearer |
| `dictionary template` | GET | `/v0/submission/template` | — |
| `dictionary template-entity` | GET | `/v0/submission/template/<entity>` | — |
| `dictionary project-template` | GET | `/v0/submission/<program>/<project>/template` | Bearer |
| `dictionary project-template-entity` | GET | `/v0/submission/<program>/<project>/template/<entity>` | Bearer |

#### `GET /v0/submission/_dictionary` — global schema links

Returns `{ "links": ["/_dictionary/case", "/_dictionary/sample", ...] }`.

#### `GET /v0/submission/_dictionary/<entry>` — full JSON schema for an entity type

Returns the JSON Schema definition for the node type (properties, required fields, links, enum values). Use this to understand what fields are valid before submitting.

#### `GET /v0/submission/template/<entity>` — download a submission template

Query params: `format` (`tsv` or `csv`, default `tsv`).

Returns a TSV/CSV file with the header row pre-filled for the entity type. Links are shown as `{link_type}.{link_unique_key}`. Useful for offline preparation of submissions.

#### `GET /v0/submission/<program>/<project>/template` — all templates for a project

Query params: `format`, `categories` (comma-separated node categories to include), `exclude` (categories to skip).

---

### files

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `files get` | GET | `/v0/submission/<program>/<project>/files/<uuid>` | Bearer |
| `files upload-multipart` | POST | `/v0/submission/<program>/<project>/files/<uuid>` | Bearer |
| `files upload-put` | PUT | `/v0/submission/<program>/<project>/files/<uuid>` | Bearer |
| `files delete` | DELETE | `/v0/submission/<program>/<project>/files/<uuid>` | Bearer |
| `files manifest` | GET | `/v0/submission/<program>/<project>/manifest` | Bearer |
| `files upload-manifest` | GET | `/v0/submission/<program>/<project>/upload_manifest` | Bearer |

#### `PUT /v0/submission/<program>/<project>/files/<uuid>` — single PUT upload

The request body is the raw binary file content. `uuid` is the file's GUID (from indexd).

Query params (optional): `partNumber` and `uploadId` for multipart continuation.

#### `POST /v0/submission/<program>/<project>/files/<uuid>` — multipart upload

Body is `multipart/form-data` with the file part.

#### `GET /v0/submission/<program>/<project>/manifest` — file manifest

Returns a JSON manifest of all files associated with the project, including GUIDs, sizes, md5, and URLs.

---

### transactions (dry-run workflow)

The dry-run → commit workflow lets you validate before committing:

1. **Submit dry-run**: `POST /v0/submission/<program>/<project>/_dry_run` → get `transaction_id`
2. **Inspect results**: check the response for errors
3. **Commit** (if successful): `POST /v0/submission/<program>/<project>/transactions/<transaction_id>/commit`
4. **Close** (to discard): `POST /v0/submission/<program>/<project>/transactions/<transaction_id>/close`

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `transactions commit` | POST/PUT | `/v0/submission/<program>/<project>/transactions/<id>/commit` | Bearer |
| `transactions close` | POST/PUT | `/v0/submission/<program>/<project>/transactions/<id>/close` | Bearer |

A transaction can only be committed if:
- It was a dry-run transaction
- It has not been committed already
- It was successful (no validation errors)

---

### xml (legacy BCR format)

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `xml biospecimen upload` | PUT | `/v0/submission/<program>/<project>/xml/biospecimen/bcr` | Bearer |
| `xml clinical upload` | PUT | `/v0/submission/<program>/<project>/xml/clinical/bcr` | Bearer |

Body: `multipart/form-data` with a `file` field containing the BCR XML. The XML is converted to JSON entities before being stored. Dry-run variants are available at `.../_dry_run`.

---

### admin

These endpoints require Sheepdog admin authorization.

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `admin entities mark-delete` | DELETE | `/v0/submission/admin/<program>/<project>/entities/<ids>/to_delete/<true\|false>` | Bearer (admin) |
| `admin files reassign` | PUT | `/v0/submission/admin/<program>/<project>/files/<uuid>/reassign` | Bearer (admin) |

`mark-delete` sets the `to_delete` system annotation on entities without removing them from the graph — useful for staging bulk deletions.

`reassign` manually updates the S3 URL stored for a data file node.

---

## Common Error Codes

| Code | Meaning |
|---|---|
| 200 | Success |
| 201 | Created |
| 400 | Bad request — validation error, missing required field, or dependency constraint |
| 401 | Unauthorized — missing or expired Bearer token; re-exchange the API key via Fence |
| 403 | Forbidden — valid token but insufficient permissions for this program/project |
| 404 | Resource not found — program, project, or entity UUID does not exist |
| 405 | Method not allowed |
| 500 | Internal server error |

**400 response shape** (entity validation errors):
```json
{
  "entities": [],
  "entity_error_list": [
    { "submitter_id": "case-001", "errors": ["missing required field: cases"] }
  ],
  "success": false
}
```

---

## Typical Agent Workflow

### Explore a commons

1. **Authenticate**: exchange `api_key` → `access_token` via Fence.
2. **List programs**: `GET {api_endpoint}/api/v0/submission/` → read `links`.
3. **List projects**: `GET {api_endpoint}/api/v0/submission/<program>` → read `links`.
4. **Inspect dictionary**: `GET /api/v0/submission/_dictionary` → browse node types.
5. **Fetch a schema**: `GET /api/v0/submission/_dictionary/<node_type>` → see required fields.

### Submit data

1. **Authenticate** via Fence.
2. **Check project state**: if not `open`, call `/open` first.
3. **Download template**: `GET /api/v0/submission/template/<node_type>?format=tsv`.
4. **Dry-run submit**: `POST /api/v0/submission/<program>/<project>/_dry_run` with your data. Check `entity_error_list`.
5. **Commit** if successful: `POST .../transactions/<id>/commit`.
6. **Verify**: `GET .../entities/<id>` for a spot-check.

### Export data

1. **Get the dictionary**: identify the target `node_label` (node type).
2. **Export**: `GET /api/v0/submission/<program>/<project>/export?node_label=case&format=tsv`.

---

## Usage Notes

1. Always **dry-run first** for any mutating operation — every write endpoint has a `/_dry_run` counterpart.
2. `submitter_id` must be unique per node type **within the project** — reusing one will update the existing node.
3. Entity graph is **not cascading on delete** — build a deletion order from leaves to root.
4. Check `project.state` before submitting — the API returns 400 if the project is not in `open` state.
5. For large datasets, use **bulk** endpoint (`/bulk`) rather than sending thousands of individual entities.
6. TSV/CSV submissions use the template column headers exactly — mismatched headers cause 400 errors.
7. Format selection is via `Content-Type` for POSTs; use `?format=` query param for exports and templates.
8. On `401`: repeat the Fence token exchange and retry the request once before surfacing an error.

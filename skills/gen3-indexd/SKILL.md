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
- All write operations require a Bearer access token obtained from Fence.

## Command Shape

```bash
gen3 indexd <resource> <method> [flags]
```

## Service Summary

Indexd is Gen3's data indexing and tracking service. It assigns globally unique identifiers (GUIDs / DIDs) to data objects and tracks their metadata: hashes, size, storage URLs, ACLs, authz resources, and full version history. It is the source of truth for file identity in a Gen3 commons.

---

## URL / Routing

In a standard Gen3 deployment, Indexd sits behind the reverse proxy at the root path. All calls use the `api_endpoint` directly:

```
{api_endpoint}/index/{GUID}
{api_endpoint}/bulk/documents
{api_endpoint}/_status
```

## Authentication

Read endpoints are generally public. Write operations (create, update, delete) require authentication.

The CLI uses a Bearer token from Fence:

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

### records

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `records get` | GET | `/index/{GUID}` | — |
| `records list` | GET | `/index` | — |
| `records create` | POST | `/index` | Bearer |
| `records update` | PUT | `/index/{GUID}` | Bearer |
| `records delete` | DELETE | `/index/{GUID}` | Bearer |
| `records versions` | GET | `/index/{GUID}/versions` | — |
| `records latest` | GET | `/index/{GUID}/latest` | — |

#### `GET /index/{GUID}` — get record

Returns the metadata for the given GUID. Resolves bundle IDs.

Query params (optional): `expand` (boolean — recursively expand bundle contents).

**Response fields**: `did`, `rev`, `baseid`, `form`, `size`, `file_name`, `version`, `uploader`, `urls`, `hashes`, `acl`, `authz`, `metadata`, `created_date`, `updated_date`.

#### `GET /index` — list records

Query params (all optional):

| Param | Notes |
|---|---|
| `limit` | Records per page (default 100) |
| `page` | Page number (0-based) |
| `start` | Start GUID for cursor-based pagination |
| `hash` | Filter by hash: `algorithm:value` (e.g. `md5:abc123`). Repeat for multiple. |
| `url` | Filter by URL. Repeat for multiple (AND logic). |
| `acl` | Comma-delimited ACEs (AND logic). |
| `authz` | Comma-delimited authz resources (AND logic). |
| `uploader` | Filter by uploader ID. |
| `metadata` | Filter by `key:value` pairs. |
| `form` | `object`, `bundle`, or `all` (default: `object`). |
| `size` | Filter by file size in bytes. |

#### `POST /index` — create record

Body (`application/json`):

```json
{
  "form": "object",
  "hashes": { "md5": "abc123...", "sha256": "def456..." },
  "size": 1234,
  "urls": ["s3://bucket/path/to/file"],
  "acl": ["*"],
  "authz": ["/programs/MyProgram/projects/MyProject"],
  "file_name": "myfile.vcf.gz",
  "metadata": {}
}
```

**Response (200)**: `{ "did": "dg.xxxx/...", "rev": "xxxxxxxx", "baseid": "..." }`

At least one hash is required.

#### `PUT /index/{GUID}` — update record

Query params: `rev` (required — current revision for optimistic locking).

Body: include only fields to change: `urls`, `acl`, `authz`, `file_name`, `version`, `metadata`, `urls_metadata`.

**Response**: `{ "did": "...", "rev": "...", "baseid": "..." }` with a new `rev`.

#### `DELETE /index/{GUID}` — delete record

Query params: `rev` (required).

Returns `200` on success. Data content is immutable — delete only removes the index entry, not the underlying object bytes.

#### `GET /index/{GUID}/versions` — list all versions

Returns records for all versions that share the same `baseid`, in the order created.

#### `GET /index/{GUID}/latest` — get latest version

Query params (optional): `has_version` (boolean — filter to records that have a `version` field set).

---

### blank

Used for the upload workflow: reserve a GUID before the file is uploaded, then fill in hashes and size after.

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `blank create` | POST | `/index/blank` | Bearer |
| `blank update` | PUT | `/index/blank/{GUID}` | Bearer |

#### `POST /index/blank` — create blank record

Body (`application/json`):

```json
{
  "uploader": "username",
  "authz": ["/programs/MyProgram/projects/MyProject"]
}
```

**Response (201)**: `{ "did": "dg.xxxx/...", "rev": "xxxxxxxx", "baseid": "..." }` — store `did` and `rev`.

#### `PUT /index/blank/{GUID}` — fill in blank record

After the upload is complete, supply hashes, size, and URLs.

Query params: `rev` (required).

Body:

```json
{
  "hashes": { "md5": "abc123..." },
  "size": 1234,
  "urls": ["s3://bucket/key"],
  "authz": ["/programs/MyProgram/projects/MyProject"]
}
```

---

### aliases

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `aliases list` | GET | `/index/{GUID}/aliases` | — |
| `aliases add` | POST | `/index/{GUID}/aliases` | Bearer |
| `aliases replace` | PUT | `/index/{GUID}/aliases` | Bearer |
| `aliases delete-all` | DELETE | `/index/{GUID}/aliases` | Bearer |
| `aliases delete` | DELETE | `/index/{GUID}/aliases/{ALIAS}` | Bearer |

Body for `add` and `replace`: `{ "aliases": ["my-alias-1", "my-alias-2"] }`

- `add` appends the new aliases (all must be globally unique).
- `replace` replaces all existing aliases for the GUID (new aliases must be globally unique except for those already belonging to this GUID).

---

### bulk

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `bulk get` | POST | `/bulk/documents` | — |

#### `POST /bulk/documents` — fetch multiple records

Body (`application/json`):

```json
{ "ids": ["dg.xxxx/aaa", "dg.xxxx/bbb"] }
```

Returns an array of index records in the same order as the input IDs.

---

### bundles

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `bundles list` | GET | `/bundle` | — |
| `bundles get` | GET | `/bundle/{GUID}` | — |
| `bundles create` | POST | `/bundle` | Bearer |
| `bundles delete` | DELETE | `/bundle/{GUID}` | Bearer |

#### `POST /bundle` — create bundle

Body (`application/json`):

```json
{
  "name": "my-bundle",
  "bundles": ["dg.xxxx/aaa", "dg.xxxx/bbb"],
  "checksum": [{ "type": "md5", "checksum": "abc..." }],
  "size": 1234
}
```

Only `bundles` is required; `name`, `checksum`, and `size` are optional and can be generated by Indexd if omitted.

#### `GET /bundle/{GUID}` — get bundle

Query params (optional): `expand` (boolean — recursively expand nested bundles).

---

### system

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `system status` | GET | `/_status` | — |
| `system version` | GET | `/_version` | — |
| `system stats` | GET | `/_stats` | — |

Use `system status` for health checks; `200` means Indexd is healthy and connected to its backing datastore.

`system stats` returns total record count, current GUID prefix, and hash distribution — useful for auditing.

---

## Common Error Codes

| Code | Meaning |
|---|---|
| 200 | Success |
| 201 | Created |
| 400 | Bad request / invalid parameters |
| 401 | Unauthorized — token missing or expired |
| 403 | Forbidden — insufficient permissions |
| 404 | Record / GUID not found |
| 409 | Conflict — revision mismatch or duplicate alias |
| 500 | Internal server error |

---

## Typical Agent Workflow

1. **Load profile** from `~/.gen3/config`; read `api_endpoint`, `api_key`.
2. **Exchange** `api_key` → `access_token` via `POST {api_endpoint}/user/credentials/api/access_token`.
3. **Call** target endpoint with `Authorization: Bearer <access_token>`.
4. **On 401**: repeat step 2 and retry once.
5. **Surface** `did`, `rev`, `baseid`, and key metadata to the user; show full payloads on demand.

### Typical Upload Workflow

```
1. gen3 indexd blank create --authz /programs/P/projects/Q
   → receives did=dg.xxxx/... rev=abc12345

2. <upload file bytes to storage using presigned URL from Fence>

3. gen3 indexd blank update --guid dg.xxxx/... --rev abc12345 \
     --hash md5:... --size 1234 --url s3://bucket/key
   → record is now complete and discoverable
```

---

## Usage Notes

1. GUIDs in Gen3 follow the pattern `dg.XXXX/UUID` — always URL-encode the slash when used in path segments.
2. The `rev` field is required for updates and deletes — always `GET` the record first to obtain the current `rev`.
3. Use `blank create` + `blank update` for the standard upload workflow (reserve GUID → upload file → register hashes).
4. Prefer explicit flags for identifiers (GUIDs, hashes, ACLs, authz resources).
5. Treat deletes and alias replacements as confirmation-worthy in the TUI.
6. `/_stats` is useful for auditing: returns total record count, current GUID prefix, and hash distribution.
7. The legacy `PUT /alias/{ALIASSTRING}` endpoint is deprecated — use `/index/{GUID}/aliases` instead.

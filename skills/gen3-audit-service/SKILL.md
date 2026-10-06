---
name: gen3-audit-service
allowed-tools: Bash, Read, Write, Edit
description: "Gen3 Audit Service: query login and presigned-URL audit logs and inspect service metadata."
---

# Gen3 Audit Service

Execute Gen3 Audit Service operations: $ARGUMENTS

## Prerequisites

- Read `../gen3-shared/SKILL.md` first for shared authentication and environment guidance.
- Confirm the target commons URL and active profile before making API calls.
- Use a Fence access token for every log query. The token must have Arborist `read` access to the requested audit category.
- Treat audit records as sensitive operational data. Do not expose usernames, IP addresses, request URLs, resource paths, or raw tokens unless the user needs them.

## Command Shape

```bash
gen3 audit-service <resource> <method> [flags]
```

The CLI-facing resources are:

- `logs query` for querying a log category.
- `system status`, `system version`, and `system schema` for service inspection.

Audit Service does not expose a general event-detail route, export-job API, retention API, or administrative API. Export query results client-side when requested.

## Service Summary

Audit Service stores and queries audit records emitted by Gen3 services. The current service defines two categories:

- `presigned_url`: file upload and download requests made through Fence presigned URLs.
- `login`: login attempts and their identity-provider details.

In a standard Gen3 deployment, Audit Service is routed under `/audit`. Build API paths from the selected profile's `api_endpoint`:

```text
{api_endpoint}/audit/log/{category}
{api_endpoint}/audit/_status
{api_endpoint}/audit/_version
{api_endpoint}/audit/_schema
```

Avoid duplicating `/audit` if the configured endpoint already includes that prefix.

## Authentication and Authorization

Query requests require:

```text
Authorization: Bearer <access_token>
```

Audit Service asks Arborist for service `audit`, method `read`, on the category resource:

| Category | Required resource |
|---|---|
| `login` | `/services/audit/login` |
| `presigned_url` | `/services/audit/presigned_url` |

Access to `/services/audit` may grant both categories, depending on the commons policy hierarchy. A valid token without the category permission returns `403`. A missing or invalid token returns `401`.

The system routes are public in the service contract. Do not infer that their availability means log queries are public.

## Resource Reference

### logs

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `logs query` | GET | `/audit/log/{category}` | Bearer + category `read` |

#### Categories and fields

Every category has these fields:

| Field | Type | Notes |
|---|---|---|
| `request_url` | string | URL handled by the audited request. |
| `status_code` | integer | HTTP result of the audited request. |
| `timestamp` | timestamp | Filter values use Unix epoch seconds. |
| `username` | string | May be unavailable when the deployment disables username querying. |
| `sub` | integer, nullable | Fence subject identifier; may be null for public data. |
| `additional_data` | object, nullable | Service-specific JSON data. |

`presigned_url` adds:

| Field | Type | Notes |
|---|---|---|
| `guid` | string | File GUID. |
| `resource_paths` | list of strings, nullable | Authz resource paths. Exact array values overlap-match; prefix matching is not implemented. |
| `action` | string | `download` or `upload`. |
| `protocol` | string, nullable | Storage protocol; may be null for missing files or uploads. |

`login` adds:

| Field | Type | Notes |
|---|---|---|
| `idp` | string | Identity provider. |
| `fence_idp` | string, nullable | Fence identity provider. |
| `shib_idp` | string, nullable | Shibboleth identity provider. |
| `client_id` | string, nullable | OAuth client identifier. |
| `ip` | string, nullable | Client IP address; legacy records may omit it. |

#### Query flags

Map flags to query parameters. Repeat a field flag to provide multiple values.

| Flag | Query parameter | Behavior |
|---|---|---|
| `--category <name>` | path `{category}` | Required; `login` or `presigned_url`. |
| `--start <epoch>` | `start` | Inclusive lower timestamp bound. |
| `--stop <epoch>` | `stop` | Exclusive upper timestamp bound. |
| `--filter <field=value>` | `<field>=<value>` | Filter on a field defined for the category. Repeatable. |
| `--group-by <field>` | `groupby=<field>` | Return counts grouped by a category field. Repeatable. |
| `--count` | `count` | Return the number of matching rows instead of rows. |
| `--all-pages` | client-side | Follow `nextTimeStamp` until it is null. Requires both `--start` and `--stop`; do not combine with `--count` or `--group-by`. JSONL and CSV pages are streamed. |
| `--output <format>` | client-side | Render `json`, `jsonl`, or `csv`; do not send this to the service. |

The raw API accepts field names directly as query keys. Same-key values use OR semantics; different keys use AND semantics:

```text
?guid=guid1&guid=guid2&status_code=200
```

This means `(guid == guid1 OR guid == guid2) AND status_code == 200`.

Only fields defined for the selected category are accepted. Invalid category names, field names, field values, time ranges, or group-by fields return `400`.

Deployments may set a maximum query time window. When that is enabled, Audit Service fills a missing boundary from the configured window and rejects ranges that exceed it. Prefer explicit `--start` and `--stop` values for reproducible queries.

#### Response shape

Normal queries return:

```json
{
  "nextTimeStamp": 1735689600,
  "data": [
    {
      "guid": "dg.XXXX/example",
      "action": "download",
      "status_code": 200,
      "timestamp": "2025-01-01T00:00:00",
      "username": "user@example.org"
    }
  ]
}
```

`nextTimeStamp` is null on the last page. Results are ordered by increasing timestamp. To fetch the next page, repeat the same request with `start=<nextTimeStamp>`. The service keeps all records that share the page-boundary timestamp together, so a page can exceed the configured page size.

With `count`, `data` is an integer and `nextTimeStamp` is null. With `groupby`, `data` is a list of grouped fields plus `count`, and `nextTimeStamp` is null. When both are supplied, `count` reports the number of grouped rows, not the total underlying events; normally choose one mode.

If the deployment disables username queries, filtering or grouping by `username` returns `400`, and returned records omit `username`.

#### Query examples

Count successful downloads of one GUID in a time window:

```bash
gen3 audit-service logs query \
  --category presigned_url \
  --filter action=download \
  --filter guid=dg.XXXX/example \
  --filter status_code=200 \
  --start 1735689600 \
  --stop 1738368000 \
  --count
```

Count successful downloads by protocol:

```bash
gen3 audit-service logs query \
  --category presigned_url \
  --filter action=download \
  --filter status_code=200 \
  --group-by protocol \
  --start 1735689600 \
  --stop 1738368000
```

Inspect logins for one identity provider:

```bash
gen3 audit-service logs query \
  --category login \
  --filter idp=example-idp \
  --start 1735689600 \
  --stop 1738368000
```

### system

| CLI verb | HTTP | Path | Auth required | Purpose |
|---|---|---|---|---|
| `system status` | GET | `/audit/_status` | No | Checks the service and database connection; healthy response is `{"status":"OK"}`. |
| `system version` | GET | `/audit/_version` | No | Returns the deployed service version. |
| `system schema` | GET | `/audit/_schema` | No | Returns category schema versions and field/type maps. |

Use `system schema` before assuming category fields. A `404` from `/_schema` indicates a legacy Audit Service; fall back to the documented core fields and report that live schema discovery is unavailable.

### Internal log creation

The service contract contains `POST /audit/log/login` and `POST /audit/log/presigned_url`, but these routes are intended for trusted internal services and are normally not exposed through the commons gateway. They require a Bearer header but do not perform category-level Arborist authorization.

Do not present creation as a normal user CLI operation. Only call it when the user explicitly targets a trusted internal endpoint and understands the deployment boundary. Omit `timestamp` for live events so Audit Service assigns it; supply an epoch timestamp only when backfilling historical records.

Required creation fields:

- Both categories: `request_url`, `status_code`, `username`.
- `presigned_url`: `guid`, `action` (`download` or `upload`).
- `login`: `idp`.

The remaining category fields are optional or nullable as described above. Successful creation returns `201`; request validation typically returns `400` or `422`.

## Common Error Codes

| Code | Meaning |
|---|---|
| `200` | Query or system request succeeded. |
| `201` | Internal log creation accepted. |
| `400` | Invalid category, field, value, group-by, action, or time range. |
| `401` | Access token missing, invalid, or unsuitable. |
| `403` | Token lacks `read` access to the category resource. |
| `404` | Route unavailable; `/_schema` may be absent on legacy deployments. |
| `422` | Request shape failed API validation. |
| `500` | Service or database failure. |

## Agent Workflow

1. Load the selected profile and show the target commons and profile before querying sensitive logs.
2. Exchange the stored API key for a Fence access token as described by the shared skill.
3. Call `GET /audit/_schema` when available to confirm live categories and fields.
4. Translate the requested time range to Unix epoch seconds. Confirm the timezone when the user's dates are ambiguous.
5. Build `GET /audit/log/{category}` with explicit filters and the Bearer token.
6. For full result sets, require explicit `start` and `stop` bounds, then follow `nextTimeStamp` with the exact same filters until it is null. Detect a repeated cursor and stop with an error rather than looping forever.
7. Summarize counts or key fields by default. Write raw sensitive rows to a file only when requested, and report the file format and path.
8. On `401`, refresh the Fence token and retry once. On `403`, report the exact required Arborist resource without attempting to broaden permissions.

## Safety and Accuracy Notes

1. Use `start` as inclusive and `stop` as exclusive; do not silently treat both ends as inclusive.
2. URL-encode GUIDs, resource paths, usernames, identity-provider names, and all repeated query values.
3. Do not claim that Audit Service provides server-side export jobs. CSV and JSONL exports are client-side renderings of query results; neutralize spreadsheet-formula prefixes in CSV string cells.
4. Do not infer per-file authorization from `resource_paths`; current query authorization is category-wide.
5. Avoid unbounded queries. Ask for or choose a narrow, explicit time window when the user's request permits it.
6. Never display or persist the access token in output, logs, or export files.

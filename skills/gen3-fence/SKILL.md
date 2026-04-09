---
allowed-tools: Bash, Read, Write, Edit
argument-hint: "[resource] [method] [flags]"
description: "Gen3 Fence: AuthN/AuthZ, identity, and token operations."
---

# Gen3 Fence

Execute Gen3 Fence operations: $ARGUMENTS

## Prerequisites

- Read `../gen3-shared/SKILL.md` first for shared auth and environment guidance.
- Confirm the target commons URL and active profile before making API calls.
- All protected endpoints require a Bearer access token obtained via the authentication flow below.

## Command Shape

```bash
gen3 fence <resource> <method> [flags]
```

## Configuration

The CLI stores credentials in `~/.gen3/config` (TOML). Each named profile contains:

```toml
[profiles.default]
api_endpoint = "https://commons.example.org"
api_key      = "<long JWT string from credentials.json>"
key_id       = "<jti / UUID of the key>"
```

The credentials file downloaded from the Fence UI (`credentials.json`) has this shape:

```json
{ "api_key": "eyJ...", "key_id": "a1b2c3d4-..." }
```

Use `gen3 auth setup` to import a credentials file and register a profile.

---

## URL / Routing

In a standard Gen3 deployment, Fence sits behind the reverse proxy at the `/user/` prefix:

```
{api_endpoint}/user/
```

So if `api_endpoint = https://new.portal.ardac.org`, all Fence calls use:
```
POST https://new.portal.ardac.org/user/credentials/api/access_token
GET  https://new.portal.ardac.org/user/user
GET  https://new.portal.ardac.org/user/logout
```

The skill lists paths without this prefix throughout (e.g., `/credentials/api/access_token`). Always prepend `/user` in practice.

## Authentication Flow

All protected Fence endpoints require a short-lived **access token** (Bearer JWT). The standard machine-to-machine flow uses the stored `api_key` to obtain one.

### 1. Exchange API key for an access token

```
POST {api_endpoint}/user/credentials/api/access_token
Content-Type: application/json

{ "api_key": "<profile.api_key>" }
```

**Response (200)**:
```json
{ "access_token": "eyJ..." }
```

Store the returned `access_token` and attach it to subsequent requests as:

```
Authorization: Bearer <access_token>
```

Access tokens are short-lived (minutes to hours). Repeat this call to refresh when a request returns `401`.

### 2. Revoke a refresh token (optional cleanup)

```
POST {api_endpoint}/oauth2/revoke
Content-Type: application/x-www-form-urlencoded

token=<refresh_token>
```

---

## Resource Reference

### auth / oauth2

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `auth token exchange` | POST | `/oauth2/token` | Client credentials |
| `auth token revoke` | POST | `/oauth2/revoke` | Client credentials |
| `auth authorize` | GET/POST | `/oauth2/authorize` | — |

#### `POST /oauth2/token` — exchange or refresh

Body (`application/x-www-form-urlencoded`):

| Field | Required | Notes |
|---|---|---|
| `grant_type` | ✓ | `authorization_code` or `client_credentials` |
| `code` | if `authorization_code` | Auth code from redirect |
| `redirect_uri` | if `authorization_code` | Must match original request |
| `scope` | optional (client_credentials) | Space-separated; must include `openid` |
| `client_id` | optional | OAuth2 client ID |

#### `GET /oauth2/authorize` — start OAuth2 flow

Query params: `client_id` (required), `response_type` (required, `code`), `redirect_uri` (required), `scope` (optional, include `openid user`), `idp` (optional: `google`, `shibboleth`, `fence`).

---

### user

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `user get` | GET | `/user` | Bearer (scope: `user`) |

#### `GET /user` — current user info

Returns the authenticated user's profile, linked identities, and project access.

**Response fields**: `username`, `email`, `sub` (subject), `resources_granted`, `project_access`, `google` (linked account info).

---

### credentials / api-keys

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `credentials list` | GET | `/credentials/api` | Bearer |
| `credentials create` | POST | `/credentials/api` | Bearer |
| `credentials delete` | DELETE | `/credentials/api/{key_id}` | Bearer |
| `credentials token` | POST | `/credentials/api/access_token` | API key in body |

#### `POST /credentials/api` — create API key

Body (`application/json`):
```json
{ "scope": ["openid", "user", "data"] }
```

**Response**: `{ "key_id": "...", "api_key": "eyJ..." }` — store both; the `api_key` cannot be retrieved again.

#### `DELETE /credentials/api/{key_id}` — delete a specific key

Path param: `key_id` — the `jti` / UUID of the key to delete.

---

### credentials / google

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `credentials google list` | GET | `/credentials/google` | Bearer |
| `credentials google create` | POST | `/credentials/google` | Bearer |
| `credentials google delete-all` | DELETE | `/credentials/google?all=true` | Bearer |
| `credentials google delete` | DELETE | `/credentials/google/{access_key}` | Bearer |

#### `POST /credentials/google` — get temporary Google key

Query params (optional): `expires_in` (seconds, default 10 days max), `userProject` (billing project).

**Response**: A Google service account JSON credentials object ready to write to disk.

---

### data

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `data download` | GET | `/data/download/{file_id}` | Bearer (scope: `user`) |
| `data upload init` | POST | `/data/upload` | Bearer (scope: `user data`) |
| `data upload url` | GET | `/data/upload/{file_id}` | Bearer (scope: `user`) |
| `data delete` | DELETE | `/data/{file_id}` | Bearer (scope: `data`) |
| `data multipart init` | POST | `/multipart/init` | Bearer |
| `data multipart upload` | POST | `/multipart/upload` | Bearer |
| `data multipart complete` | POST | `/multipart/complete` | Bearer |
| `data buckets` | GET | `/data/buckets` | — |

#### `GET /data/download/{file_id}` — generate a download signed URL

Path: `file_id` — the GUID of the file in indexd.

Query params (all optional):
| Param | Notes |
|---|---|
| `protocol` | `s3`, `gs`, or `http` |
| `expires_in` | Seconds until URL expires (default/max: 3600) |
| `redirect` | If `true`, responds with 302 redirect instead of JSON |
| `no_force_sign` | If `true`, returns unsigned (public) URL |
| `userProject` | GCP billing project for requester-pays buckets |

**Response**: `{ "url": "https://..." }`

#### `POST /data/upload` — initiate a new upload (< 5 GB)

Body (`application/json`):
```json
{
  "file_name": "my-file.vcf.gz",
  "authz": ["/programs/MyProgram/projects/MyProject"],
  "guid": "<optional existing GUID>"
}
```

**Response (201)**: `{ "guid": "dg.xxxx/...", "url": "https://presigned-s3-url..." }`

Use the presigned `url` to PUT the file bytes directly to the storage bucket.

#### `POST /multipart/init` — initiate multipart upload (> 5 GB)

Body same as `/data/upload`. **Response (201)**: `{ "guid": "...", "uploadId": "..." }`.

Then call `/multipart/upload` with `{ "key": "<guid>", "uploadId": "...", "partNumber": 1 }` for each part (min 5 MB per part), and `/multipart/complete` with the list of `{ "partNumber": N, "ETag": "..." }` pairs.

---

### admin

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `admin user get` | GET | `/admin/user/{username}` | Bearer (scope: `user`), admin role |
| `admin user create` | POST | `/admin/user` | Bearer, admin role |
| `admin user delete` | DELETE | `/admin/user/{username}/soft` | Bearer (scope: `admin`) |

#### `GET /admin/user/{username}` — fetch any user's info

Returns the same `UserInfo` shape as `GET /user` but for the named user.

#### `POST /admin/user` — create a new user

Body: `{ "username": "...", "email": "...", "role": "user" }`.

#### `DELETE /admin/user/{username}/soft` — deactivate a user

Sets the user to inactive (soft delete). Returns `204` on success.

---

### link

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `link google start` | GET | `/link/google?redirect=<url>` | Bearer |
| `link google extend` | PATCH | `/link/google` | Bearer |
| `link google remove` | DELETE | `/link/google` | Bearer |

#### `GET /link/google` — start Google identity linking

Query: `redirect` (required) — URL to redirect after linking. `expires_in` (optional, seconds).

Initiates an OAuth2 flow with Google. On success, redirects to the callback at `/link/google/callback` which adds the Google account to the user's proxy group for GCS data access.

#### `PATCH /link/google` — extend access

Extends the linked Google account's data access expiry. Query: `expires_in` (optional).

**Response (200)**: `{ "linked_google_account_exp": <unix timestamp> }`

---

### google (service accounts)

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `google sa list` | GET | `/google/service_accounts/?google_project_ids=<ids>` | Bearer |
| `google sa register` | POST | `/google/service_accounts/` | Bearer |
| `google sa register dry-run` | POST | `/google/service_accounts/_dry_run` | Bearer |
| `google sa update` | PATCH | `/google/service_accounts/{id}` | Bearer |
| `google sa update dry-run` | PATCH | `/google/service_accounts/_dry_run/{id}` | Bearer |
| `google sa delete` | DELETE | `/google/service_accounts/{id}` | Bearer |
| `google sa monitor` | GET | `/google/service_accounts/monitor` | Bearer |
| `google billing` | GET | `/google/billing_projects/` | Bearer |

#### `POST /google/service_accounts/` — register a service account

Body: `{ "service_account_email": "sa@project.iam.gserviceaccount.com", "google_project_id": "my-gcp-project", "project_access": ["program-project"] }`.

Use the `_dry_run` variant first to detect issues without committing the registration.

---

### keys (JWT public keys)

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `keys list` | GET | `/jwt/keys` | — |

Returns the JWKS (JSON Web Key Set) used to validate tokens issued by Fence.

---

### system

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `system status` | GET | `/_status` | — |
| `system version` | GET | `/_version` | — |

Use `/_status` for health checks; a `200` response means Fence is healthy.

---

### logout

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `logout` | GET | `/logout` | Bearer |

Query params (optional): `next` (redirect URL after logout), `force_era_global_logout` (`true` to propagate logout through eRA Commons iTrust).

---

## GA4GH DRS (data access)

| CLI verb | HTTP | Path | Auth required |
|---|---|---|---|
| `drs access get` | GET | `/ga4gh/drs/v1/objects/{object_id}/access/{access_id}` | Bearer |
| `drs access post` | POST | `/ga4gh/drs/v1/objects/{object_id}/access/{access_id}` | Passport JWT in body |

Used to retrieve signed download URLs for GA4GH DRS objects. The POST variant accepts a GA4GH Passport JWT in the body for cross-commons access.

---

## Common Error Codes

| Code | Meaning |
|---|---|
| 200 | Success |
| 201 | Created |
| 204 | Success (no content) |
| 302 | Redirect (login/logout flows) |
| 400 | Bad request / invalid parameters |
| 401 | Unauthorized — token missing, expired, or invalid; re-exchange API key |
| 403 | Forbidden — valid token but insufficient permissions |
| 404 | Resource not found |
| 500 | Internal server error |

---

## Typical Agent Workflow

1. **Load profile** from `~/.gen3/config`; read `api_endpoint`, `api_key`.
2. **Exchange** `api_key` → `access_token` via `POST {api_endpoint}/credentials/api/access_token`.
3. **Call** target endpoint with `Authorization: Bearer <access_token>`.
4. **On 401**: repeat step 2 and retry once.
5. **Surface** `status`, key identifiers, and response summaries to the user; show full payloads on demand.

## Usage Notes

1. Keep the CLI shape resource-first so Ratatui flows can drill from service → resource → method.
2. Prefer explicit flags for identifiers (GUIDs, `key_id`, usernames, project IDs, scopes).
3. Treat admin and destructive operations (`delete`, `soft delete`, service account registration) as confirmation-worthy in the TUI.
4. For file uploads, always check `/_status` and `GET /data/buckets` first to confirm the service is reachable and the target bucket exists.
5. The `credentials/cdis` endpoints are deprecated — always use `credentials/api` instead.

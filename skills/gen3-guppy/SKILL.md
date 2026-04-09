---
allowed-tools: Bash, Read, Write, Edit
argument-hint: "[resource] [method] [flags]"
description: "Gen3 Guppy: Elasticsearch-backed aggregation, faceted counts, histograms, and record download."
---

# Gen3 Guppy

Execute Gen3 Guppy operations: $ARGUMENTS

## Prerequisites

- Read `../gen3-shared/SKILL.md` first for shared auth and environment guidance.
- Confirm the target commons URL and active profile before making API calls.
- All protected endpoints require a Bearer access token obtained via the authentication flow below.

## Command Shape

```bash
gen3 guppy <resource> <method> [flags]
```

## Configuration

The CLI stores credentials in `~/.gen3/config` (TOML). Each named profile contains:

```toml
[profiles.default]
api_endpoint = "https://commons.example.org"
api_key      = "<long JWT string from credentials.json>"
key_id       = "<jti / UUID of the key>"
```

Use `gen3 auth setup` to import a credentials file and register a profile.

---

## URL / Routing

Guppy sits behind the reverse proxy at the `/guppy/` prefix:

```
{api_endpoint}/guppy/
```

The Express app registers routes at `/graphql`, `/download`, `/_status`, `/_version`, and `/_refresh`.
The revproxy strips the prefix, so public paths become:

```
GET  {api_endpoint}/guppy/_status
GET  {api_endpoint}/guppy/_version
POST {api_endpoint}/guppy/graphql
POST {api_endpoint}/guppy/download
POST {api_endpoint}/guppy/_refresh
```

---

## Authentication

All meaningful operations require a Bearer token:

```
Authorization: Bearer <access_token>
```

Obtain the token via Fence:

```http
POST {api_endpoint}/user/credentials/api/access_token
Content-Type: application/json

{ "api_key": "<api_key_from_credentials_json>" }
```

`/_status` and `/_version` are unauthenticated. All data endpoints enforce Arborist policy.
For "regular" tier access commons, unauthenticated queries return 0 counts.

---

## Resources

### `system` — Health and version

#### `gen3 guppy system status`

Simple health check. Returns healthy/unhealthy.

```
GET /guppy/_status
```

Response (200):
```json
{
  "statusCode": 200,
  "warnings": null,
  "indices": { ... }
}
```

#### `gen3 guppy system version`

Returns the Guppy version and git commit.

```
GET /guppy/_version
```

Response:
```json
{ "version": "2026.03", "commit": "538d3e0b..." }
```

#### `gen3 guppy system indices`

Lists all Elasticsearch indices configured in Guppy, their aliases, and array field
configurations. Uses `GET /guppy/_status` and prints the full JSON payload.

---

### `graphql` — Raw GraphQL access

Guppy exposes an Apollo GraphQL server. The schema is dynamically generated from the
Elasticsearch index mappings configured at startup. CSRF protection is active — requests
**must** use `POST` with `Content-Type: application/json`.

#### `gen3 guppy graphql query --query '<gql>'`

Run any GraphQL query against Guppy. Requires auth.

```
POST /guppy/graphql
Authorization: Bearer <token>
Content-Type: application/json

{ "query": "...", "variables": { ... } }
```

Flags:
- `--query` (required) — GraphQL query string
- `--vars` — JSON-encoded variables object

Example — total counts for all types:
```bash
gen3 guppy graphql query --query '{ _aggregation { case { _totalCount } follow_up { _totalCount } } }'
```

Example — with filter variable:
```bash
gen3 guppy graphql query \
  --query 'query($f: JSON) { _aggregation { case(filter: $f) { _totalCount } } }' \
  --vars '{"f": {"AND": [{"=": {"field": "gender", "value": "Male"}}]}}'
```

#### `gen3 guppy graphql introspect`

Runs a GraphQL introspection query and prints a readable summary of all OBJECT types
and their field counts.

---

### `aggregation` — Counts and histograms

#### `gen3 guppy aggregation counts [--type <type>] [--filter '<json>'] [--accessibility all|accessible]`

Get total record counts per index type. Without `--type`, queries all available types.

Flags:
- `--type` — index type name (e.g. `case`, `follow_up`). Omit for all types.
- `--filter` — JSON filter expression (see Filter Syntax below)
- `--accessibility` — `all` (default), `accessible`, or `unaccessible`

Example:
```bash
gen3 guppy aggregation counts
gen3 guppy aggregation counts --type case --accessibility accessible
gen3 guppy aggregation counts --type case --filter '{"AND":[{"=":{"field":"gender","value":"Male"}}]}'
```

#### `gen3 guppy aggregation histogram --type <type> --field <field> [--filter '<json>'] [--accessibility all|accessible]`

Get a value histogram (key/count pairs) for a field. Works for both string and
numeric fields (uses `asTextHistogram` which is type-agnostic). Outputs a formatted bar chart.

Flags:
- `--type` (required) — index type (e.g. `case`)
- `--field` (required) — field name (e.g. `gender`, `age_at_index`)
- `--filter` — JSON filter expression
- `--accessibility` — `all` (default), `accessible`, or `unaccessible`

Example:
```bash
gen3 guppy aggregation histogram --type case --field gender
gen3 guppy aggregation histogram --type follow_up --field aki_status --accessibility accessible
```

---

### `mapping` — Field discovery

#### `gen3 guppy mapping list [--type <type>] [--search <term>]`

List the available fields in one or all index types. Uses the `_mapping` GraphQL operation.

Flags:
- `--type` — index type (e.g. `case`). Omit to list all types.
- `--search` — filter fields by substring match

Example:
```bash
gen3 guppy mapping list
gen3 guppy mapping list --type case
gen3 guppy mapping list --type case --search age
```

---

### `download` — Raw record export

#### `gen3 guppy download records --type <type> [flags]`

Download raw records from an Elasticsearch index. Auth is enforced based on the
commons' `tier_access_level` setting:
- `private` — always filters to accessible resources
- `regular` — with `--accessibility accessible`, applies accessible filter; without it,
  returns 401 if you request any out-of-scope resources
- `libre` — no auth check; all records returned

```
POST /guppy/download
Authorization: Bearer <token>
Content-Type: application/json

{
  "type": "case",
  "fields": ["subject_id", "gender", "age_at_index"],
  "filter": { ... },
  "sort": [{"field": "age_at_index", "order": "asc"}],
  "accessibility": "accessible"
}
```

Flags:
- `--type` (required) — index type name
- `--fields` — comma-separated list of fields to include (omit for all)
- `--filter` — JSON filter expression
- `--sort` — JSON sort array
- `--accessibility` — `accessible` (default), `all`, or `unaccessible`

Example:
```bash
gen3 guppy download records --type case --fields subject_id,gender,age_at_index --accessibility accessible
gen3 guppy download records --type follow_up \
  --filter '{"AND":[{"=":{"field":"aki_status","value":"Yes"}}]}' \
  --accessibility accessible
```

---

## GraphQL Schema Overview

The schema is generated dynamically from ES index mappings. The key entry points:

### Query type

```graphql
type Query {
  case(offset: Int, first: Int, filter: JSON, sort: JSON, accessibility: Accessibility): [Case]
  follow_up(offset: Int, first: Int, filter: JSON, sort: JSON, accessibility: Accessibility): [Follow_up]
  # ... one per configured index type
  _aggregation: Aggregation
  _mapping: Mapping
}
```

### Aggregation type

```graphql
type Aggregation {
  case(filter: JSON, filterSelf: Boolean, nestedAggFields: JSON, accessibility: Accessibility): CaseAggregation
  # ... one per type
}

type CaseAggregation {
  _totalCount: Int
  gender: HistogramForString    # or HistogramForNumber for numeric fields
  age_at_index: HistogramForNumber
  # ... one per mapped field
}
```

### Histogram types

```graphql
type HistogramForString {
  _totalCount: Int
  _cardinalityCount(precision_threshold: Int): Int
  histogram: [BucketsForNestedStringAgg]   # { key: String, count: Int }
  asTextHistogram: [BucketsForNestedStringAgg]
}

type HistogramForNumber {
  _totalCount: Int
  _cardinalityCount(precision_threshold: Int): Int
  histogram(rangeStart: Int, rangeEnd: Int, rangeStep: Int, binCount: Int): [BucketsForNestedNumberAgg]
  asTextHistogram: [BucketsForNestedStringAgg]
}
```

### Mapping type

```graphql
type Mapping {
  case(searchInput: String): [String]
  # ... one per type
}
```

### Enums

```graphql
enum Accessibility { all  accessible  unaccessible }
enum Format        { json tsv csv }
```

---

## Filter Syntax

Guppy uses a Gen3-specific filter format (not ES query DSL directly). Filters are
nested boolean expressions:

```json
{
  "AND": [
    { "=":  { "field": "gender",      "value": "Male" } },
    { ">=": { "field": "age_at_index","value": 18 } },
    { "IN": { "field": "race",        "value": ["White", "Asian"] } }
  ]
}
```

Supported operators: `=`, `!=`, `<`, `<=`, `>`, `>=`, `IN`, `EXCLUDE`
Boolean combiners: `AND`, `OR`, `NOT`

---

## Known Quirks

- **CSRF**: `POST /guppy/graphql` requires `Content-Type: application/json`. A bare GET
  returns 400 "potential CSRF".
- **Tier access**: On "regular" tier commons, unauthenticated queries return 0 counts.
  Pass `accessibility: accessible` with a token to get real counts you have access to.
- **Index types vs ES indices**: The GraphQL type names (e.g. `case`) map to ES index
  aliases (e.g. `ardac_case`). The `_status` endpoint shows the mapping.
- **`_refresh`**: Admin-only endpoint (`POST /guppy/_refresh`) reloads the schema from ES.
  Requires a token with `admin_access` on `/guppy_admin` resource in Arborist.

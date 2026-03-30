# gen3

A fast, profile-aware command-line interface for the [Gen3](https://gen3.org) data platform. Built in Rust.

## Installation

Download the latest binary for your platform from the [Releases](https://github.com/alan-walsh/gen3-cli/releases) page:

| Platform | File |
|---|---|
| Linux (x86\_64) | `gen3-linux-x86_64` |
| macOS (Apple Silicon) | `gen3-macos-aarch64` |
| Windows (x86\_64) | `gen3-windows-x86_64.exe` |

Make the binary executable and place it on your `PATH`:

```bash
chmod +x gen3-linux-x86_64
mv gen3-linux-x86_64 /usr/local/bin/gen3
```

### Build from source

Requires [Rust](https://rustup.rs) stable.

```bash
git clone https://github.com/alan-walsh/gen3-cli.git
cd gen3-cli
cargo build --release
# binary is at target/release/gen3
```

---

## Quick start

**1. Download your API credentials** from your Gen3 commons profile page (`/identity`). Save the file as `credentials.json`.

**2. Set up a profile:**

```bash
gen3 auth setup
# Enter your commons URL (e.g. https://new.portal.ardac.org)
# Enter the path to your credentials.json
# Choose a profile name (default: "default")
```

This saves your endpoint and API key to `~/.gen3/config`.

**3. Run your first query:**

```bash
# Check that Peregrine is up
gen3 peregrine system status

# Count records in Guppy
gen3 guppy aggregation counts --type case

# Fetch an Indexd record
gen3 indexd records get --guid dg.xxxx/your-guid-here
```

---

## Commands

### `gen3 auth`

| Sub-command | Description |
|---|---|
| `auth setup` | Configure a profile (endpoint + credentials) |

### `gen3 config`

| Sub-command | Description |
|---|---|
| `config list` | List all configured profiles |
| `config use <name>` | Switch the active profile |
| `config show` | Show the active profile details |

### `gen3 sheepdog`

Data submission service. Auth required for all write operations.

| Sub-command | Description |
|---|---|
| `sheepdog programs list` | List all programs |
| `sheepdog projects list --program <prog>` | List projects in a program |
| `sheepdog entities list --program <prog> --project <proj> --node <type>` | List entities of a node type |

### `gen3 indexd`

GUID-based file index (hashes, sizes, storage URLs).

| Sub-command | Description |
|---|---|
| `indexd records get --guid <guid>` | Fetch a record by GUID |
| `indexd records list` | List records (supports filters) |
| `indexd records create` | Create a new index record |
| `indexd blank create` | Reserve a GUID before upload |
| `indexd blank update --guid <guid>` | Fill in hashes/size after upload |
| `indexd aliases list --guid <guid>` | List aliases for a GUID |
| `indexd bulk get --guids <g1,g2,...>` | Fetch multiple records at once |
| `indexd bundles get --guid <guid>` | Fetch a bundle record |
| `indexd system status` | Health check |
| `indexd system stats` | Record counts and hash distribution |

### `gen3 peregrine`

GraphQL read-only metadata query service.

| Sub-command | Description |
|---|---|
| `peregrine graphql query --query '<gql>'` | Run a raw GraphQL query |
| `peregrine graphql introspect` | Introspect the full schema |
| `peregrine metadata schema` | Fetch the data dictionary schema |
| `peregrine system status` | Health check |
| `peregrine system version` | Version and dictionary version |

### `gen3 guppy`

Elasticsearch-backed aggregation, faceted counts, histograms, and bulk record download.

| Sub-command | Description |
|---|---|
| `guppy graphql query --type <type> --query '<gql>'` | Run a raw Guppy GraphQL query |
| `guppy graphql introspect` | Introspect the Guppy schema |
| `guppy aggregation counts` | Total record counts across all types |
| `guppy aggregation counts --type <type>` | Count for a specific type (with optional `--filter`) |
| `guppy aggregation histogram --type <type> --field <field>` | Bar chart histogram for a field |
| `guppy mapping list --type <type>` | List all fields for a type |
| `guppy download records --type <type>` | Download all records as JSON |
| `guppy system status` | Health check with index names |
| `guppy system version` | Version info |
| `guppy system indices` | List all Elasticsearch indices |

---

## Configuration

Credentials are stored in `~/.gen3/config` (TOML). Each named profile holds:

```toml
active_profile = "default"

[profiles.default]
api_endpoint = "https://new.portal.ardac.org"
api_key      = "eyJ..."   # from credentials.json
key_id       = "a1b2c3d4-..."
```

The CLI exchanges your `api_key` for a short-lived Bearer token automatically before each authenticated request.

Switch profiles with `gen3 config use <name>`.

---

## Skills (AI agent interface)

The `skills/` directory contains `SKILL.md` files that describe each Gen3 service for use with AI coding assistants and agentic CLI tools. Skills follow the same pattern as the `gws` (Google Workspace) CLI: one directory per service with a detailed reference covering endpoints, auth, request/response shapes, and common workflows.

See [`skills/README.md`](skills/README.md) for the full skills index.

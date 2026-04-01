# gen3

A fast, profile-aware command-line interface for the [Gen3](https://gen3.org) data platform. Built in Rust.

## Installation

Download the latest archive for your platform from the [Releases](https://github.com/alan-walsh/gen3-cli/releases) page:

| Platform | File |
|---|---|
| Linux (x86\_64) | `gen3_Linux_amd64.tar.gz` |
| macOS (Apple Silicon) | `gen3_Darwin_arm64.tar.gz` |
| Windows (x86\_64) | `gen3_Windows_amd64.zip` |

Extract and place the binary on your `PATH`:

**Linux / macOS**
```bash
tar -xzf gen3_Linux_amd64.tar.gz   # or gen3_Darwin_arm64.tar.gz
chmod +x gen3
mv gen3 /usr/local/bin/gen3
```

**Windows** (PowerShell)
```powershell
Expand-Archive gen3_Windows_amd64.zip -DestinationPath gen3_Windows_amd64
# Move gen3.exe to a directory in your PATH
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

The `skills/` directory contains a `SKILL.md` reference file for each Gen3 service. These files give AI agent CLIs (GitHub Copilot CLI, Claude, Gemini CLI, and others) the context they need to understand Gen3 endpoints, auth flows, request/response shapes, and common workflows — so the agent can operate the CLI on your behalf.

Each release archive includes the full `skills/` tree alongside the binary.

See [`skills/README.md`](skills/README.md) for the full index of available skills.

### Deploying skills for agent use

After extracting the release archive, register the skills with your agent CLI of choice.

#### GitHub Copilot CLI

Copy the skill directories into your project's extensions folder (project-scoped) or your user extensions directory (available in every project):

```bash
# Project-scoped (checked into your repo)
cp -r skills/gen3-* .github/extensions/

# User-scoped (available everywhere)
# Linux / macOS
cp -r skills/gen3-* ~/.config/gh-copilot/extensions/

# Windows (PowerShell)
Copy-Item -Recurse skills\gen3-* "$env:APPDATA\GitHub Copilot CLI\extensions\"
```

#### Claude / Claude Code

Add the skills directory as project context so Claude can read it during a session:

```bash
# From your project root — point Claude at the skills tree
cp -r skills/ .claude/gen3-skills/
```

Or reference individual `SKILL.md` files directly in your `CLAUDE.md`:

```markdown
<!-- CLAUDE.md -->
@.claude/gen3-skills/gen3-shared/SKILL.md
@.claude/gen3-skills/gen3-indexd/SKILL.md
```

#### Gemini CLI

Place the skills in your project so Gemini picks them up as context:

```bash
cp -r skills/ .gemini/gen3-skills/
```

Then reference them in your `GEMINI.md`:

```markdown
<!-- GEMINI.md -->
See .gemini/gen3-skills/ for Gen3 service documentation.
```

#### Any agent that accepts markdown context

Every `SKILL.md` is plain markdown. Load individual files directly into your agent's context window, paste them into a system prompt, or point your tool at the `skills/` directory — whichever your agent supports.

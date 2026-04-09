---
allowed-tools: Bash, Read, Write, Edit
argument-hint: "[subcommand] [flags]"
description: "Gen3 Auth: Profile management and credential setup operations."
---

# Gen3 Auth

Execute Gen3 Auth operations: $ARGUMENTS

## Prerequisites

- Read `../gen3-shared/SKILL.md` first for shared auth and environment guidance.
- Confirm the credentials JSON file has been downloaded from the target Gen3 data commons UI before running `setup`.
- Ensure `~/.gen3/` is writable before creating or updating profiles.

## Command Shape

```bash
gen3 auth <subcommand> [flags]
```

## Service Summary

`gen3 auth` manages named profiles stored in `~/.gen3/config`. Each profile holds the API endpoint URL for a Gen3 data commons, an API key (JWT), and a key ID. Multiple profiles are supported, allowing users to switch between different commons or environments without re-entering credentials.

## Config File Format

Profiles are persisted in TOML format at `~/.gen3/config`:

```toml
[profiles.default]
api_endpoint = "https://my-commons.example.org"
api_key = "<JWT token from credentials JSON file>"
key_id = "<UUID key identifier>"

[profiles.staging]
api_endpoint = "https://staging.example.org"
api_key = "..."
key_id = "..."
```

## Credentials JSON Input Format

The credentials JSON file is downloaded from the Gen3 data commons UI (via the Fence service's API key download). Its format is:

```json
{
  "api_key": "<JWT token>",
  "key_id": "<UUID>"
}
```

## Available Subcommands

### setup

Interactive TUI wizard that configures a new profile or overwrites an existing one.

Prompts:
1. **Profile name** — name for this profile (default: `"default"`)
2. **API endpoint URL** — base URL of the target Gen3 data commons (e.g. `https://my-commons.example.org`)
3. **Credentials file path** — path to the credentials JSON file downloaded from the commons UI

Saves the resulting profile to `~/.gen3/config`.

```bash
gen3 auth setup
```

## Usage Notes

1. Profiles are selected by name when running other `gen3` subcommands; use the profile name that matches the target commons.
2. The credentials JSON file format is produced by the Fence service's API key download endpoint — download it from the commons UI before running `setup`.
3. The config file is stored in TOML format at `~/.gen3/config`; it can be edited manually if needed.
4. Multiple profiles allow seamless switching between production, staging, or other Gen3 environments.

## Task

Use this skill to plan or execute Gen3 Auth profile and credential work in the Gen3 CLI. Start by identifying the target profile name, the commons URL, and the location of the downloaded credentials JSON file.

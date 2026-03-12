# Skills scaffold

This repository now includes an initial `skills/` scaffold for a skills-driven Gen3 CLI.

## Layout

The structure mirrors the broad Google Workspace CLI pattern:

- one directory per skill
- one `SKILL.md` per skill directory
- a shared skill for common auth and environment guidance
- service-specific skills that can later map cleanly onto Rust + Ratatui commands

## Current skills

- `gen3-shared`
- `gen3-auth`
- `gen3-fence`
- `gen3-arborist`
- `gen3-sheepdog`
- `gen3-peregrine`
- `gen3-guppy`
- `gen3-indexd`
- `gen3-requestor`
- `gen3-metadata-service`
- `gen3-manifestservice`
- `gen3-hatchery`
- `gen3-audit-service`
- `gen3-workspace-token-service`

## Next steps

- refine each service with concrete REST or GraphQL resources
- add helper and workflow skills on top of the service skills
- wire the skill names and descriptions into the Rust CLI command surface

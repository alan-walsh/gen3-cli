# Skills

The `skills/` directory provides a `SKILL.md` reference file for each Gen3 microservice. These files drive the AI agent interface for the `gen3` CLI and follow the same pattern as the `gws` (Google Workspace) CLI.

## Layout

```
skills/
  gen3-shared/        # Shared auth, config, and cross-service guidance
  gen3-fence/         # Fence: AuthN/AuthZ, credentials, signed URLs
  gen3-indexd/        # Indexd: GUID-based file index
  gen3-peregrine/     # Peregrine: GraphQL metadata queries
  gen3-guppy/         # Guppy: Elasticsearch aggregation and download
  gen3-sheepdog/      # Sheepdog: data submission
  gen3-metadata-service/  # MDS: per-GUID object metadata
  gen3-manifestservice/   # Manifest Service: file manifest CRUD
  gen3-audit-service/     # Audit: log export
  gen3-workspace-token-service/  # WTS: workspace-scoped tokens
  gen3-hatchery/      # Hatchery: workspace lifecycle
  gen3-arborist/      # Arborist: authz policy engine
  gen3-requestor/     # Requestor: data access request workflows
```

## Implementation status

| Skill | SKILL.md | Rust commands |
|---|---|---|
| gen3-shared | ✅ | — (shared guidance) |
| gen3-fence | ✅ | — |
| gen3-indexd | ✅ | ✅ `gen3 indexd` |
| gen3-peregrine | ✅ | ✅ `gen3 peregrine` |
| gen3-guppy | ✅ | ✅ `gen3 guppy` |
| gen3-sheepdog | ✅ | ✅ `gen3 sheepdog` |
| gen3-metadata-service | ✅ | — |
| gen3-manifestservice | ✅ | — |
| gen3-audit-service | ✅ | — |
| gen3-workspace-token-service | ✅ | — |
| gen3-hatchery | ✅ | — |
| gen3-arborist | ✅ | — |
| gen3-requestor | ✅ | — |

Auth and config are implemented directly in `src/auth/` and `src/commands/config/` rather than as a standalone skill.

## Next steps

Services prioritised for Rust implementation:

1. **Fence** — credential management, user info, signed download URLs
2. **Metadata Service (MDS)** — per-GUID metadata store; public listing + auth CRUD
3. **Manifest Service** — create and download file manifests (key part of the Guppy → manifest → download workflow)
4. **Sheepdog** — flesh out existing stub with full submission and export commands

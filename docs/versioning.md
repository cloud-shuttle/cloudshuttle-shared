# Versioning Strategy

CloudShuttle shared libraries follow semantic versioning to ensure predictable releases and backward compatibility.

## Semantic Versioning

We use [Semantic Versioning](https://semver.org/) with the format `MAJOR.MINOR.PATCH`:

- **MAJOR**: Breaking changes that require updates in consuming services
- **MINOR**: New features that are backward compatible
- **PATCH**: Bug fixes and small improvements that are backward compatible

### Examples

- `1.0.0` → `1.1.0`: Added new feature
- `1.1.0` → `1.1.1`: Bug fix
- `1.1.1` → `2.0.0`: Breaking change

## Release Process

### Automated Releases

1. **Feature Development**: Changes are made on feature branches
2. **Pull Request**: Code review and CI checks
3. **Merge to Main**: Triggers release preparation
4. **Version Bump**: Automatic version determination based on changes
5. **Tag Creation**: Git tag created (e.g., `v1.2.3`)
6. **Publishing**: Libraries published to respective registries

### Manual Releases

For special cases, releases can be triggered manually:

```bash
# Create and push version tag
git tag v1.2.3
git push origin v1.2.3
```

## Version Compatibility

### Rust Libraries

- Follow strict semantic versioning
- Breaking changes require major version bump
- Feature flags used for optional functionality

### TypeScript Libraries

- Follow semantic versioning
- Peer dependency ranges allow compatible updates
- Breaking changes clearly documented

## Deprecation Policy

### Deprecation Process

1. **Announcement**: Deprecation warning added to code and documentation
2. **Grace Period**: Minimum 2 minor versions before removal
3. **Migration Guide**: Provided for breaking changes
4. **Removal**: Breaking changes in major version

### Example

```rust
// v1.x - Function available
pub fn old_function() { ... }

// v2.x - Function deprecated
#[deprecated(since = "2.0.0", note = "Use new_function instead")]
pub fn old_function() { ... }

// v3.x - Function removed
// old_function no longer exists
```

## Dependency Management

### Internal Dependencies

Libraries within the monorepo use relative path dependencies during development:

```toml
# In Cargo.toml
cloudshuttle-error-handling = { path = "../error-handling", version = "0.1.0" }
```

### External Dependencies

Published versions use version ranges for compatibility:

```toml
# Consuming service
cloudshuttle-error-handling = "1.0"
```

### TypeScript Dependencies

```json
{
  "dependencies": {
    "@cloudshuttle/components": "^1.0.0"
  }
}
```

## Release Cadence

### Regular Releases

- **Minor Releases**: Every 2 weeks for feature additions
- **Patch Releases**: As needed for bug fixes
- **Major Releases**: Only for breaking changes, coordinated across all services

### Pre-releases

For testing new features:

- **Alpha**: `1.0.0-alpha.1` - Early testing
- **Beta**: `1.0.0-beta.1` - Feature complete
- **RC**: `1.0.0-rc.1` - Release candidate

## Branching Strategy

- `main`: Stable, release-ready code
- `develop`: Integration branch for features
- `feature/*`: Feature branches
- `hotfix/*`: Bug fix branches

## Changelog

All changes documented in `CHANGELOG.md`:

```markdown
## [1.2.3] - 2024-01-15

### Added
- New feature description

### Changed
- Breaking change description

### Fixed
- Bug fix description

### Removed
- Removed feature description
```

## Migration Guides

For major version changes, migration guides provided in `docs/migrations/`.

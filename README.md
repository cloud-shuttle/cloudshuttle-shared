# CloudShuttle Shared Libraries

Reusable code components and utilities that multiple CloudShuttle services depend on. This repository eliminates code duplication and ensures consistency across the platform.

## Overview

The shared libraries repository contains:
- **Rust Libraries**: Core backend utilities for error handling, database access, authentication, observability, etc.
- **TypeScript Libraries**: React components, hooks, types, and utilities for frontend applications
- **Documentation**: Comprehensive guides and API documentation
- **CI/CD**: Automated testing, publishing, and release management

## Repository Structure

```
cloudshuttle-shared/
├── rust/                          # Rust shared libraries
│   ├── error-handling/           # Standardized error types
│   ├── database/                 # Database utilities
│   ├── auth/                     # Authentication helpers
│   ├── observability/            # Logging & metrics
│   ├── config/                   # Configuration management
│   ├── api/                      # API utilities
│   ├── validation/               # Input validation
│   └── crypto/                   # Cryptographic utilities
├── typescript/                   # TypeScript shared libraries
│   ├── components/              # React components
│   ├── hooks/                   # React hooks
│   ├── types/                   # TypeScript types
│   ├── utils/                   # Utility functions
│   ├── api/                     # API client utilities
│   └── stores/                  # State management
├── docs/                        # Documentation
│   ├── rust-libraries.md        # Rust library guide
│   ├── typescript-libraries.md  # TypeScript library guide
│   ├── versioning.md            # Versioning strategy
│   └── contributing.md          # Contribution guidelines
├── .github/                     # CI/CD workflows
│   └── workflows/
│       ├── test.yml             # Testing all libraries
│       ├── publish.yml          # Publishing releases
│       └── security.yml         # Security scanning
└── scripts/                     # Build and release scripts
    ├── build-all.sh            # Build all libraries
    ├── test-all.sh             # Test all libraries
    └── release.sh              # Automated release
```

## Quick Start

### Rust Libraries

Add to your `Cargo.toml`:

```toml
[dependencies]
cloudshuttle-error-handling = "0.1.0"
cloudshuttle-database = "0.1.0"
cloudshuttle-auth = "0.1.0"
```

### TypeScript Libraries

```bash
npm install @cloudshuttle/components @cloudshuttle/hooks @cloudshuttle/types
```

## Development

### Prerequisites

- Rust 1.70+
- Node.js 18+
- Cargo
- npm/yarn

### Building

```bash
# Build all Rust libraries
./scripts/build-all.sh

# Build all TypeScript libraries
cd typescript && npm run build
```

### Testing

```bash
# Test all Rust libraries
./scripts/test-all.sh

# Test all TypeScript libraries
cd typescript && npm test
```

### Contributing

See [CONTRIBUTING.md](docs/contributing.md) for detailed contribution guidelines.

## Versioning

This repository follows [Semantic Versioning](docs/versioning.md):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

## Publishing

Libraries are automatically published when tags are pushed:

- Rust crates: Published to [crates.io](https://crates.io)
- TypeScript packages: Published to [npm](https://www.npmjs.com)

## Support

- [Documentation](docs/)
- [Issues](https://github.com/cloudshuttle/cloudshuttle-shared/issues)
- [Discussions](https://github.com/cloudshuttle/cloudshuttle-shared/discussions)

## License

This project is licensed under MIT OR Apache-2.0.

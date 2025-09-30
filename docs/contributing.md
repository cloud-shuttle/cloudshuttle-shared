# Contributing Guide

Welcome! This guide explains how to contribute to CloudShuttle shared libraries.

## Development Setup

### Prerequisites

- **Rust**: 1.70+ with Cargo
- **Node.js**: 18+ with npm
- **Git**: For version control

### Clone and Setup

```bash
git clone https://github.com/cloudshuttle/cloudshuttle-shared.git
cd cloudshuttle-shared

# Install dependencies
./scripts/setup.sh
```

### Development Workflow

1. **Create Feature Branch**
   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make Changes**
   - Follow coding standards
   - Add tests for new functionality
   - Update documentation

3. **Run Tests**
   ```bash
   ./scripts/test-all.sh
   ```

4. **Commit Changes**
   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

5. **Create Pull Request**
   - Push branch to GitHub
   - Create PR with description
   - Wait for CI checks

## Coding Standards

### Rust Code

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting
- Use `clippy` for linting
- Add documentation comments for public APIs

```rust
/// Brief description of the function
///
/// # Arguments
/// * `param` - Description of parameter
///
/// # Returns
/// Description of return value
///
/// # Examples
/// ```
/// // Example usage
/// ```
pub fn my_function(param: Type) -> ReturnType {
    // Implementation
}
```

### TypeScript Code

- Use TypeScript strict mode
- Follow [Airbnb JavaScript Style Guide](https://github.com/airbnb/javascript)
- Use ESLint and Prettier
- Add JSDoc comments for complex functions

```typescript
/**
 * Brief description of the function
 *
 * @param param - Description of parameter
 * @returns Description of return value
 *
 * @example
 * ```typescript
 * // Example usage
 * const result = myFunction(param);
 * ```
 */
export function myFunction(param: Type): ReturnType {
  // Implementation
}
```

## Testing

### Rust Testing

- Unit tests for all public functions
- Integration tests for complex functionality
- Use `#[test]` attribute
- Test error conditions

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        // Test implementation
        assert_eq!(my_function(input), expected);
    }

    #[test]
    fn test_my_function_error() {
        // Test error conditions
        assert!(my_function(invalid_input).is_err());
    }
}
```

### TypeScript Testing

- Unit tests with Jest
- Component tests with React Testing Library
- Hook tests with custom utilities

```typescript
import { render, screen } from '@testing-library/react';
import { MyComponent } from './MyComponent';

test('renders component correctly', () => {
  render(<MyComponent />);
  expect(screen.getByText('Expected text')).toBeInTheDocument();
});
```

## Documentation

### API Documentation

- All public APIs must have documentation
- Include usage examples
- Document error conditions
- Keep examples up to date

### README Updates

- Update README.md for new features
- Add migration guides for breaking changes
- Update installation instructions

## Pull Request Process

### Before Submitting

- [ ] Tests pass locally
- [ ] Code formatted and linted
- [ ] Documentation updated
- [ ] Breaking changes documented
- [ ] Migration guide added (if needed)

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Breaking Changes
List any breaking changes and migration steps

## Checklist
- [ ] Code follows style guidelines
- [ ] Documentation updated
- [ ] Tests pass
- [ ] Ready for review
```

### Review Process

1. **Automated Checks**: CI runs tests and linting
2. **Code Review**: At least 2 maintainers review
3. **Approval**: Maintainers approve changes
4. **Merge**: PR merged to main branch

## Release Process

### For Contributors

Contributors don't need to worry about releases - they're automated.

### For Maintainers

1. **Version Bump**: Update version numbers in Cargo.toml/package.json
2. **Changelog**: Update CHANGELOG.md
3. **Tag**: Create git tag
4. **Publish**: CI automatically publishes

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help newcomers learn
- Follow our community guidelines

## Getting Help

- **Issues**: Bug reports and feature requests
- **Discussions**: Questions and general discussion
- **Documentation**: Check docs/ directory
- **Maintainers**: Contact team for urgent issues

## Recognition

Contributors are recognized in:
- GitHub contributor stats
- CHANGELOG.md entries
- Release notes
- Project documentation

Thank you for contributing to CloudShuttle! 🚀

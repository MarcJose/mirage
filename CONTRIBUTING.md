# Contributing to mirage

Thank you for your interest in contributing to mirage! This document provides guidelines and information for contributors.

## Code of Conduct

This project adheres to a code of conduct that all contributors are expected to follow. Please be respectful and constructive in all interactions.

## Getting Started

### Prerequisites

- Rust 1.89 or later (as specified in Cargo.toml)
- Git

### Development Setup

1. Fork the repository on GitHub
2. Clone your fork locally:

   ```bash
   git clone https://github.com/YOUR_USERNAME/mirage.git
   cd mirage
   ```

3. Build the project:

   ```bash
   cargo build
   ```

4. Run the tests to ensure everything works:

   ```bash
   cargo test
   ```

## Development Workflow

### Building

- **Debug build**: `cargo build`
- **Release build**: `cargo build --release`

### Testing

- **Run all tests**: `cargo test`
- **Run specific test**: `cargo test test_name`
- **Run tests with output**: `cargo test -- --nocapture`
- **Run tests with coverage**: `cargo tarpaulin`

### Running

```bash
# Run with default settings
cargo run

# Run with specific arguments
cargo run -- --country Germany --number 5 --verbose

# Test against local mirrors
cargo run -- --list-countries
```

### Security and Compliance Checks

The project uses cargo-deny for security and license compliance:

```bash
# Run all security and compliance checks
cargo deny check

# Individual checks
cargo deny check advisories  # Security vulnerabilities
cargo deny check licenses    # License compliance
cargo deny check bans        # Banned dependencies
cargo deny check sources     # Source verification
```

## Code Style and Standards

### Rust Guidelines

- Follow standard Rust formatting: `cargo fmt`
- Ensure no clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
- Write comprehensive tests for new functionality
- Document public APIs with rustdoc comments

## Contributing Guidelines

### Reporting Issues

When reporting issues, please include:

- mirage version (`mirage --version`)
- Operating system and version
- Rust version (`rustc --version`)
- Command that caused the issue
- Expected vs actual behavior
- Any error messages

### Feature Requests

Before submitting a feature request:

- Check if similar functionality already exists
- Search existing issues to avoid duplicates
- Explain the use case and rationale
- Consider if it fits the project's scope (Arch Linux mirror management)

### Pull Requests

#### Before Submitting

1. **Create an issue** first to discuss the change
2. **Fork the repository** and create a feature branch
3. **Write tests** for new functionality
4. **Update documentation** if needed (README, man page, etc.)
5. **Ensure all tests pass**: `cargo test`
6. **Check formatting**: `cargo fmt`
7. **Check for warnings**: `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
8. **Run security checks**: `cargo deny check`

#### Pull Request Process

1. Create a descriptive title
2. Reference any related issues
3. Provide a clear description of changes
4. Include test results
5. Update CHANGELOG.md if applicable

#### Review Process

- All PRs require review before merging
- Address feedback promptly
- Keep PRs focused and reasonably sized
- Squash commits if requested

## Architecture Guidelines

### Adding New Filtering Options

1. Add field to `Config` struct in `src/lib.rs`
2. Add corresponding CLI argument in `src/main.rs`
3. Update `filter_mirrors()` function logic
4. Add comprehensive tests
5. Update man page documentation

### Adding New Sorting Methods

1. Add new match arm in `sort_mirrors()` function
2. Add option to clap value parser in CLI
3. Write tests for the new sorting method
4. Update man page with description

### Configuration File Changes

- Maintain backward compatibility
- Follow XDG Base Directory Specification
- Ensure CLI arguments take precedence
- Test config file parsing thoroughly

## Testing Guidelines

### Unit Tests

- Test all public functions
- Cover edge cases and error conditions
- Use descriptive test names
- Include both positive and negative test cases

### Integration Tests

- Test CLI functionality end-to-end
- Test configuration file loading
- Test error handling and user messages

### Test Data

- Use `tempfile` crate for temporary files in tests
- Mock external dependencies when possible
- Don't rely on external network resources in tests

## Documentation

### Code Documentation

- Document all public functions and structs
- Include examples in rustdoc comments
- Explain complex algorithms or business logic

### User Documentation

- Keep README.md up to date
- Update man page for new features
- Include practical examples

## Release Process

### Version Updates

1. Update version in `Cargo.toml`
2. Update version in man page
3. Update PKGBUILD if applicable
4. Create release notes

### Packaging

- Test PKGBUILD on Arch Linux
- Ensure man page installs correctly
- Verify binary works in release mode

## Security Considerations

mirage prioritizes security and all contributors should follow these guidelines:

### Security Practices

- **Never log or expose sensitive information**
- **Validate all user inputs** thoroughly
- **Be cautious with file operations** and filesystem access
- **Use secure defaults** for network operations
- **Report security issues privately** (see SECURITY.md)

### Dependency Security

The project uses cargo-deny to ensure dependency security and license compliance:

- **No banned dependencies**: Avoid `openssl`, old versions of security-sensitive crates
- **License compliance**: Only permissive licenses (MIT, Apache-2.0, BSD, ISC) are allowed
- **Vulnerability scanning**: All dependencies are scanned for known security vulnerabilities
- **Trusted sources**: Dependencies must come from crates.io or approved Git repositories

### Security Configuration

When adding new dependencies:

1. **Check license compatibility**: Ensure the license is allowed in `deny.toml`
2. **Verify security record**: Research the dependency's security history
3. **Prefer memory-safe alternatives**: Choose Rust-native implementations when available
4. **Test security scanning**: Run `cargo deny check` after adding dependencies
5. **Update deny.toml if needed**: Add new acceptable licenses or configure exemptions

### TLS and Network Security

- **Use rustls over OpenSSL**: For memory safety and security
- **Enforce HTTPS**: Never allow unencrypted connections for sensitive data
- **Validate certificates**: Ensure proper certificate validation
- **Handle network errors securely**: Don't leak sensitive information in error messages

## Getting Help

- Create an issue for questions about contributing
- Check existing issues and pull requests
- Review the codebase and tests for examples

## License

By contributing to mirage, you agree that your contributions will be licensed under the same MIT license terms as the project.

# Security Policy

## Reporting Security Vulnerabilities

The security and privacy of mirage users is a top priority. If you discover a security vulnerability, please report it responsibly.

### How to Report

**Please do NOT report security vulnerabilities through public GitHub issues.**

Instead, please report security vulnerabilities by email to:
**[marc@hahn-jose.de](mailto:marc@hahn-jose.de)**

Include the following information in your report:

- A description of the vulnerability
- Steps to reproduce the issue
- Potential impact assessment
- Any suggested fixes (if available)

### Response Timeline

- **Initial Response**: Within 48 hours of receiving the report
- **Confirmation**: Within 7 days, we will confirm the vulnerability
- **Fix Timeline**: Critical vulnerabilities will be addressed within 30 days
- **Disclosure**: Coordinated disclosure after fix is released

## Supported Versions

Security updates are provided for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |

## Security Considerations

### Network Security

mirage makes HTTP/HTTPS requests to the Arch Linux Mirror Status API and mirror servers:

- **API Endpoint**: `https://archlinux.org/mirrors/status/json/` (HTTPS by default)
- **Mirror Testing**: Connections to mirror servers use their configured protocols
- **Certificate Validation**: All HTTPS connections validate SSL certificates
- **Timeout Protection**: Connection and download timeouts prevent hanging requests

### File System Security

mirage interacts with the local file system for:

- **Configuration Files**: Read from XDG-compliant locations (`~/.config/mirage/config`)
- **Mirror List Output**: Write to user-specified paths (with validation)
- **Path Validation**: All file operations validate permissions and path safety

#### File Operation Safety

- Path traversal protection for output files
- Permission validation before file writing
- Parent directory existence checks
- Read-only file detection and reporting

### Input Validation

All user inputs are validated:

- **Regex Patterns**: Include/exclude patterns are validated before compilation
- **Numeric Values**: Timeouts, counts, and percentages are range-checked
- **File Paths**: Output paths are validated for writability and safety
- **URLs**: Custom API URLs are validated (though not recommended)

### Configuration Security

- **Configuration Files**: Parsed safely with quote handling and comment filtering
- **CLI Precedence**: Command-line arguments override configuration files
- **No Credential Storage**: mirage does not store or handle authentication credentials

### Privacy Considerations

- **No Personal Data**: mirage does not collect or transmit personal information
- **API Requests**: Only requests publicly available mirror status data
- **Logging**: Verbose output goes to stderr and contains no sensitive information
- **Caching**: Mirror data is cached locally with configurable timeouts

## Security Best Practices

### For Users

1. **Use HTTPS Mirrors**: Prefer `--protocol https` for package downloads
2. **Verify Mirror Lists**: Review generated mirror lists before using
3. **Regular Updates**: Keep mirage updated to the latest version
4. **Secure File Permissions**: Ensure mirror list files have appropriate permissions
5. **Configuration Security**: Protect configuration files from unauthorized modification

### For System Administrators

1. **Automated Updates**: Implement secure automated mirror list updates
2. **Backup Mirror Lists**: Keep backups of working mirror configurations
3. **Monitor Mirror Health**: Regularly verify mirror functionality
4. **Network Security**: Use firewalls and network monitoring
5. **File System Security**: Implement proper file system permissions

### For Package Maintainers

1. **Signature Verification**: Always verify package signatures regardless of mirror
2. **Mirror Validation**: Test mirrors before adding to package build systems
3. **Secure Distribution**: Use secure channels for distributing mirage packages
4. **Dependency Security**: Keep Rust dependencies updated

## Known Security Considerations

### Network Requests

- **Man-in-the-Middle**: Use HTTPS mirrors to prevent MITM attacks
- **DNS Security**: Consider DNS security measures (DNSSEC, secure DNS)
- **Network Monitoring**: Be aware that mirror requests are visible to network administrators

### File System Access

- **Output File Permissions**: Generated mirror lists inherit umask permissions
- **Configuration File Security**: Configuration files may contain sensitive preferences
- **Temporary Files**: No temporary files are created during normal operation

### Performance Testing

- **Mirror Speed Tests**: Speed testing involves network requests that may be logged by mirrors
- **Concurrent Connections**: Multiple threads may create many simultaneous connections
- **Resource Usage**: High thread counts may impact system performance

## Security Architecture

### Threat Model

mirage operates with the following threat model:

**In Scope:**

- Protection against malicious mirror data
- Safe file system operations
- Input validation and sanitization
- Network request security

**Out of Scope:**

- Package signature verification (handled by pacman)
- Mirror content integrity (handled by pacman)
- System-level security (OS/kernel vulnerabilities)
- Network infrastructure security

### Security Controls

1. **Input Validation**: All inputs are validated before processing
2. **Path Sanitization**: File paths are validated and sanitized
3. **Network Timeouts**: All network operations have timeouts
4. **Error Handling**: Secure error handling prevents information disclosure
5. **Dependency Management**: Regular dependency updates for security patches

## Vulnerability Disclosure Policy

### Coordinated Disclosure

We follow responsible disclosure practices:

1. **Private Reporting**: Security issues should be reported privately
2. **Investigation Period**: Allow time for investigation and fix development
3. **Coordinated Release**: Security fixes are released with appropriate advisory
4. **Public Disclosure**: Details disclosed after fixes are available

### Recognition

Contributors who report security vulnerabilities responsibly will be:

- Credited in security advisories (with permission)
- Listed in project acknowledgments
- Eligible for recognition in release notes

## Security Resources

### External Security Information

- [Arch Linux Security](https://security.archlinux.org/)
- [Rust Security Advisory Database](https://rustsec.org/)
- [OWASP Secure Coding Practices](https://owasp.org/www-project-secure-coding-practices-quick-reference-guide/)

### Security Tools and Testing

- **Static Analysis**: Use `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
- **Dependency Scanning**: Regular `cargo audit` checks
- **Fuzzing**: Consider fuzzing for input validation
- **Security Reviews**: Code reviews with security focus

## Updates and Patches

Security updates will be:

- Released as soon as possible for critical vulnerabilities
- Documented in release notes and security advisories
- Distributed through standard package management channels
- Announced on the project's communication channels

## Questions and Contact

For security-related questions that don't involve vulnerability reports:

- Create a GitHub issue with the `security` label
- Email: [marc@hahn-jose.de](mailto:marc@hahn-jose.de) (for non-sensitive security questions)

---

**Last Updated:** August 2025  
**Policy Version:** 1.0

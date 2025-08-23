# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-08-22

### Added

- Initial release of Mirage - Arch Linux mirror retrieval and filtering tool
- Comprehensive mirror filtering by country, protocol, age, completion percentage
- Multiple sorting methods (age, score, country, delay, duration)
- Performance testing and rating of mirrors with concurrent processing
- Caching system with configurable timeouts for improved performance
- Configuration file support with XDG-compliant paths
- Shell completion generation for bash, zsh, fish, and PowerShell
- Comprehensive man page documentation
- Extensive test coverage (90%+ for most modules)
- Security-focused design with HTTPS-only API access
- Colored output and progress indicators for better user experience
- Systemd service and timer integration examples
- PKGBUILD for Arch Linux packaging

### Features

- Mirror fetching from Arch Linux Mirror Status API
- Advanced filtering options:
  - Country and protocol filtering
  - Age-based filtering (last sync time)
  - Completion percentage thresholds
  - Include/exclude regex patterns
  - Active/inactive mirror selection
  - IPv4/IPv6 support filtering
  - ISO hosting capability filtering
- Sorting algorithms:
  - By last sync time (latest/oldest)
  - By mirror score
  - By geographic location
  - By response delay
  - By connection duration
- Performance testing:
  - Latency measurement
  - Download speed testing
  - Concurrent mirror rating with configurable threads
- Output formats:
  - Standard mirrorlist format compatible with pacman
  - Detailed mirror information display
  - Country distribution listing
- Caching and optimization:
  - Persistent JSON cache with XDG compliance
  - Configurable cache timeouts
  - ETag support for conditional requests
  - Memory-efficient data structures
- Configuration management:
  - XDG-compliant config file locations
  - CLI argument precedence over config files
  - Comprehensive default settings
  - Comment and quote support in config files

### Technical

- Built with Rust 2024 edition for modern language features
- Async/await architecture with Tokio runtime
- Structured logging with tracing framework
- Comprehensive error handling with custom error types
- Memory-safe and performant implementation
- Extensive unit and integration testing (219+ tests)
- Code coverage reporting with tarpaulin
- Security hardening with HTTPS-only connections and TLS 1.2+
- Clippy pedantic linting compliance
- Cross-platform support (Linux primary, with portable design)

---

**Note**: This project follows semantic versioning. This 1.0.0 release represents the initial stable API that is ready for production use.

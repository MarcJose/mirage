# mirage

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.89+-blue.svg)](https://www.rust-lang.org)

A command-line tool for retrieving and filtering Arch Linux mirrors, similar to [reflector](xyne.dev/projects/reflector/). mirage fetches mirror data from the Arch Linux Mirror Status API and provides extensive filtering, sorting, and rating capabilities to help you find the best mirrors for your needs.

## Features

- **Comprehensive Filtering**: Filter by country, protocol, sync age, completion percentage, and more
- **Multiple Sorting Options**: Sort by speed, sync age, country, score, delay, and duration
- **Concurrent Mirror Testing**: Multi-threaded mirror speed testing
- **Configuration Files**: XDG-compliant configuration with CLI override support
- **Caching**: Intelligent caching to reduce API calls
- **Rich Output**: Standard mirrorlist format or detailed mirror information
- **Robust Error Handling**: Comprehensive validation and error reporting
- **Security Hardened**: Uses rustls-based TLS with vulnerability scanning via cargo-deny

## Installation

### From Source

```bash
git clone https://github.com/MarcJose/mirage.git
cd mirage
cargo build --release
sudo cp target/release/mirage /usr/local/bin/
sudo chmod 755 /usr/local/bin/mirage
```

### Installing Man Page

```bash
# Create man directory
MAN_DIR=${MAN_DIR:-/usr/local/share/man/man1}
sudo mkdir -vp "${MAN_DIR}"

# Copy man file
sudo cp man/mirage.1 "${MAN_DIR}/"

# Set permissions
sudo chmod 755 "${MAN_DIR}"
sudo chmod 644 "${MAN_DIR}/mirage.1"
```

### Installing Shell Completions

Pre-generated shell completions are available in the `shell_completions/` directory:

```bash
# Bash (choose one)
sudo cp shell_completions/bash/mirage /etc/bash_completion.d/
# OR for user installation:
mkdir -p ~/.local/share/bash-completion/completions
cp shell_completions/bash/mirage ~/.local/share/bash-completion/completions/

# Zsh (choose one) 
sudo cp shell_completions/zsh/_mirage /usr/share/zsh/site-functions/
# OR for user installation:
mkdir -p ~/.local/share/zsh/site-functions
cp shell_completions/zsh/_mirage ~/.local/share/zsh/site-functions/

# Fish (choose one)
sudo cp shell_completions/fish/mirage.fish /usr/share/fish/completions/
# OR for user installation:
mkdir -p ~/.config/fish/completions
cp shell_completions/fish/mirage.fish ~/.config/fish/completions/

# Set permissions
sudo chmod 644 "/path/to/shell/completion"
```

See [`shell_completions/README.md`](shell_completions/README.md) for more details.

### Systemd Service Installation (Automated Mirror Updates)

For automatic daily mirror updates, install the included systemd service files:

#### 1. Install Service Files

```bash
# Copy systemd service files
sudo cp systemd/mirage.service /etc/systemd/system/
sudo cp systemd/mirage.timer /etc/systemd/system/
sudo cp systemd/mirage.tmpfiles /etc/tmpfiles.d/mirage.conf

# Set proper permissions
sudo chmod 644 /etc/systemd/system/mirage.{service,timer}
sudo chmod 644 /etc/tmpfiles.d/mirage.conf
```

#### 2. Create Configuration Directory

```bash
# Create configuration directories and files
sudo systemd-tmpfiles --create /etc/tmpfiles.d/mirage.conf

# Copy configuration file
sudo cp systemd/mirage.conf /etc/mirage/
sudo chmod 644 /etc/mirage/mirage.conf
```

#### 3. Configure Mirror Selection

Edit `/etc/mirage/mirage.conf` to customize mirror selection:

```bash
sudo nano /etc/mirage/mirage.conf
```

#### 4. Enable and Start Services

```bash
# Reload systemd configuration
sudo systemctl daemon-reload

# Enable timer for daily updates
sudo systemctl enable mirage.timer

# Start timer immediately
sudo systemctl start mirage.timer

# Check timer status
sudo systemctl status mirage.timer
```

#### 5. Manual Execution and Testing

```bash
# Test the service manually
sudo systemctl start mirage.service

# Check service status and logs
sudo systemctl status mirage.service
sudo journalctl -u mirage.service

# View timer schedule
sudo systemctl list-timers mirage.timer
```

## Quick Start

```bash
# List all available mirrors
mirage

# Get 10 fastest mirrors from Germany
mirage --country Germany --fastest 10

# Get HTTPS mirrors that synchronized within the last 6 hours
mirage --protocol https --age 6

# Save a custom mirrorlist
mirage --country "United States" --protocol https --number 5 --save /etc/pacman.d/mirrorlist
```

## Usage

### Basic Examples

```bash
# List all mirrors
mirage

# Filter by country (name or code)
mirage --country Germany
mirage --country DE
mirage --country "United States,Canada"

# Filter by protocol
mirage --protocol https
mirage --protocol "https,ftp"

# Filter by synchronization age (hours)
mirage --age 24

# Get detailed mirror information
mirage --info --country Germany --number 3
```

### Advanced Filtering

```bash
# Mirrors with 100% completion in the last 12 hours
mirage --age 12 --completion-percent 100

# IPv6-enabled HTTPS mirrors with low delay
mirage --ipv6 --protocol https --delay 2

# Exclude slow mirrors using regex
mirage --exclude "\.slow\." --fastest 10

# ISO-hosting mirrors only
mirage --isos --country "Germany,France"
```

### Sorting and Limiting

```bash
# Sort by different criteria
mirage --sort age        # Most recently synchronized first
mirage --sort score      # Highest score first
mirage --sort country    # Alphabetically by country
mirage --sort delay      # Lowest delay first
mirage --sort duration   # Fastest connection first

# Limit results
mirage --number 10           # Maximum 10 mirrors
mirage --fastest 5           # 5 fastest mirrors
mirage --latest 8            # 8 most recently synchronized
mirage --score 3             # 3 highest scoring mirrors
```

### Configuration Files

mirage supports configuration files for persistent settings. Files are searched in order:

1. `$XDG_CONFIG_HOME/mirage/config`
2. `~/.config/mirage/config`
3. `~/.mirage/config`

#### Configuration File Format

```bash
# Default configuration for mirage
--country Germany
--country "United States"
--protocol https
--completion-percent 95
--age 24
--sort score
--verbose
```

**Features:**

- One argument per line
- Comments start with `#`
- Quoted arguments for values with spaces
- CLI arguments override config file settings

## Command-Line Options

### Connection Options

- `--connection-timeout N` - Connection timeout in seconds (default: 5)
- `--download-timeout N` - Download timeout in seconds (default: 5)
- `--cache-timeout N` - Cache timeout in seconds (default: 300)
- `--url URL` - Mirror status API URL
- `--threads N` - Number of threads for mirror rating

### Filtering Options

- `-a, --age N` - Only mirrors synchronized within N hours
- `--delay N` - Only mirrors with sync delay ≤ N hours
- `-c, --country COUNTRY` - Filter by country name or code
- `-p, --protocol PROTOCOL` - Filter by protocol (http, https, ftp)
- `--completion-percent N` - Minimum completion percentage (default: 100)
- `--isos` - Only mirrors hosting ISOs
- `--ipv4` - Only IPv4-enabled mirrors
- `--ipv6` - Only IPv6-enabled mirrors
- `-i, --include REGEX` - Include URLs matching regex
- `-x, --exclude REGEX` - Exclude URLs matching regex

### Output Options

- `--sort METHOD` - Sort by: age, rate, country, score, delay, duration, duration-std
- `-f, --fastest N` - Return N fastest mirrors
- `-l, --latest N` - Return N most recently synchronized mirrors
- `--score N` - Return N highest scoring mirrors
- `-n, --number N` - Maximum number of mirrors
- `--save PATH` - Save mirrorlist to file
- `--info` - Show detailed mirror information
- `--list-countries` - Show mirror distribution by country
- `--verbose` - Show extra information

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Reporting Security Issues

For security-related issues, please see [SECURITY.md](SECURITY.md).

## License

This project is licensed under:

- MIT License ([LICENSE](LICENSE))

## Acknowledgments

- Inspired by the original [reflector](https://xyne.archlinux.ca/projects/reflector/) tool
- Built for the Arch Linux community
- Uses the official [Arch Linux Mirror Status API](https://archlinux.org/mirrors/status/json/)

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release history.

## Disclaimer

This project is an independent implementation and is not affiliated with, endorsed by, or connected to:

- **Arch Linux** or the Arch Linux development team
- **reflector** or its maintainers
- The official Arch Linux Mirror Status API (though we use their public API)

mirage is developed independently as a community tool. Any issues, bugs, or feature requests should be directed to this project's repository, not to Arch Linux or reflector maintainers.

The name "mirage" was chosen as a descriptive term and does not imply any official relationship with existing projects.

# Shell Completions for mirage

This directory contains pre-generated shell completions for mirage.

## Installation

### Bash

```bash
# System-wide installation (requires root)
sudo cp bash/mirage /etc/bash_completion.d/

# User installation
mkdir -p ~/.local/share/bash-completion/completions
cp bash/mirage ~/.local/share/bash-completion/completions/
```

### Zsh

```bash
# System-wide installation (requires root)
sudo cp zsh/_mirage /usr/share/zsh/site-functions/

# User installation
mkdir -p ~/.local/share/zsh/site-functions
cp zsh/_mirage ~/.local/share/zsh/site-functions/
# Add to ~/.zshrc: fpath=(~/.local/share/zsh/site-functions $fpath)
```

### Fish

```bash
# System-wide installation (requires root)
sudo cp fish/mirage.fish /usr/share/fish/completions/

# User installation
mkdir -p ~/.config/fish/completions
cp fish/mirage.fish ~/.config/fish/completions/
```

## Usage

After installation, restart your shell or source your profile to enable completions.

Test completions work by typing:
```bash
mirage --<TAB>
```

## Regenerating Completions

These completions are pre-generated from the CLI definition. If CLI arguments change, they must be manually updated.

### Method 1: Using clap_complete directly

Create a temporary script to generate completions:

```bash
# Create a temporary completion generator script
cat > generate_completions.rs << 'EOF'
use clap::Command;
use clap_complete::{generate_to, shells::{Bash, Fish, Zsh}};
use std::fs;

// Include your CLI definition here (copy from src/cli.rs build_cli function)
// Then generate completions to shell_completions/ directories
EOF

# Run the script (requires setting up dependencies)
# This is only needed when CLI changes
```

# RCargo - Rust Cargo Wrapper with Redirect Target Directory

**Language**: [English](README.md) | [中文](README_cn.md)

RCargo is a high-performance wrapper around Rust's cargo that redirects the target directory to fast storage (typically RAM disk) to dramatically speed up compilation times and save your SSDs.

## ✨ Features

- 🚀 **Per-project target directories**: Each project gets its own isolated cache directory
- 🔧 **Environment variable support**: Customize cache location with `RCARGO_TARGET_DIR`
- 📊 **Cache management**: Built-in commands to view and clean cache
- 🔄 **Cargo pass-through**: All standard cargo commands work transparently
- 💾 **Space-efficient**: Easy cleanup of unused cache directories

## 🚀 Quick Start

### Installation

```bash
# From source
git clone https://github.com/sakarie9/rcargo.git
cd rcargo
cargo install --path .

# Or directly from git
cargo install --git https://github.com/sakarie9/rcargo.git
```

### Basic Usage

Use `rcargo` exactly like `cargo`, but your target will be put in specified directory, default to `/tmp/rcargo_targets`

set `RCARGO_TARGET_DIR` environment varibles to custom target directory.

## 📋 Commands

### Standard Cargo Commands

All standard cargo commands work seamlessly:

```bash
rcargo build          # Build the project
rcargo test           # Run tests  
rcargo run            # Run the project
rcargo check          # Check for errors
rcargo clean          # Clean target directory (cargo's clean)
```

### RCargo-Specific Commands

#### View Cache Sizes

```bash
# Show current project cache size
rcargo size

# Show all cached projects
rcargo size --all
```

**Example output:**

```text
Current project 'my-app' target size: 125.4 MB
```

```text
All cached project target directories:
  my-app-a1b2c3d: 125.4 MB
  web-server-x7y8z9: 89.2 MB  
  cli-tool-m4n5o6: 45.8 MB
Total cache size: 260.4 MB
```

#### Clean Cache

```bash
# Clean current project cache (with confirmation)
rcargo purge

# Clean all project caches (with confirmation)
rcargo purge --all

# Skip confirmation prompt
rcargo purge --yes
rcargo purge --all --yes
```

#### Version Information

```bash
# Shows both rcargo and cargo versions
rcargo --version
```

## ⚙️ Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RCARGO_TARGET_DIR` | Custom target directory location | `/tmp/rcargo_targets` |
| `RCARGO_NO_TARGET_LINK` | Not create link to target | `false` |
| `RCARGO_TARGET_LINK_NAME` | Target link name in project root | `target_rcargo` |

### Examples

```bash
# Use custom cache directory
export RCARGO_TARGET_DIR="/mnt/ramdisk/cargo_cache"
rcargo build

# Use SSD for cache
export RCARGO_TARGET_DIR="/fast-ssd/cargo_cache" 
rcargo build
```

## 📁 Cache Directory Structure

RCargo creates unique cache directories using the format:

```text
{project_name}-{path_hash}
```

Where:

- `project_name`: Extracted from `Cargo.toml` or directory name
- `path_hash`: 7-character MD5 hash of the project path

**Example:**

```text
/tmp/rcargo_targets/
├── my-web-app-a1b2c3d/     # /home/user/projects/my-web-app
├── my-web-app-x7y8z9a/     # /home/user/work/my-web-app (different path)
└── cli-tool-m4n5o6p/       # /home/user/tools/cli-tool
```

This ensures projects with the same name but different locations get separate caches.

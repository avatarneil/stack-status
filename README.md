# stack-status

A lean CLI tool that displays Graphite stack hierarchy with live CI status check progress. Also functions as an MCP server for integration with AI assistants.

## Features

- **Stack Visualization**: Display your Graphite stack hierarchy with PR numbers
- **Live CI Status**: Real-time progress of GitHub Actions and other CI checks
- **Watch Mode**: Auto-refresh display with configurable intervals
- **MCP Server**: Expose status via Model Context Protocol for AI assistant integration
- **Cross-Platform**: Works on macOS, Linux, and Windows

## Prerequisites

- **Graphite CLI (`gt`)**: For stack hierarchy. [Install from graphite.dev](https://graphite.dev/)
- **GitHub CLI (`gh`)**: For CI check status. [Install from cli.github.com](https://cli.github.com/)

The tool gracefully degrades if either CLI is missing.

## Installation

### From Source

```bash
# Clone and build
git clone https://github.com/yourusername/stack-status
cd stack-status
cargo build --release

# Install globally
cargo install --path .
```

### Pre-built Binaries

Download from the [releases page](https://github.com/yourusername/stack-status/releases).

## Usage

### Basic Usage

```bash
# Show current stack status
stack-status

# Show with detailed CI checks
stack-status --details

# Output as JSON
stack-status --json
```

### Watch Mode

```bash
# Live refresh every 10 seconds (default)
stack-status --watch

# Custom refresh interval (5 seconds)
stack-status --watch --interval 5
```

### MCP Server Mode

```bash
# Run as MCP server (stdio transport)
stack-status --mcp
```

## Output Example

```
╭─────────────────────────────────────────────────────────╮
│  Stack Status                         Updated: 12:34:56 │
╰─────────────────────────────────────────────────────────╯

◉ add-dark-mode (#247)
  ◐ 2/3 running
    ├─ ✓ lint (12s)
    ├─ ◐ test
    └─ ○ build

◯ refactor-theme (#246)
  ✓ 3/3 passed

◯ setup-theming (#245)
  ✗ 1 failed

● main
```

### Status Icons

| Icon | Meaning |
|------|---------|
| ◉ | Current branch |
| ◯ | Stack branch |
| ● | Trunk (main/master) |
| ✓ | Passed |
| ✗ | Failed |
| ◐ | Running |
| ○ | Queued/Skipped |
| ⊘ | Cancelled |

## MCP Integration

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "stack-status": {
      "command": "/path/to/stack-status",
      "args": ["--mcp"]
    }
  }
}
```

### Claude Code

Add to `.claude/mcp.json` in your project:

```json
{
  "mcpServers": {
    "stack-status": {
      "command": "stack-status",
      "args": ["--mcp"]
    }
  }
}
```

### Available MCP Tools

| Tool | Description |
|------|-------------|
| `get_stack_status` | Get full stack with CI status for all PRs |
| `get_pr_checks` | Get detailed checks for a specific branch |
| `get_branch_info` | Get info about the current branch |

## CLI Options

```
Options:
  -w, --watch                Watch mode: continuously refresh status
  -i, --interval <SECONDS>   Refresh interval in seconds [default: 10]
  -b, --branch <BRANCH>      Show specific branch's stack
      --json                 Output as JSON
      --mcp                  Run as MCP server (stdio transport)
  -d, --details              Show detailed check information
  -h, --help                 Print help
  -V, --version              Print version
```

## Dependencies

This tool is designed to be lean:

- Shells out to `gt` and `gh` CLI tools (no API tokens needed)
- Minimal Rust dependencies
- Single binary (~1MB release build)

## Building

```bash
# Debug build
cargo build

# Release build (optimized for size)
cargo build --release

# Run tests
cargo test
```

## License

MIT

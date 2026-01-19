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

```bash
# Install via cargo (recommended)
cargo install --git https://github.com/avatarneil/stack-status
```

### From Source

```bash
git clone https://github.com/avatarneil/stack-status
cd stack-status
cargo install --path .
```

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

### Quick Setup (One-Liners)

**Claude Code (CLI):**
```bash
# Project-level (recommended)
mkdir -p .claude && echo '{"mcpServers":{"stack-status":{"command":"stack-status","args":["--mcp"]}}}' > .claude/mcp.json

# Or global (all projects)
mkdir -p ~/.claude && echo '{"mcpServers":{"stack-status":{"command":"stack-status","args":["--mcp"]}}}' >> ~/.claude/mcp.json
```

**Claude Desktop (macOS):**
```bash
# First, backup existing config if any
cp ~/Library/Application\ Support/Claude/claude_desktop_config.json ~/Library/Application\ Support/Claude/claude_desktop_config.json.bak 2>/dev/null || true

# Add stack-status MCP server
cat > ~/Library/Application\ Support/Claude/claude_desktop_config.json << 'EOF'
{
  "mcpServers": {
    "stack-status": {
      "command": "stack-status",
      "args": ["--mcp"]
    }
  }
}
EOF
```

**Cursor:**
```bash
# Add to Cursor's MCP config
mkdir -p ~/.cursor && echo '{"mcpServers":{"stack-status":{"command":"stack-status","args":["--mcp"]}}}' > ~/.cursor/mcp.json
```

**Windsurf:**
```bash
# Add to Windsurf's MCP config
mkdir -p ~/.codeium/windsurf && echo '{"mcpServers":{"stack-status":{"command":"stack-status","args":["--mcp"]}}}' > ~/.codeium/windsurf/mcp.json
```

**VS Code + Continue:**
```bash
# Add to Continue's config
mkdir -p ~/.continue && cat >> ~/.continue/config.json << 'EOF'
{
  "mcpServers": [
    { "name": "stack-status", "command": "stack-status", "args": ["--mcp"] }
  ]
}
EOF
```

### Manual Configuration

<details>
<summary>Claude Desktop</summary>

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

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
</details>

<details>
<summary>Claude Code</summary>

Add to `.claude/mcp.json` in your project (or `~/.claude/mcp.json` for global):

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
</details>

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

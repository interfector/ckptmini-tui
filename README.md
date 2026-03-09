# ckptmini-tui

A terminal user interface (TUI) for [ckptmini](https://github.com/anomalyco/ckptmini), a checkpoint/restore tool for Linux processes.

## Features

- **Process Management** - Browse and search running processes
- **Memory Inspection** - View memory regions with hex dump capability
- **Checkpoint Operations** - Create, restore, and delete process checkpoints

## Installation

```bash
cargo build --release
```

## Usage

```bash
# Run with default paths
cargo run

# Or specify custom paths
cargo run -- [ckptmini-path] [checkpoint-directory]
```

Default paths:
- ckptmini: `/usr/local/bin/ckptmini`
- checkpoint directory: `/tmp/checkpoints`

## Keyboard Shortcuts

### Navigation
| Key | Action |
|-----|--------|
| `Tab` | Switch between tabs |
| `Space` | Toggle focus between list and output panels |
| `j` / `k` or `↓` / `↑` | Navigate up/down |
| `PageUp` / `PageDown` | Page through list |
| `Home` / `End` | Jump to start/end |
| `Enter` | Select / Enter subview |

### Processes Tab
| Key | Action |
|-----|--------|
| `/` | Search processes |
| `c` | Create checkpoint |
| `M` | Sort by memory |
| `P` | Sort by PID |
| `N` | Sort by name |
| `r` | Refresh process list |

### Memory Tab
| Key | Action |
|-----|--------|
| `v` or `Enter` | View hex dump of memory region |
| `/` | Search in hex dump (when hex view focused) |

### Checkpoints Tab
| Key | Action |
|-----|--------|
| `u` | Restore checkpoint |
| `d` | Delete checkpoint |

### General
| Key | Action |
|-----|--------|
| `?` | Toggle help overlay |
| `Esc` | Close hex view / Cancel search |
| `q` | Quit |

## Requirements

- Linux
- Rust (latest stable)
- ckptmini binary

## Architecture

ckptmini-tui wraps the ckptmini CLI as a subprocess and parses its output. It uses [ratatui](https://github.com/ratatui/ratatui) for the terminal interface with the Catppuccin Mocha color scheme.

```
ckptmini-tui
├── src/
│   ├── main.rs           # Entry point, event handling
│   ├── lib.rs            # Module exports
│   ├── theme.rs          # Color theme
│   ├── wrapper/          # CLI wrapper
│   ├── models/           # Data structures
│   └── ui/               # UI components
├── Cargo.toml
├── Visuals.md            # Design specifications
└── Plan.md               # Implementation plan
```

## License

MIT

---

*This program was generated entirely using AI.*

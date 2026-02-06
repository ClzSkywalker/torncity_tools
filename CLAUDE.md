# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# Project Context — Godot 4.6 + Rust (GDExtension)

**Role**: Senior game engine developer. Prioritize correctness, engine constraints, and maintainability.

## Tech Stack
- **Engine**: Godot 4.6
- **Languages**: Rust (GDExtension), GDScript (UI/scene glue)
- **Build**: Cargo + godot-rust (gdext 0.4.5)
- **Platforms**: Desktop (macOS/Windows), Android

## Architecture

### Godot Side
- Scenes are composition units, not logic containers
- GDScript for: UI, scene wiring, signals
- Avoid heavy logic in `_process` / `_physics_process`
- No blocking operations on main thread

### Rust Side
- Core logic, data processing, performance-critical systems
- Deterministic, testable modules
- Explicit ownership, avoid unsafe

## GDExtension Rules
- Follow lifecycle: `_ready` → init, `_exit_tree` → cleanup
- Don't assume node availability before `_ready`
- Never store raw Node pointers without validation
- Use `Gd<T>` safely

## Code Style

### Rust
- No magic numbers, prefer enums over booleans
- Explicit error handling (`Result`, no silent unwraps)
- No global mutable state
- Small, focused modules

### GDScript
- Typed where possible
- Signals over tight coupling
- Delegate logic to Rust

## Build Commands

### Building
```bash
cargo build -p bin --manifest-path rust/Cargo.toml          # Debug
cargo build -p bin --manifest-path rust/Cargo.toml --release # Release
make build                                                    # Alias
make clippy                                                    # Lint
```

### Running
```bash
godot          # Editor
make godot-lsp # LSP (port 6005)
```

### Exporting
```bash
bash script/package-desktop.sh   # Desktop (auto-detect OS)
bash script/package-android.sh    # Android (requires ANDROID_HOME, ANDROID_NDK_HOME)
bash script/package-all.sh [target]  # all|desktop|mobile|android|ios|mac|win|linux
```

## Project Structure
```
project-root/
├── godot/                 # Godot project
│   ├── addons/rust_tools/ # Build plugin
│   ├── native/            # .gdextension files
│   └── scenes/            # .tscn files
├── rust/                  # Rust workspace
│   ├── bin/               # GDExtension entry (cdylib)
│   └── crates/
│       ├── nodex/         # Godot node classes
│       ├── weav3r/        # Weav3r logic
│       ├── torn_logic/    # Torn game logic
│       ├── tools/         # Shared utilities
│       └── model/         # Data models, errors
└── script/                # Export scripts
```

### Dependencies
```
bin → nodex → weav3r/torn_logic → tools → model
```

## Important Patterns

**Configuration**: Platform-specific paths (see README.md)
- macOS: `~/Library/Application Support/Godot/app_userdata/Torn Trade/settings.cfg`
- Windows: `%APPDATA%\Godot\app_userdata\Torn Trade\settings.cfg`
- Use `CfgTool::new(config_path)` to read/write

**Godot Version**: Android export requires godot-rust 0.4.2. Export scripts handle version switching automatically.

## Output Rules

When generating code:
1. Code first
2. Explain only non-obvious decisions
3. No pseudo-code
4. No speculative features

If requirements unclear:
- List assumptions explicitly
- Don't invent engine behavior

## Do NOT

- Invent Godot APIs that don't exist in 4.6
- Mix web/backend patterns into engine code
- Over-abstract prematurely
- Guess file contents or scene structure

## Performance & Safety

- Avoid allocations in hot paths
- No blocking I/O in runtime
- Validate external data (JSON, files)
- Assume mobile constraints

## Workflow

1. Clarify scene / data flow
2. Define Rust interfaces
3. Integrate with Godot
4. Optimize only when needed

# MoFA Studio - Project Context

> Context document for Claude to quickly restore project understanding.

## Project Overview

MoFA Studio is an **AI voice chat desktop application** built with Rust + Makepad UI framework, using Dora dataflow framework for AI pipeline orchestration.

**Current Status**: UI prototype complete, voice chat dataflow integration in development.

## Tech Stack

| Layer | Technology |
|-------|------------|
| UI Framework | Makepad (GPU-accelerated, wyeworks fork) |
| Language | Rust 2021 + Python 3.12 |
| Async | Tokio (full features) |
| Audio | CPAL 0.15 |
| Dataflow | Dora 0.3.12 |
| ML | PyTorch 2.2, FunASR, Kokoro TTS, Qwen3 |
| Package Mgmt | Cargo (Rust), Pixi (Python) |

## Project Structure

```
mofa-studio/
├── mofa-studio-shell/        # Main app entry (binary)
│   └── src/app.rs            # Main app widget (~1310 lines)
├── mofa-widgets/             # Shared UI component library
│   └── src/theme.rs          # Color palette & font definitions
├── mofa-ui/                  # UI infrastructure
│   └── src/theme.rs          # Runtime theme management
├── mofa-dora-bridge/         # UI↔Dora communication bridge
│   └── src/shared_state.rs   # Thread-safe state container
├── apps/                     # Plugin applications
│   ├── mofa-fm/              # Voice chat (main feature)
│   ├── mofa-settings/        # Provider configuration
│   ├── mofa-debate/          # Debate mode
│   ├── mofa-tts/             # Text-to-speech
│   └── mofa-test-app/        # Test application
├── node-hub/                 # Dora dataflow nodes
│   ├── dora-asr/             # Speech recognition (Python)
│   ├── dora-kokoro-tts/      # TTS synthesis (Python)
│   ├── dora-qwen3/           # LLM inference (Python)
│   ├── dora-maas-client/     # LLM API client (Rust)
│   └── dora-conference-*/    # Conference bridge/controller (Rust)
└── models/                   # Model management & download scripts
```

## Key Architecture Patterns

### Plugin System
- Each app implements `MofaApp` trait (`mofa-ui/src/app_trait.rs`)
- Shell discovers and manages apps via `AppRegistry`
- Apps are fully decoupled from shell

### State Management
- `SharedDoraState`: Arc-wrapped thread-safe state
- Dirty tracking for efficient UI updates
- Scope-based dependency injection

### Theme System
- Tailwind-style color palette (`mofa-widgets/src/theme.rs`)
- Dark/light mode with animated transitions
- Semantic color naming

### Makepad Specifics
- `live_design!` macro for UI definitions
- `#[derive(Live, Widget)]` derive macros
- `DrawQuad`, `DrawText` drawing primitives
- Events: `finger_down`, `finger_up`, `actions`

## Common Commands

```bash
# Build and run
cargo run --release

# Python environment
pixi install
pixi run setup

# Start voice chat dataflow
dora up
dora start apps/mofa-fm/dataflow/voice-chat.yml
```

## Key File Reference

| Purpose | Path |
|---------|------|
| Main entry point | `mofa-studio-shell/src/main.rs` |
| Main app widget | `mofa-studio-shell/src/app.rs` |
| Color/font definitions | `mofa-widgets/src/theme.rs` |
| Runtime theme | `mofa-ui/src/theme.rs` |
| App trait | `mofa-ui/src/app_trait.rs` |
| Dora state bridge | `mofa-dora-bridge/src/shared_state.rs` |
| Voice chat main UI | `apps/mofa-fm/src/screen/mod.rs` |
| Settings UI | `apps/mofa-settings/src/screen/mod.rs` |
| Architecture docs | `ARCHITECTURE.md` |
| Development guide | `APP_DEVELOPMENT_GUIDE.md` |

## Workspace Members

```toml
members = [
    "mofa-studio-shell",  # binary
    "mofa-widgets",       # lib
    "mofa-dora-bridge",   # lib
    "mofa-ui",            # lib
    "apps/*",             # multiple libs
]
```

## Dependency Versions (workspace.dependencies)

- `makepad-widgets`: wyeworks fork, rev 53b2e5c84
- `dora-node-api`: v0.3.12 (git tag)
- `cpal`: 0.15
- `tokio`: 1.x (full)
- `serde`: 1.0

## Development Notes

1. **Makepad Widget Registration**: Must register at compile-time in `live_register`
2. **State Updates**: Set dirty flag after modifying `SharedDoraState`
3. **Theme Switching**: Implemented via shader instance variables, no widget rebuild needed
4. **Adding New Apps**: Follow `APP_DEVELOPMENT_GUIDE.md`, implement `MofaApp` trait

## Pending Features

- [ ] Complete voice chat dataflow integration
- [ ] Multi-participant conference mode
- [ ] Model download progress UI
- [ ] Audio device hot-plug support

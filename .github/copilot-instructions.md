# MoFA Studio - AI Agent Instructions

MoFA Studio is an AI voice chat desktop app built with Rust, Makepad UI framework (GPU-accelerated immediate mode), and Dora dataflow for audio pipelines.

## Architecture Overview

- **UI Layer**: Makepad GPU-accelerated widgets with compile-time `live_design!` macro
- **App System**: Plugin architecture via `MofaApp` trait in `apps/` directory
- **Dataflow**: Dora orchestrates Python (ASR, TTS, LLM) and Rust nodes for real-time audio
- **State**: `SharedDoraState` (Arc<RwLock>) with dirty tracking for UI↔Dora communication
- **Theme**: Tailwind-style color palette with dark mode support via shader instances

### Directory Structure

```
mofa-studio-shell/       # Main binary entry point
mofa-widgets/            # Shared UI components + theme system
mofa-ui/                 # Runtime theme, app traits, shared infrastructure
mofa-dora-bridge/        # Thread-safe state bridge for Dora↔UI
apps/{mofa-fm,mofa-settings,...}  # Plugin apps (libraries)
node-hub/                # Dora dataflow nodes (Python & Rust)
libs/dora-common/        # Shared Python library for Dora nodes
models/                  # Model download scripts and validation
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed module relationships.

## Critical Development Patterns

### Makepad UI Specifics

**Compile-time widgets**: All widget types must be registered at compile time via `live_design!(cx)` - no dynamic loading. Apps provide widgets through `MofaApp::live_design()`.

**State updates**: Use `apply_over()` for runtime changes to shader instances (dark mode, visibility, colors):
```rust
view.apply_over(cx, live!{ visible: true, draw_bg: { dark_mode: 1.0 } });
view.redraw(cx); // Always redraw after apply_over
```

**Event handling order**: Process hover events (`FingerHoverIn/Out`) BEFORE extracting actions - hover state affects widget appearance.

**Color limitations**: Hex colors like `#3b82f6` work in `live_design!` properties but NOT in shader `fn pixel()` code or `apply_over()` - use `vec4(r, g, b, a)` literals instead.

**Theme usage**: Import `use mofa_widgets::theme::*;` for semantic colors (`PANEL_BG`, `TEXT_PRIMARY`, etc.) and fonts (`FONT_REGULAR`, `FONT_MEDIUM`). Dark mode variants end in `_DARK`.

### App Plugin System

Apps must implement `MofaApp` trait in `apps/*/src/lib.rs`:
```rust
impl MofaApp for MyApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "My App",
            id: "my-app",  // Unique identifier
            tab_id: Some(live_id!(my_app_tab)),
            page_id: Some(live_id!(my_page)),
            show_in_sidebar: true,
            ..Default::default()
        }
    }
    fn live_design(cx: &mut Cx) {
        screen::live_design(cx);  // Register app's widgets
    }
}
```

Apps are **fully decoupled** from the shell - no direct shell dependencies allowed. See [APP_DEVELOPMENT_GUIDE.md](APP_DEVELOPMENT_GUIDE.md) for complete guide.

### Dora Dataflow Integration

**Shared state pattern**: Use `SharedDoraState` (in `mofa-dora-bridge`) for thread-safe UI↔Dora communication:
```rust
// Producer (Dora thread)
state.chat.push(ChatMessage { ... });

// Consumer (UI thread, in timer callback)
if let Some(messages) = state.chat.read_if_dirty() {
    // Update UI only when data changed
}
```

**Bridge nodes**: UI-connected nodes use `DoraNode::init_from_node_id()` to find themselves in the graph dynamically without hardcoded ports.

**Lifecycle**: Dataflow starts with `dora up && dora start <yaml>`. UI communicates via Dora events through bridge layer. See [MOFA_DORA_ARCHITECTURE.md](MOFA_DORA_ARCHITECTURE.md) for complete flow.

## Build & Run Commands

**Rust (UI shell)**:
```bash
cargo run --release                    # Run GUI
cargo build                            # Dev build
RUST_LOG=debug cargo run              # With debug logging
```

**Python environment** (for dataflow nodes):
```bash
pixi install                           # Install Python dependencies
pixi run setup                         # Setup environment + install packages
```

**Models** (required for voice chat):
```bash
cd models/model-manager
pixi run python download_models.py --download funasr    # ASR models
pixi run python download_models.py --download kokoro    # TTS models
```

**Dora dataflow** (voice chat):
```bash
dora up                                               # Start daemon
dora start apps/mofa-fm/dataflow/voice-chat.yml      # Start dataflow
dora list                                             # Check running graphs
dora stop <uuid> --grace-duration 0s                 # Stop dataflow
```

**Nix users** (automated):
```bash
./run.sh                               # Handles all setup + starts GUI
```

See [DEPLOY_WITH_NIX.md](DEPLOY_WITH_NIX.md) for Nix-based deployment.

## Project Conventions

**Module organization**: Each app/library has `src/lib.rs` exporting public API and `src/screen.rs` (or `src/screen/mod.rs`) for main UI widget.

**Workspace dependencies**: All dependencies declared in root `Cargo.toml` [workspace.dependencies] for version consistency. Makepad uses `wyeworks` fork at specific commit.

**Dora version pinning**: CLI and Python packages both at `0.3.12` - mismatched versions cause message format incompatibility.

**State management**: Prefer `parking_lot::RwLock` + `AtomicBool` dirty flags over channels for UI↔background thread state.

**Timer lifecycle**: Apps implement `start_timers()`/`stop_timers()` for resource cleanup when switching between apps.

**Live ID naming**: Use consistent prefixes: `my_app_tab` for tabs, `my_page` for content areas, `my_widget` for UI elements.

## Common Pitfalls

- **Missing redraw**: Always call `widget.redraw(cx)` after `apply_over()` or state changes
- **Channel deadlocks**: Use non-blocking `try_send()` in Dora nodes, not blocking `send()`
- **Python env conflicts**: Activate correct pixi environment before running Dora nodes
- **Widget not found**: Ensure `live_design(cx)` called for app in shell's `LiveRegister::live_register()`
- **Dataflow stalls**: Check buffer gauges in UI - backpressure indicates pipeline congestion

## Key Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) - Complete system architecture
- [APP_DEVELOPMENT_GUIDE.md](APP_DEVELOPMENT_GUIDE.md) - Creating new apps
- [MOFA_DORA_ARCHITECTURE.md](MOFA_DORA_ARCHITECTURE.md) - Dataflow integration details
- [WIDGET_GUIDE.md](mofa-widgets/WIDGET_GUIDE.md) - Shared widget usage
- [CONTRIBUTING.md](CONTRIBUTING.md) - Code style and PR process
- [claude.md](claude.md) - Quick reference for Claude conversations

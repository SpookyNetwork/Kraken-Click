# 🦑 Kraken Click — KRK Pointer Control Subsystem

> **Not a separate app.** A sensory-motor layer plugged into the KRK-OS control plane.

Kraken Click is the **real-time perception + interaction layer** for the Kraken Code agent operating surface. It gives agents eyes (screen understanding), hands (input automation), and a voice (cursor intelligence) — all gated by the KRK-PSC safety kernel.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      KRK-OS CORE                            │
│              (state + PSC + control loop)                   │
└──────────────────────────┬──────────────────────────────────┘
                           │
     ┌─────────────────────┼─────────────────────┐
     ▼                     ▼                     ▼
Kraken CLI          Kraken Click            Agent Swarm
(control)        (sensory + motor)         (execution)
     │                     │                     │
     └──────────┬──────────┴──────────┬──────────┘
                ▼                     ▼
          Windows Host         External Tools
```

## Module Map

```
kraken-click/
├── lib.rs            # Crate root — module declarations
├── Cargo.toml        # Package manifest
│
├── pointer/          # Pointer intelligence — "what is under cursor?"
│   ├── mod.rs
│   ├── detector.rs   # UI element detection (WinUI Automation + OCR)
│   ├── semantic.rs   # Semantic labeling of screen regions
│   └── models.rs     # Data models for pointer state
│
├── vision/           # Screen understanding layer
│   ├── mod.rs
│   ├── capture.rs    # Screen capture (Desktop Duplication API)
│   ├── analyzer.rs   # Vision model integration (Gemini/Claude/local VLM)
│   └── ocr.rs        # OCR fallback for text extraction
│
├── overlay/          # Cursor overlay + caption system
│   ├── mod.rs
│   ├── window.rs     # Transparent overlay windows
│   ├── cursor.rs     # Animated cursor highlighting + pointing
│   ├── caption.rs    # Caption bubbles + multi-marker support
│   └── renderer.rs   # Direct2D/DirectWrite rendering
│
├── actions/          # Action layer (PSC-gated)
│   ├── mod.rs
│   ├── click.rs      # Mouse click/drag operations
│   ├── input.rs      # Keyboard input + text typing
│   ├── window.rs     # Window focus, app switching
│   └── gate.rs       # PSC gate hook — all actions pass through here
│
├── agents/           # Agent bridge layer
│   ├── mod.rs
│   ├── router.rs     # Routes pointer queries to the cognition layer
│   └── context.rs    # Screen context packaging for agents
│
├── cognition/        # Provider-agnostic AI runtime abstraction
│   ├── mod.rs
│   ├── runtime.rs    # CognitiveRuntime trait + async dispatch
│   └── types.rs      # Shared types (CognitiveQuery, CognitiveResult, etc.)
│
├── providers/        # Concrete AI provider implementations
│   ├── mod.rs
│   └── anthropic.rs  # Anthropic / Claude API adapter
│
├── integrations/     # External tool integrations
│   ├── mod.rs
│   ├── winrt.rs      # Windows Runtime API bindings
│   └── accessibility.rs # MSAA/UI Automation bindings
│
└── docs/             # Operating manuals + design docs
    ├── ARCHITECTURE.md
    └── POINTER_PROTOCOL.md
```

## Core Principles (from DeepMind AI Pointer)

1. **Maintain the flow** — AI works across all apps, no "AI detours"
2. **Show and tell** — Pointer captures visual/semantic context around it
3. **"This" and "That"** — Natural language shorthand + pointing gestures
4. **Turn pixels into actionable entities** — AI understands what you're pointing at

## Platform

- **Target:** Windows 11 ARM64 (Surface)
- **Language:** Rust (core) + TypeScript (bridge layers)
- **Screen Capture:** Windows Desktop Duplication API (DXGI)
- **UI Detection:** Windows UI Automation API + OCR fallback
- **Overlay:** Direct2D + DirectWrite via windows-rs
- **PSC Gate:** All actions routed through krk-psc

## Status

| Module | Status | Notes |
|--------|--------|-------|
| pointer/ | 🔲 Scaffolded | Interfaces defined, WinRT bindings needed |
| vision/ | 🔲 Scaffolded | Capture + analyzer stubs |
| overlay/ | 🔲 Scaffolded | Window + cursor + caption stubs |
| actions/ | 🔲 Scaffolded | Click/input/window + PSC gate |
| agents/ | 🔲 Scaffolded | Router + context builder (providers live in `providers/`) |
| cognition/ | 🔲 Scaffolded | Provider-agnostic runtime trait + types |
| providers/ | 🔲 Scaffolded | Anthropic adapter (`anthropic.rs`) |
| integrations/ | 🔲 Scaffolded | WinRT + Accessibility bindings |

## Production integration path

Per [ADR 001](../../docs/adr/001-python-orchestrator-as-master.md), **`runtime/orchestration` is the system decision engine**. Kraken Click is perception/embodiment; it does not route agents or choose models.

**`pointer_bridge.py` (`runtime/orchestration/`) is the production integration path.** The Python orchestrator communicates with the pointer subsystem via cross-runtime contracts (`PerceptionRequest` / `PerceptionResult`). Rust FFI binding from the bridge to this crate is a follow-up optimisation, not a production requirement today. See [ADR 002](../../docs/adr/002-kraken-click-prod-gate.md) and [ADR 003](../../docs/adr/003-pointer-bridge-architecture.md).

| Layer | Path | Role |
|-------|------|------|
| Orchestrator | `runtime/orchestration/orchestrator.py` | `personnel_map["pointer"]` → **KrakenClick**; `task_type == "pointer"` → `pointer_worker()` |
| Bridge (prod) | `runtime/orchestration/pointer_bridge.py` | `KrakenClickBridge`, `get_bridge()`, `pointer_query()` — **production Python↔Windows contract** |
| Contracts | `runtime/orchestration/contracts.py` | `PerceptionRequest` / `PerceptionResult` types at the bridge boundary |
| Rust crate | `products/kraken-click/` (this tree) | Target embodiment; Rust FFI wiring is follow-up after `cargo check` is green ([ADR 002](../../docs/adr/002-kraken-click-prod-gate.md)) |

**Hook flow (today):**

```text
Client → orchestrator.process_large_task()
       → dispatch task_type "pointer"
       → pointer_worker() → pointer_bridge.KrakenClickBridge
       → (ctypes / UIA on Windows; Rust crate not yet linked)
```

**When the orchestrator invokes pointer:**

- User asks the agent to interact with the screen
- Agent needs visual context to complete a task
- "Click here" / "What is this?" / "Show me" style queries

**Compile status:** Run `cargo check` in this directory. CI runs the same via `.github/workflows/kraken-products.yml` with a documented non-blocking exclusion until scaffold errors are cleared.

# Kraken Click — Architecture Document

## System Context

Kraken Click is a **sensory-motor layer** for the KRK-OS control plane. It is not a standalone application — it is a subsystem that plugs into Kraken Code's agent orchestration layer.

```
┌─────────────────────────────────────────────────────────────────┐
│                        KRK-OS CORE                              │
│                   (state + PSC + control loop)                  │
│                     Rust: krk-core-v2                           │
└────────────────────────────┬────────────────────────────────────┘
                             │
        ┌────────────────────┼────────────────────┐
        ▼                    ▼                    ▼
  Kraken CLI           Kraken Click           Agent Swarm
  (control plane)      (sensory + motor)      (execution)
  Python + Rust TS     Rust core + TS bridge  Rust + Python
        │                    │                    │
        └──────────┬─────────┴─────────┬──────────┘
                   ▼                   ▼
            Windows Host          External Tools
            (ARM64 Surface)       (Claude, Gemini, etc.)
```

## Module Architecture

```
kraken-click/
│
├── pointer/          ← "What is under the cursor?"
│   ├── models.rs     ← Core data types (ScreenPoint, UIElement, PointerState)
│   ├── detector.rs   ← Multi-strategy element detection
│   └── semantic.rs   ← Semantic labeling + deictic resolution
│
├── vision/           ← "See and understand the screen"
│   ├── capture.rs    ← DXGI Desktop Duplication / GDI fallback
│   ├── analyzer.rs   ← Vision model integration (Gemini/Claude/local)
│   └── ocr.rs        ← OCR fallback (Windows OCR / Tesseract / Azure)
│
├── overlay/          ← "Show, don't tell"
│   ├── window.rs     ← Transparent overlay windows (per-monitor)
│   ├── cursor.rs     ← Animated cursor companion + pointing
│   ├── caption.rs    ← Caption bubbles + multi-marker layout
│   └── renderer.rs   ← Direct2D rendering backend
│
├── actions/          ← "Act on the screen, safely"
│   ├── gate.rs       ← PSC gate — all actions pass through here
│   ├── click.rs      ← Mouse click/drag via SendInput
│   ├── input.rs      ← Keyboard input + text typing
│   └── window.rs     ← Window focus + app switching
│
├── agents/           ← "Connect pointer to intelligence"
│   ├── context.rs    ← Screen context packaging for AI models
│   ├── router.rs     ← Query routing (fast path vs. AI path)
│   └── claude.rs     ← Claude Code / Claude API adapter
│
└── integrations/     ← Windows API bindings
    ├── winrt.rs      ← DXGI, Graphics Capture, Direct2D, SendInput
    └── accessibility.rs ← MSAA / UI Automation
```

## Data Flow

### "What is this?" Query Flow

```
User: "What's that button?"
    │
    ▼
Agent Router (agents/router.rs)
    │
    ├─→ Fast Path: UI Automation hit test (pointer/detector.rs)
    │   └─→ Returns UIElement if found (< 10ms)
    │
    ├─→ Medium Path: Element detector with label matching
    │   └─→ Returns matching UIElements (< 100ms)
    │
    └─→ Slow Path: Vision model (vision/analyzer.rs)
        ├─→ Capture screen region (vision/capture.rs)
        ├─→ Send to Gemini/Claude with question
        └─→ Parse response → UIElement (< 2s)
```

### "Click the Submit button" Action Flow

```
User: "Click the Submit button"
    │
    ▼
Agent Router (agents/router.rs)
    │
    ├─→ Find element: detector.find_by_label("Submit")
    │
    ├─→ PSC Gate evaluation (actions/gate.rs)
    │   ├─→ Rate limit check
    │   ├─→ Permission check
    │   ├─→ Destructive action check
    │   └─→ Audit log
    │
    ├─→ GateDecision::Allow
    │   └─→ ClickExecutor::click() (actions/click.rs)
    │       └─→ SendInput(MOUSEEVENTF_LEFTDOWN/UP)
    │
    └─→ GateDecision::Deny
        └→ Return error to agent
```

### Screen Context Packaging Flow

```
Agent needs screen context
    │
    ▼
ContextPackager (agents/context.rs)
    │
    ├─→ ScreenCapture::capture_screen() (vision/capture.rs)
    │   └─→ DXGI Desktop Duplication → JPEG
    │
    ├─→ ElementDetector::detect_all() (pointer/detector.rs)
    │   ├─→ UI Automation tree walk
    │   └─→ Returns Vec<UIElement>
    │
    ├─→ GetCursorPosition() (Windows API)
    │
    ├─→ GetForegroundWindow() (Windows API)
    │
    └─→ ScreenContext { screenshot, elements, cursor, window }
        │
        ├─→ For vision model: send JPEG directly
        └─→ For text-only: build_text_description() → markdown
```

## PSC Integration

All actions pass through the KRK-PSC (Phase-Stabilizing Controller):

```
Action Request
    │
    ▼
┌─────────────────────────────────────┐
│           PSC GATE                  │
│                                     │
│  1. Rate limiting                   │
│     - Max 10 actions/second         │
│     - Sliding window                │
│                                     │
│  2. Permission check                │
│     - Is this action type allowed?  │
│     - Text input enabled?           │
│     - Window management enabled?    │
│                                     │
│  3. Destructive action check        │
│     - Contains "delete"/"format"?   │
│     - Right-click on system dialog? │
│     → Require user confirmation     │
│                                     │
│  4. Audit logging                   │
│     - Timestamp                     │
│     - Action type + parameters      │
│     - Decision (Allow/Deny)         │
│                                     │
│  5. Decision                        │
│     → Allow / Deny / Confirm / Wait │
└─────────────────────────────────────┘
```

## Windows ARM64 Considerations

| Concern | Approach |
|---------|----------|
| **Screen Capture** | DXGI Desktop Duplication works on ARM64. GPU is Qualcomm Adreno — DDA is hardware-accelerated. |
| **UI Automation** | Fully supported on ARM64. Same COM interfaces as x64. |
| **SendInput** | Works identically on ARM64. No emulation needed. |
| **Direct2D** | Hardware-accelerated on Adreno GPU via DirectX 12. |
| **OCR** | Windows.Media.Ocr works on ARM64. Tesseract has ARM64 builds. |
| **Rust compilation** | `aarch64-pc-windows-msvc` target is stable. All windows-rs features work. |
| **Node.js bridge** | Node.js has official ARM64 builds for Windows. |

## Security Model

1. **No raw pointer data leaves the machine** — screen capture stays local
2. **Vision API calls** send screenshots to AI providers — user must consent
3. **All actions are gated** — PSC evaluates before execution
4. **Audit trail** — every action is logged with timestamp and decision
5. **Rate limiting** — prevents action flooding / runaway automation
6. **Confirmation for destructive actions** — user must approve risky operations

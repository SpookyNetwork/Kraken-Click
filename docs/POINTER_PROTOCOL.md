# Pointer Protocol — Kraken Click Interaction Specification

## Overview

The Pointer Protocol defines how Kraken Click translates between human intent (natural language + pointing) and machine actions (screen coordinates + input events).

It is inspired by the DeepMind AI Pointer's four principles:
1. **Maintain the flow** — AI works across all apps
2. **Show and tell** — Pointer captures visual/semantic context
3. **"This" and "That"** — Natural language shorthand + pointing
4. **Turn pixels into actionable entities** — AI understands what you're pointing at

## Query Types

### 1. Deictic Queries ("This/That/Here/There")

User points at something and says:
- "What is **this**?"
- "Click **that**"
- "Move **this** to **there**"
- "What does **that** mean?"

Resolution:
1. Capture cursor position at time of utterance
2. Run element detection at that position
3. If ambiguous (multiple overlapping elements), use context from utterance
4. Return resolved element + confidence

### 2. Label Queries ("The Submit button")

User refers to an element by description:
- "Click the **Submit button**"
- "Type in the **search field**"
- "Open the **File menu**"

Resolution:
1. Parse description → element type + label keywords
2. Search UI Automation tree for matching elements
3. Filter by interactability
4. Return best match (or multiple if ambiguous)

### 3. Spatial Queries ("The top-right corner")

User refers to a screen region:
- "Click in the **top-right corner**"
- "What's in the **center of the screen**?"
- "Scroll the **left sidebar**"

Resolution:
1. Parse spatial reference → screen region
2. Map to absolute coordinates based on screen resolution
3. Detect elements within that region

### 4. Content Queries ("The email from John")

User refers to content:
- "Open the **email from John**"
- "Click the **link about pricing**"
- "Find the **error message**"

Resolution:
1. Parse content description
2. Use OCR + element detection to find matching text
3. Return element containing the content

## Action Types

### Click Actions
```
Click {
    target: ElementReference,
    button: Left | Right | Middle,
    click_type: Single | Double | Triple,
}
```

### Type Actions
```
Type {
    target: ElementReference,
    text: String,
    clear_first: bool,     // Clear existing text before typing
    submit: bool,          // Press Enter after typing
}
```

### Scroll Actions
```
Scroll {
    target: ElementReference,
    direction: Up | Down | Left | Right,
    amount: Lines(u32) | Page | ToEnd,
}
```

### Drag Actions
```
Drag {
    from: ElementReference,
    to: ElementReference,
    button: Left | Right,
}
```

### Window Actions
```
WindowAction {
    action: Focus | Minimize | Maximize | Close | Move | Resize,
    target: WindowReference,
    position: Option<(i32, i32)>,
    size: Option<(u32, u32)>,
}
```

## Overlay Protocol

The overlay system renders visual feedback for pointer operations:

### Cursor Companion
- Triangle that zips from cursor to target
- Shows caption at target
- Returns to cursor position
- Animation: 400ms travel, 2000ms hold, 400ms return

### Highlight Ring
- Circle around the target element
- Pulsing animation
- Color: accent color (default blue #2563EB)

### Caption Bubble
- Rounded rectangle with text
- Arrow pointing to target
- Auto-positioned to avoid screen edges
- Fade in: 200ms, Fade-out: 300ms

### Multi-Marker
- Multiple simultaneous highlights
- Auto-layout to avoid overlap
- Each marker has independent color + caption
- Auto-expire after duration or persistent until cleared

## State Machine

```
Idle → Hovering → Pointing → Captioning → Idle
  │         │          │          │
  │         │          │          └─→ Caption fades out
  │         │          └─→ Triangle zips to target, shows caption
  │         └─→ Cursor over element, highlight ring appears
  └─→ No element under cursor

Idle → Selecting → Pointing → Idle
  │         │          │
  │         │          └─→ Point at selection center
  │         └─→ User draws selection rectangle
  └─→ Selection mode activated

Idle → MultiPointing → Idle
  │         │
  │         └─→ Multiple markers shown simultaneously
  └─→ Multi-marker command received
```

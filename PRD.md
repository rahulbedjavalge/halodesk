# HaloDesk + HaloRouter PRD
**Product Requirements Document (PRD)**  
**File:** `PRD.md`  
**Version:** 1.1  
**Owner:** Rahul  
**Last updated:** 2026-02-06 (Europe/Berlin)

---

## 1. Product summary

**HaloDesk** is a floating desktop AI widget for Windows and macOS.

**HaloRouter** is a **local-only API router** (runs on the user’s machine) that:
- stores the user’s API keys securely
- routes requests to the user-selected model/provider
- provides a consistent local API to HaloDesk
- manages local memory (stored on-device in system storage)

**Important:** There is **no cloud backend** for this product.  
The only external network calls are to the user-configured model providers and only when the user sends a request.

---

## 2. What this is and what it is not

### This is
- A **floating widget** for micro tasks (rewrite, comments, corrections, summarize)
- A **local routing layer** that abstracts providers and models
- A **local memory store** the user controls (on-device)

### This is NOT (MVP)
- Not a browser controller
- Not an automation agent that clicks buttons or types into websites
- Not background screen monitoring
- Not a cloud account product

---

## 3. Problem statement

Small tasks have huge friction:
- switching tabs
- opening a web UI
- taking screenshots
- copying and pasting back and forth

The real pain is **context switching**.

---

## 4. Goals

### Primary goals
1. **Floating widget UX**: 1 hotkey, instant overlay, no workflow break.
2. **Local router**: all AI requests go through HaloRouter on `localhost`.
3. **User-owned config**: user adds their own API keys and selects models/providers.
4. **Local memory**: store history and memory on-device in system storage.
5. **Cross-platform shipping**: Windows installer and macOS app bundle.

### Secondary goals
- Model routing rules (task-based routing)
- Prompt presets
- Offline-friendly UX (still usable for non-AI actions like clipboard tools)

---

## 5. Target users and top use cases

### Personas
- Builders and founders: quick replies and rewrites all day
- Job seekers: DMs, follow-ups, grammar fixes
- Students: summarize snippets, explain quickly

### Top use cases
- “Write 3 comments to reply to this LinkedIn post”
- “Rewrite this sentence simpler”
- “Fix grammar and keep tone”
- “Summarize this screenshot”
- “Turn this into bullets”

---

## 6. Success metrics

- Hotkey → overlay visible: **< 250ms**
- Send → first tokens shown (stream): **< 1.5s** (network dependent)
- Capture screenshot → response → copy: **< 12s**
- Error-free session rate: **> 98%**
- Successful provider setup rate: **> 90%**

---

## 7. UX principles

- Invisible until needed
- Explicit permissions (screen capture only when user triggers it)
- Trust builders: preview what will be sent (especially screenshots)
- Fast refine loop: shorter, more direct, more friendly, etc.

---

## 8. System architecture

### Two-component product
1. **HaloDesk (UI)**
   - floating overlay window
   - presets, prompt box, output, copy buttons
   - NEVER talks to providers directly
   - calls local router at `http://127.0.0.1:<port>`

2. **HaloRouter (Local API service)**
   - local server bundled with app
   - stores provider keys securely
   - performs model routing and provider requests
   - manages local memory (SQLite by default)
   - exposes a clean local API to HaloDesk

---

## 9. Core user flows

### Flow A: Quick Ask (text only)
1. Hotkey opens HaloDesk overlay.
2. User selects preset or types prompt.
3. User hits Send.
4. HaloDesk sends request to HaloRouter.
5. HaloRouter selects model/provider, calls external API, streams back.
6. User copies output.

**Acceptance criteria**
- No external request happens unless user presses Send
- Streaming works if provider supports it

---

### Flow B: Ask with Screenshot (optional)
1. Hotkey opens capture mode.
2. User selects region/window.
3. Preview is shown with “Send / Cancel”.
4. Only if Send: HaloDesk passes image + prompt to HaloRouter.
5. HaloRouter routes to a vision-capable model.

**Acceptance criteria**
- Preview is required before sending
- Cancel means nothing is sent

---

### Flow C: Clipboard rewrite
1. User copies text.
2. Hotkey opens overlay and detects clipboard content.
3. User chooses “Fix grammar”, “Shorten”, etc.
4. Response streams and user copies.

---

## 10. Functional requirements (HaloDesk)

### 10.1 Overlay window
- Always-on-top toggle
- Pin/unpin
- Resize
- Close to tray
- Keyboard-first UX

### 10.2 Presets (MVP)
- LinkedIn Comment (3 variants)
- Rewrite Cleaner (1 improved + 1 shorter)
- Grammar Fix
- Summarize (text)
- Summarize Screenshot (if vision model configured)
- Translate

Each preset defines:
- system prompt
- output constraints (length, tone)
- default routing policy (text vs vision)

### 10.3 Output tools
- Copy button
- Regenerate
- Quick refine chips:
  - shorter
  - more direct
  - more friendly
  - more formal
  - add emojis (optional)

### 10.4 Settings UI
- Provider list (OpenRouter first, then more)
- Add/update/remove API keys
- Select default models for text and vision
- Routing rules editor (simple in MVP)
- Memory settings:
  - store history (ON by default)
  - clear history
  - optional export (post-MVP)

---

## 11. Functional requirements (HaloRouter)

### 11.1 Local API endpoints (MVP)
Local-only (bind to 127.0.0.1)

- `GET /health`
- `GET /v1/models`
- `POST /v1/chat`
- `POST /v1/memory/store`
- `POST /v1/memory/query` (recommended, can be MVP-lite)

### 11.2 Provider adapters
MVP providers:
- OpenRouter
- Custom HTTP endpoint (optional)

Adapter responsibilities:
- validate key exists
- format request payload
- handle streaming
- normalize output into one internal response format

### 11.3 Routing engine
Inputs:
- preset
- user-selected model override
- image present (needs vision)
- user routing rules

Outputs:
- provider + model ID + params

MVP routing:
- If image → vision default
- Else → text default
- If model fails → fallback model

### 11.4 Local memory (system storage)
Storage: SQLite (encryption optional in post-MVP)

Store:
- conversation history
- pinned snippets
- custom presets and prompts

Behavior:
- store by default
- easy clear all

### 11.5 Secure key storage
- Windows Credential Manager
- macOS Keychain

---

## 12. Non-functional requirements

### Performance
- Router start: < 2 seconds
- Memory query: < 150ms
- Minimal RAM footprint

### Reliability
- Router auto-restarts
- Friendly errors: missing key, invalid key, model missing, network

### Privacy
- Local-first
- No background capture
- Preview before sending screenshot
- “Never send screenshots” toggle

---

## 13. Permissions

### macOS
- Screen Recording permission needed for capturing other apps.
- If missing, screenshot mode disabled but text-only still works.

### Windows
- Capture works normally, handle protected content failures gracefully.

---

## 14. Packaging and distribution

### Windows
- Installer installs:
  - HaloDesk UI
  - HaloRouter background service/helper

### macOS
- `.app` bundle with a background helper for HaloRouter

---

## 15. Milestones

1. Overlay skeleton (hotkeys, tray)
2. HaloRouter local server
3. OpenRouter adapter + streaming
4. Presets + refine chips
5. Local memory (SQLite)
6. Screenshot mode + vision routing
7. Packaging (Win + Mac)

---

## 16. Definition of done (MVP)

- Runs on Windows and macOS
- Floating widget works with hotkeys
- HaloRouter runs locally and handles all AI calls
- User configures their own keys and models
- Local memory is stored on-device and can be cleared
- No browser control and no automation agent features

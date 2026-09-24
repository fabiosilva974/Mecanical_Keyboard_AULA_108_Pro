# AULA F108 Pro GUI - Architecture & Decoupled Design

## 1. System Overview

The **AULA F108 Pro GUI** (`f108-pro-gui`) is a lightweight native Linux desktop application built for configuring and controlling the AULA F108 Pro mechanical keyboard.

### Key Architectural Tenet: Unidirectional Decoupling
```
+-------------------------------------------------------------+
|                     Linux Desktop UI                        |
|   HTML5 / CSS3 / JavaScript (WebKitGTK 4.1 Native Engine)   |
|  - Dynamic 108-Key Canvas                                   |
|  - Real-Time CLI Command Composer                           |
|  - Multi-Language i18n Engine (en, pt_BR, ...)              |
|  - Customizable Themes (Purple Smoke, OLED, Cyberpunk)      |
+-------------------------------------------------------------+
                              |
                     IPC Bridge (WebKit2)
                 `window.webkit.messageHandlers`
                              v
+-------------------------------------------------------------+
|                 Python Host Bridge (app.py)                 |
|  - GTK3 Window & WebKit.WebView Container                   |
|  - Asynchronous Subprocess Invoker (`asyncio.subprocess`)   |
|  - Non-blocking Standard Output / Error Streamer            |
+-------------------------------------------------------------+
                              |
                     Standard CLI Invocation
                              v
+-------------------------------------------------------------+
|              Standalone Rust CLI Driver (`f108-pro`)        |
|  - Zero GUI Dependencies                                    |
|  - Full HID Feature & Interrupt Protocol Engine             |
|  - USB Device Filtering (`0c45:800a` / `05ac:024f`)         |
+-------------------------------------------------------------+
                              |
                        Raw USB Packets
                              v
+-------------------------------------------------------------+
|                 AULA F108 Pro Hardware                      |
|  - Interface 3: Control & Feature Reports (64B)             |
|  - Interface 2: TFT LCD Display & Frame Streaming           |
+-------------------------------------------------------------+
```

### Unidirectional Dependency Guarantee
- **The GUI depends on the CLI**: The user interface generates and executes standard CLI commands (`f108-pro light ...`, `f108-pro remap ...`, `f108-pro perkey ...`, `f108-pro clock`, `f108-pro lcd ...`).
- **The CLI NEVER depends on the GUI**: The CLI binary (`~/.local/bin/f108-pro`) is completely self-sufficient. It can be run headless, in CI/CD, in systemd timers, or from bash scripts without X11 or Wayland libraries.

---

## 2. Technology Stack Selection Rationale

| Layer | Selected Tech | Rationale |
|---|---|---|
| **Window Host** | Python 3 + PyGObject (`gi.repository.Gtk 3.0`) | Native integration with Linux desktops (GNOME, KDE, XFCE). Zero extra package installation required. |
| **Rendering Engine** | `gi.repository.WebKit2 4.1` | Ultra-crisp modern CSS rendering, hardware-accelerated SVG/PNG composite, sub-300ms startup, ~25MB memory footprint. Matches Tauri architecture without Node.js or C headers. |
| **Frontend Core** | Vanilla HTML5 / CSS3 / ES6 Modules | Zero external build tools (no webpack, no npm), instant startup, easy customization, maintainable by subagents. |
| **Hardware Driver** | Compiled Rust Binary (`f108-pro`) | High-speed USB communication, strict type safety, zero segfaults, deterministic 35ms packet delays. |

---

## 3. Communication Protocol (IPC Bridge)

The frontend communicates with the host operating system via WebKitGTK's native `UserContentManager` message handlers.

### Frontend Dispatch (`ui/js/app.js`):
```javascript
// Generic bridge invocation
window.hostBridge.invoke("execute_cli", {
    command: "f108-pro light Breath 5 3 255 0 0"
}).then(response => {
    console.log("CLI Result:", response.stdout);
});
```

### Host Handler (`app.py`):
```python
def on_script_message(user_content_manager, javascript_result):
    payload = json.loads(javascript_result.to_json())
    action = payload.get("action")
    params = payload.get("params", {})
    if action == "execute_cli":
        run_cli_async(params.get("command"))
```

---

## 4. Directory Layout

```
f108-pro-gui/
├── app.py                      # Main native GTK3 + WebKit2 window & IPC host
├── f108-pro-gui.desktop        # Linux desktop environment application shortcut
├── assets/                     # Official visual assets extracted from firmware
│   ├── keyboard/               # 800x300 official 3D layout & key_coordinates.json
│   ├── icons/                  # High-res SVG and PNG UI icons
│   ├── themes/                 # Background wallpapers (home_bkg, cyberpunk, etc.)
│   └── logo/                   # AULA brand logos
├── ui/                         # WebKit frontend application
│   ├── index.html              # Main application shell
│   ├── locales/                # Internationalization dictionaries (en.json, pt_BR.json)
│   ├── styles/                 # Modular CSS stylesheets
│   │   ├── main.css            # Base layouts, flexboxes, typography
│   │   ├── keyboard.css        # Matrix positioning, keycaps, LED glow animations
│   │   ├── terminal.css        # Collapsible CLI bottom composer styling
│   │   └── themes.css          # Theme definitions & color palettes
│   └── js/                     # Modular frontend logic
│       ├── i18n.js             # Client-side dynamic translation engine
│       ├── keyboard_canvas.js  # 108-key interactive matrix renderer
│       ├── cli_composer.js     # Real-time CLI syntax generator & executor
│       ├── tab_remap.js        # Physical key remapping module
│       ├── tab_lighting.js     # Global RGB lighting effects controller
│       ├── tab_perkey.js       # Individual per-key RGB canvas painter
│       ├── tab_lcd.js          # TFT screen image previewer & clock sync
│       └── tab_settings.js     # Theme selector & preferences
├── docs/                       # Technical documentation for all modules
│   ├── 01_architecture.md      # This document
│   ├── 02_keyboard_canvas.md   # Key matrix layout & coordinate engine
│   ├── 03_cli_console.md       # Command composer & terminal integration
│   ├── 04_remap_layers.md      # Key remapping protocol & YAML serialization
│   ├── 05_lighting_modes.md    # 20 RGB modes & color parameters
│   ├── 06_perkey_matrix.md     # Per-key LED table protocol & presets
│   ├── 07_lcd_tft_protocol.md  # 240x135 frame conversion & clock sync
│   └── 08_i18n_guide.md        # Guide for adding new translations
└── tests/                      # Verification test suites
    └── test_coordinates.py     # Validates coordinate integrity against CLI keys
```

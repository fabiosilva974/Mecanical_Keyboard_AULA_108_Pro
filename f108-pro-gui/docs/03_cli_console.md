# AULA F108 Pro GUI - Real-Time CLI Command Composer & Console

## 1. Concept & User Experience

A core requirement of the application is **visibility and empowerment**:
- Every time the user adjusts a slider, clicks a color preset, selects a lighting mode, rebinds a key, or requests a clock sync, the exact CLI command that produces that action is generated in real-time.
- The user can view the command, learn the CLI syntax, copy the command with a single click, or execute it directly on the hardware.
- The bottom console can be freely toggled / collapsed with an arrow button or keyboard shortcut (`Ctrl+\``).

```
+-----------------------------------------------------------------------------------+
|  [v] CLI Command Composer & Terminal                                              |
|                                                                                   |
|  Generated: [ f108-pro light Breath 5 3 255 0 0                             ]     |
|             [ Copy Command ]  [ Run in Terminal ]  [ Clear Logs ]                 |
|                                                                                   |
|  > [INFO] Executing: f108-pro light Breath 5 3 255 0 0                            |
|  > [USB] Opened device: 0c45:800a (Interface 3 claimed)                           |
|  > [OK] Lighting mode applied successfully in 38ms!                               |
+-----------------------------------------------------------------------------------+
```

---

## 2. Dynamic Command Mapping Reference

| UI Action | Generated CLI Command | Notes |
|---|---|---|
| Change Lighting Mode | `f108-pro light <Mode> <Brightness> <Speed> <R> <G> <B>` | e.g. `f108-pro light Colourful 5 3 0 0 0` |
| Adjust Brightness | `f108-pro brightness <0..5>` | Quick brightness adjustment |
| Turn Off RGB | `f108-pro off` | Turns off LEDs completely |
| Sync RTC Clock | `f108-pro clock` | Sinks system time to hardware LCD |
| Remap Single Key | `f108-pro remap <SOURCE_KEY> <TARGET_KEY>` | e.g. `f108-pro remap capslock lctrl` |
| Remap FN Layer Key | `f108-pro remap --fn <SOURCE_KEY> <TARGET_KEY>` | e.g. `f108-pro remap --fn w up` |
| Key Shortcut (Modifiers) | `f108-pro remap <SOURCE_KEY> <TARGET_KEY> --ctrl --shift` | e.g. `f108-pro remap f1 c --ctrl` |
| Multimedia Action | `f108-pro remap <SOURCE_KEY> --consumer <ACTION>` | e.g. `f108-pro remap f12 --consumer vol_up` |
| Mouse Action | `f108-pro remap <SOURCE_KEY> --mouse <ACTION>` | e.g. `f108-pro remap pause --mouse left_click` |
| Apply Per-Key RGB File | `f108-pro perkey <FILE.yaml>` | Sends 576-byte LED matrix buffer |
| Flash LCD Display | `f108-pro lcd <FILE.bin>` | Streams 4096-byte frames to Interface 2 |

---

## 3. Subprocess Execution & Output Streaming

When the user clicks **Run in Terminal** (or when "Auto-Apply" is enabled in settings):

1. The frontend calls `window.hostBridge.invoke("execute_cli", { command: "f108-pro ..." })`.
2. Python's `asyncio.create_subprocess_shell` handles the execution asynchronously without blocking the GTK UI loop.
3. Standard output (`stdout`) and standard error (`stderr`) lines are streamed back to the frontend console in real time via WebKit's `evaluate_javascript`.
4. Color formatting (ANSI / status badges: `[OK]`, `[ERROR]`, `[INFO]`) is rendered cleanly in the console window.

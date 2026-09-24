# AULA F108 Pro GUI - Key Remap & Multi-Layer Protocol

## 1. Overview
The AULA F108 Pro keyboard controller (Sonix) maintains two independent 576-byte remap tables in hardware:
- **Base / Normal Layer**: Activated by default during regular typing. Initialized with opcode `0x04 0x11`.
- **FN Layer**: Activated when holding or toggling the `Fn` modifier key. Initialized with opcode `0x04 0x27`.

## 2. Remap Action Types
Every key slot in the 144-slot matrix table consists of 4 bytes: `[action, param1, param2, param3]`.

| Action Type | Hex Code | Description | CLI Syntax |
|---|---|---|---|
| **Key Swap** | `0x02` | Standard HID usage replacement | `f108-pro remap <SOURCE> <TARGET>` |
| **Combo / Shortcut** | `0x02` | HID key with modifier bitmask | `f108-pro remap <SOURCE> <TARGET> --ctrl --shift` |
| **Consumer Control** | `0x03` | Multimedia & system actions | `f108-pro remap <SOURCE> --consumer <ACTION>` |
| **Mouse Emulation** | `0x07` | Click, middle click, or scroll | `f108-pro remap <SOURCE> --mouse <ACTION>` |
| **Disabled** | `0x00` | Disables keystroke on hardware | `f108-pro remap <SOURCE> none` |

## 3. FN Layer Syntax
To target the FN layer, append the `--fn` flag:
```bash
f108-pro remap --fn capslock lctrl
f108-pro remap --fn f1 --consumer mute
```

## 4. YAML Profile Format
Profiles can be exported and imported directly in the GUI:
```yaml
# f108_remap_profile.yaml
layer: base # or fn
mappings:
  capslock: lctrl
  f1: "media:mute"
  pause: "mouse:left_click"
```

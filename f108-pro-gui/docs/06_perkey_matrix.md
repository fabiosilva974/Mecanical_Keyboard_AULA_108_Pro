# AULA F108 Pro GUI - Per-Key RGB Matrix Engine

## 1. Overview
Individual key backlighting (Per-Key RGB) allows setting unique 24-bit colors for each of the 108 LED positions.

## 2. Interactive Canvas Painting
In the GUI's **Per-Key RGB** tab:
1. Select an active brush color from the palette or color picker.
2. Click or drag the mouse across keycaps on the virtual keyboard to paint them in real time.
3. Use zone presets for instant configurations:
   - **WASD + Arrows**: Red gamer cluster with cyan navigation arrows.
   - **Numpad**: Distinct green lighting for numeric keypad.
   - **Rainbow**: Multi-spectral wave across the 21 matrix columns.

## 3. CLI & YAML Formats
The GUI automatically converts the active paint map into CLI arguments or a standard YAML file:

### Inline Quartet Format:
```bash
f108-pro perkey W 255 0 0 A 255 0 0 S 255 0 0 D 255 0 0
```

### Profile YAML Format:
```yaml
# perkey_gamer.yaml
brightness: 5
all: [0, 0, 0]
keys:
  W: [255, 0, 0]
  A: [255, 0, 0]
  S: [255, 0, 0]
  D: [255, 0, 0]
  UP: [0, 240, 255]
  DOWN: [0, 240, 255]
  LEFT: [0, 240, 255]
  RIGHT: [0, 240, 255]
```
Invocation:
```bash
f108-pro perkey perkey_gamer.yaml
```

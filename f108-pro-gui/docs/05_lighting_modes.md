# AULA F108 Pro GUI - Global Lighting Effects (20 Modes)

## 1. Overview
The AULA F108 Pro provides 20 hardware-accelerated RGB lighting animations managed by the onboard MCU.

## 2. Modes Enumeration

| Mode ID | Mode Name | Description |
|---|---|---|
| `0` | **Off** | LEDs turned off completely (`f108-pro off`) |
| `1` | **Static** | Solid static color |
| `2` | **SingleOn** | Key lights up upon press, then fades |
| `3` | **SingleOff** | Key extinguishes upon press, then lights back up |
| `4` | **Glittering** | Twinkling stars across the keyboard |
| `5` | **Falling** | Matrix raindrop effect |
| `6` | **Colourful** | Rainbow wave across rows |
| `7` | **Breath** | Gentle breathing pulse in selected color |
| `8` | **Spectrum** | Smooth rainbow spectrum cycling |
| `9` | **Outward** | Expanding circular ripples from keystrokes |
| `10` | **Scrolling** | Horizontal wave scrolling |
| `11` | **Rolling** | Diagonal rainbow rolling |
| `12` | **Rotating** | Circular vortex rotation |
| `13` | **Explode** | Concentric explosion upon key press |
| `14` | **Launch** | Vertical laser beam shooting up from key |
| `15` | **Ripples** | Water ripple waves across key rows |
| `16` | **Flowing** | Flowing rainbow stream |
| `17` | **Pulsating** | Heartbeat rhythmic pulse |
| `18` | **Tilt** | Cascading diagonal wash |
| `19` | **Shuttle** | High-speed tracer scanning across rows |

## 3. CLI Command Signature
```bash
f108-pro light <MODE> <BRIGHTNESS: 0..5> <SPEED: 1..5> <R: 0..255> <G: 0..255> <B: 0..255>
```
Example:
```bash
f108-pro light Breath 5 3 147 51 234
```

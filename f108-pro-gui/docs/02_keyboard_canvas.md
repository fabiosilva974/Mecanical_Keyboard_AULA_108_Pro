# AULA F108 Pro GUI - Keyboard Matrix & Canvas Engine

## 1. Overview

The interactive keyboard canvas provides a 1:1 visual replica of the AULA F108 Pro mechanical keyboard. It overlays 104 interactive, clickable key buttons on top of the official 800x300 3D rendered keyboard image (`img_keyboard_layout.png`).

```
+---------------------------------------------------------------------------------+
| 800x300 Coordinate Space                                                        |
|                                                                                 |
|  [Esc]      [F1][F2][F3][F4]  [F5][F6][F7][F8]  [F9][F10][F11][F12] [Prt][Scr][Pause] [TFT Screen]
|                                                                                 |
|  [`][1][2][3][4][5][6][7][8][9][0][-][=][ Backspace ]  [Ins][Home][PgUp] [Num][/][*][-]
|  [Tab ][Q][W][E][R][T][Y][U][I][O][P][{][}][   \    ]  [Del][End ][PgDn] [ 7 ][8][9][+]
|  [Caps ][A][S][D][F][G][H][J][K][L][;]['][  Enter   ]                   [ 4 ][5][6] |
|  [Shift  ][Z][X][C][V][B][N][M][,][.][/][ Shift   ]        [ ↑ ]       [ 1 ][2][3][E]
|  [Ctrl][Win][Alt][      Space       ][Alt][Fn][App][Ctrl]  [←][ ↓ ][→]  [   0  ][.][n]
+---------------------------------------------------------------------------------+
```

---

## 2. Coordinate Extraction & Data Schema

Key coordinates are derived directly from the official firmware layout file (`layouts/rgb-keyboard.xml`) and saved into `assets/keyboard/key_coordinates.json`.

### Schema Definition:
```json
{
  "canvas": {
    "width": 800,
    "height": 300,
    "image": "img_keyboard_layout.png"
  },
  "keys": [
    {
      "id": "ESC",
      "cli_name": "ESC",
      "label": "Esc",
      "desc": "Esc",
      "code": "0x29",
      "key_index": 1,
      "light_index": 1,
      "rect": {
        "x": 27,
        "y": 52,
        "w": 26,
        "h": 26
      },
      "row_col": "0#0",
      "fnlayer_disable": 0
    }
  ]
}
```

### Attribute Descriptions:
- `id` / `cli_name`: The canonical key identifier expected by the `f108-pro` CLI commands (e.g. `f108-pro remap capslock lctrl`).
- `label`: Visual label rendered inside the keycap element.
- `code`: USB HID usage ID in hexadecimal format (`0x29` = ESC).
- `key_index`: Physical switch position in the hardware matrix (1..=132).
- `light_index`: Physical LED address in the RGB lighting controller.
- `rect`: Absolute pixel bounds `(x, y, w, h)` within the 800x300 layout.
- `row_col`: Hardware scan matrix row and column (`row#col`).
- `fnlayer_disable`: Flag indicating whether the key can be assigned on the Fn layer.

---

## 3. Rendering Engine (`ui/js/keyboard_canvas.js`)

The canvas container is styled with `position: relative; width: 800px; height: 300px;`.
Each key is instantiated as a `<button class="keycap">` element:

```javascript
export function renderKeyboard(container, keysData, onKeyClick) {
    container.innerHTML = '';
    
    // Background keyboard graphic
    const bg = document.createElement('img');
    bg.src = '../assets/keyboard/img_keyboard_layout.png';
    bg.className = 'keyboard-bg';
    container.appendChild(bg);

    // Render interactive keys
    keysData.keys.forEach(k => {
        const btn = document.createElement('button');
        btn.className = 'keycap';
        btn.dataset.keyId = k.id;
        btn.dataset.lightIndex = k.light_index;
        btn.style.left = `${k.rect.x}px`;
        btn.style.top = `${k.rect.y}px`;
        btn.style.width = `${k.rect.w}px`;
        btn.style.height = `${k.rect.h}px`;
        
        btn.innerHTML = `<span class="key-label">${k.label}</span><span class="key-led"></span>`;
        btn.addEventListener('click', (e) => onKeyClick(k, btn, e));
        container.appendChild(btn);
    });
}
```

---

## 4. Key Visual States

Keys dynamically receive CSS classes based on user interaction:
- `.selected`: Currently focused key for remapping.
- `.has-remap`: Indicates key has a non-default custom mapping applied.
- `.led-glow`: Active simulated RGB backlighting in the Per-Key tab.
- `.hover`: Cursor hovering state displaying tooltip with key metadata.

# Protocolo de Remapeamento de Teclas e Camadas (AULA F108 Pro)

Este documento descreve o protocolo de remapeamento de teclas (Key Remap), suporte a camadas (Layer normal e FN), códigos USB HID, controles de consumidor (Consumer), comandos de mouse e macros de combinação.

---

## 1. Arquitetura de Camadas e Mapeamento

O teclado AULA F108 Pro permite reatribuir qualquer tecla física (`KEY_ESC` a `KEY_CALCULATOR`, índices 1 a 132) para comportar-se como:
- Outra tecla padrão do teclado (via **USB HID Usage ID**, página `0x07`).
- Tecla modificadora (`LCTRL`, `LSHIFT`, `LALT`, `LWIN`, etc.).
- Atalho multimídia ou de sistema (**Consumer Control**, ex: Mute, Volume Up/Down, Play/Pause, Calculadora).
- Emulação de clique de mouse ou rolagem (`MOUSE_LEFT`, `MOUSE_RIGHT`, `WHEEL_UP`, `WHEEL_DOWN`).

---

## 2. Estrutura do Pacote de Remapeamento

Cada regra de remapeamento é codificada em estruturas de 4 a 8 bytes enviadas via pacotes de controle:

- **Byte 0**: Índice físico da tecla origem (`1` a `132`).
- **Byte 1**: Tipo de Ação (`0x01` = Tecla HID padrão, `0x02` = Consumer Control, `0x03` = Mouse Emulation, `0x04` = Macro / Combo).
- **Bytes 2-3**: Código HID ou parâmetro da ação.
- **Bytes 4-7**: Modificadores associados (bitmask de Shift, Ctrl, Alt, Win).

---

## 3. Funções Auxiliares de Tradução

O módulo `constants.rs` fornece mapeamentos robustos para conversão:
- `key_name_to_index(name)`: Converte string legível (ex: `"F1"`, `"SPACE"`) para índice físico.
- `key_name_to_hid(name)`: Converte para o Usage ID HID padrão (ex: `"A"` -> `0x04`).
- `consumer_name_to_code(name)`: Mapeia mídias e controles (ex: `"VOLUMEUP"` -> `0xE9`).
- `mouse_name_to_params(name)`: Retorna os parâmetros de botão e delta para emulação de mouse.
- `is_modifier_key(code)` / `hid_to_modifier_bit(code)`: Gerenciam modificadores de teclado.

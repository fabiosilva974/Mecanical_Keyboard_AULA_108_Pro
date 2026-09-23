# Transporte USB e Gerenciamento de Interfaces (AULA F108 Pro)

Este documento detalha o transporte USB, o gerenciamento de interfaces, o desanexação de drivers do kernel (kernel driver detach) e os fallbacks de conexão entre o modo com fio (Wired) e o dongle sem fio 2.4GHz (Wireless).

---

## 1. Identificadores USB (VID / PID)

O teclado AULA F108 Pro opera em dois modos principais de conexão reconhecidos pelo subsistema USB:

- **Modo Com fio (Wired)**:
  - `VENDOR_ID_WIRED`: `0x0C45` (Sonix Technology Co., Ltd. - detectado frequentemente pelo kernel Linux como Vivitar Vivicam).
  - `PRODUCT_ID_WIRED`: `0x800A` (Teclado mecânico AULA F108 Pro).
- **Modo Sem Fio (Wireless 2.4GHz Dongle)**:
  - `VENDOR_ID_WIRELESS`: `0x05AC`
  - `PRODUCT_ID_WIRELESS: u16 = 0x024F`

---

## 2. Mapeamento de Interfaces USB HID

O dispositivo apresenta múltiplas interfaces USB para separar as funções de teclado padrão, controle customizado e streaming de LCD:

```mermaid
graph LR
    Device[AULA F108 Pro USB Device] --> IF0[Interface 0: Boot Keyboard]
    Device --> IF1[Interface 1: Consumer Control / System]
    Device --> IF2[Interface 2: LCD TFT Streaming (EP3 OUT / EP4 IN)]
    Device --> IF3[Interface 3: Control & Feature Reports (64 bytes)]
```

- **Interface 3 (`INTERFACE_CONTROL = 3`)**: Utilizada para enviar pacotes de configuração de 64 bytes (`REPORT_SIZE = 64`) via requisições USB HID `SET_REPORT` (`0x09`) e `GET_REPORT` (`0x01`).
- **Interface 2 (`INTERFACE_LCD = 2`)**: Dedicada ao streaming de imagens para o display TFT colorido. Utiliza transferência por interrupção:
  - `LCD_OUT_EP`: Endpoint `3` (OUT) para envio de páginas de 4096 bytes (`LCD_PAGE_SIZE`).
  - `LCD_IN_EP`: Endpoint `4` (IN) para recepção de ACKs e confirmações de quadro pelo firmware.

---

## 3. Kernel Driver Detach e Gerenciamento de Conflitos

No Linux, o kernel (módulos `usbhid` ou `hid_generic`) reivindica automaticamente as interfaces USB ao conectar o teclado. Para que o driver userspace em Rust (`rusb`) possa enviar comandos de controle e dados de LCD sem interferência:

1. O driver verifica se o kernel driver está ativo na interface (`libusb_kernel_driver_active`).
2. Caso esteja ativo, executa a desanexação (`libusb_detach_kernel_driver`).
3. Ao encerrar a sessão, opcionalmente reanexa o driver do kernel para restaurar o funcionamento padrão de teclado HID do sistema operacional.

---

## 4. Fallbacks de Conexão (Cabo vs Dongle)

O gerenciador de transporte implementa uma estratégia de descoberta automática:
- Tenta primariamente abrir o dispositivo usando `VENDOR_ID_WIRED` (`0x0C45`) e `PRODUCT_ID_WIRED` (`0x800A`).
- Se não encontrado, verifica a presença do dongle sem fio (`VENDOR_ID_WIRELESS`, `PRODUCT_ID_WIRELESS`).
- Retorna um erro amigável caso nenhum dos dispositivos seja detectado nas portas USB ativas.

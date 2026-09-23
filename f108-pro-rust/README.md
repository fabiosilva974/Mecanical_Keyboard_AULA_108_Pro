# f108-pro-rust

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux-lightgrey.svg)](https://www.kernel.org/)

**f108-pro-rust** é um driver em espaço de usuário (*userspace*) e interface de linha de comando (CLI) desenvolvidos em **Rust** para o controle total do teclado mecânico **AULA F108 Pro** (equipado com display LCD TFT colorido e controladora Sonix/Microdia).

---

## Destaques Arquiteturais

- **Comunicação USB em Espaço de Usuário (`rusb` / libusb 1.0):** Comunicação direta de baixo nível via endpoints HID e Bulk/Interrupt, sem necessidade de drivers de kernel personalizados.
- **Auto-Detach de Driver de Kernel:** O driver gerencia automaticamente a desativação temporal (`detach_kernel_driver`) de controladores HID padrão do kernel Linux para evitar conflitos de acesso.
- **Controle Rígido de Timing (35ms):** Implementa um intervalo obrigatório de **35ms** entre transações consecutivas de comandos via Feature Reports para prevenir estouros de buffer (*buffer overflow*) ou travamentos na controladora Sonix.
- **Gerenciamento de Múltiplas Interfaces USB:**
  - **Interface 3:** Controle de iluminação global, iluminação por tecla (`perkey`), remapeamento (`remap`) e sincronização de relógio via Feature Reports de 64 bytes.
  - **Interface 2:** Streaming de páginas de 4 KB (4096 bytes) para o display LCD TFT via Endpoint Interrupt OUT (EP 3) com confirmação (ACK) no Endpoint Interrupt IN (EP 4).

---

## Requisitos do Sistema

Antes de compilar e executar o projeto em sistemas Linux, certifique-se de instalar as dependências de desenvolvimento do `libusb`:

### 1. Dependências de Pacotes (Ubuntu / Debian / Arch Linux)
```bash
# Ubuntu / Debian
sudo apt update && sudo apt install -y build-essential pkg-config libusb-1.0-0-dev libudev-dev

# Arch Linux
sudo pacman -S base-devel libusb pkgconf
```

### 2. Configuração de Regras Udev (Permissões de Acesso USB)
Para executar a CLI sem privilégios de `root` (`sudo`), configure uma regra udev para o teclado AULA F108 Pro (`VID:PID = 0C45:800A`):

Crie o arquivo `/etc/udev/rules.d/99-aula-f108.rules`:
```text
SUBSYSTEM=="usb", ATTRS{idVendor}=="0c45", ATTRS{idProduct}=="800a", MODE="0666", TAG+="uaccess"
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="0c45", ATTRS{idProduct}=="800a", MODE="0666", TAG+="uaccess"
```

Recarregue as regras do udev:
```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

---

## Compilação e Instalação

Clone o repositório e compile a versão otimizada de produção (`--release`):

```bash
git clone https://github.com/seu-usuario/f108-pro-rust.git
cd f108-pro-rust

# Compilar com cargo
cargo build --release

# O binário compilado estará em target/release/f108-pro
```

Para instalar globalmente no sistema (opcional):
```bash
sudo cp target/release/f108-pro /usr/local/bin/
```

---

## Guia Rápido de Uso

Abaixo estão alguns exemplos práticos dos principais comandos da CLI `f108-pro`:

```bash
# 1. Listar todos os 20 modos de iluminação disponíveis
f108-pro modes

# 2. Ativar o efeito de respiração (Breath) em vermelho (brilho 5, velocidade 3)
f108-pro light Breath 5 3 255 0 0

# 3. Ativar o efeito multicolorido Spectrum
f108-pro light Spectrum 5 3 colorful

# 4. Ajustar rapidamente o brilho global para o nível 3
f108-pro brightness 3

# 5. Desligar toda a iluminação RGB
f108-pro off

# 6. Sincronizar o relógio do display LCD com o horário do sistema operacional
f108-pro clock

# 7. Exibir o mapa de nomes válidos de teclas
f108-pro keys

# 8. Enviar animação customizada para o display LCD (limite de 141 frames)
f108-pro lcd custom_anim.bin
```

Para consultar a documentação completa e avançada (incluindo iluminação por tecla e remapeamento via YAML), consulte o [Manual da CLI](07_cli_usage.md).

---

## Documentação Técnica do Projeto (`docs/`)

O projeto possui uma documentação detalhada dividida em módulos na pasta `docs/`:

- [`01_architecture.md`](docs/01_architecture.md): Arquitetura geral, USB descriptors e camadas de abstração.
- [`02_usb_transport.md`](docs/02_usb_transport.md): Camada de transporte USB, gerenciamento de interfaces e controle de timing (35ms).
- [`03_lighting_subsystem.md`](docs/03_lighting_subsystem.md): Protocolo de iluminação global e os 20 modos de efeitos.
- [`04_lcd_subsystem.md`](docs/04_lcd_subsystem.md): Protocolo de streaming de frames TFT LCD e limites de buffer.
- [`05_perkey_subsystem.md`](docs/05_perkey_subsystem.md): Mapeamento de cores RGB individuais por tecla e arquivos YAML.
- [`06_remap_subsystem.md`](docs/06_remap_subsystem.md): Remapeamento de scancodes, camada FN e mídias.
- [`07_cli_usage.md`](docs/07_cli_usage.md): Manual completo de uso da CLI com exemplos práticos.

---

## Licença

Distribuído sob os termos das licenças MIT e Apache 2.0. Consulte [LICENSE-MIT](LICENSE-MIT) e [LICENSE-APACHE](LICENSE-APACHE) para obter mais detalhes.

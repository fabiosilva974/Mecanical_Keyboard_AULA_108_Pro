# Manual da CLI `f108-pro`

Este documento fornece o manual completo de uso da interface de linha de comando (CLI) **`f108-pro`** para o controle avançado do teclado mecânico **AULA F108 Pro** em sistemas Linux.

---

## Sumário
1. [Visão Geral e Sintaxe Geral](#visão-geral-e-sintaxe-geral)
2. [Subcomandos de Iluminação Global (`light`, `brightness`, `off`, `modes`)](#subcomandos-de-iluminação-global)
3. [Sincronização de Relógio (`clock`)](#sincronização-de-relógio-clock)
4. [Mapeamento de Teclas (`keys`)](#mapeamento-de-teclas-keys)
5. [Iluminação Por Tecla (`perkey`)](#iluminação-por-tecla-perkey)
6. [Remapeamento de Teclas (`remap`)](#remapeamento-de-teclas-remap)
7. [Gerenciamento do Display LCD (`lcd`)](#gerenciamento-do-display-lcd-lcd)
8. [Ferramentas Auxiliares (`mkimage`, `dumphid`)](#ferramentas-auxiliares)

---

## 1. Visão Geral e Sintaxe Geral

A ferramenta CLI `f108-pro` interage diretamente com o teclado AULA F108 Pro utilizando comunicação USB em espaço de usuário via `rusb` (libusb 1.0). O acesso requer permissões adequadas nas interfaces HID (Interface 3 para comandos e Interface 2 para streaming LCD).

### Sintaxe Básica
```bash
f108-pro [OPÇÕES] <SUBCOMANDO> [ARGUMENTOS]
```

### Opções Globais
- `-v, --verbose`: Exibe logs detalhados de comunicação USB e pacotes transmitidos.
- `-d, --device <VID:PID>`: Especifica manualmente o Vendor ID e Product ID (padrão: `0C45:800A` para modo com fio).
- `-h, --help`: Exibe a ajuda detalhada.

---

## 2. Subcomandos de Iluminação Global

O subsistema de iluminação global permite controlar os 20 efeitos predefinidos na controladora Sonix/Microdia, ajustando brilho, velocidade, direção e cores RGB.

### `light`
Configura um efeito de iluminação específico com parâmetros de brilho, velocidade e cor.

**Sintaxe:**
```bash
f108-pro light <MODO> <BRILHO> <VELOCIDADE> [RED GREEN BLUE | colorful]
```
- `<BRILHO>`: Nível de 0 a 5.
- `<VELOCIDADE>`: Nível de 0 a 5.
- Modo Arco-íris/Multicolorido: passe a palavra `colorful` ou forneça os canais RGB (0 a 255).

**Exemplos Práticos:**
```bash
# Efeito de respiração (Breath) em vermelho forte, brilho 5, velocidade 3
f108-pro light Breath 5 3 255 0 0

# Efeito espectro multicolorido (Spectrum), brilho 5, velocidade 3
f108-pro light Spectrum 5 3 colorful

# Efeito estático azul sólido
f108-pro light Static 4 3 0 0 255
```

### Lista Completa dos 20 Modos de Iluminação (`modes`)
O subcomando `modes` lista todos os efeitos suportados pelo firmware:

```bash
f108-pro modes
```

| ID | Nome | Descrição |
|----|------|-----------|
| `0` | `Off` | Iluminação completamente desligada |
| `1` | `Static` | Cor estática em todas as teclas |
| `2` | `SingleOn` | Acende a tecla pressionada e apaga gradualmente |
| `3` | `SingleOff` | Apaga a tecla pressionada momentaneamente |
| `4` | `Glittering` | Efeito cintilante / poeira de estrelas |
| `5` | `Falling` | Efeito de cascata / chuva vertical |
| `6` | `Colourful` | Onda RGB multicolorida contínua |
| `7` | `Breath` | Efeito de respiração (fade in/out) |
| `8` | `Spectrum` | Transição suave de espectro de cores |
| `9` | `Outward` | Onda expansiva a partir do centro |
| `10` | `Scrolling` | Barras de cores deslizantes horizontais |
| `11` | `Rolling` | Efeito rotativo linear |
| `12` | `Rotating` | Rotação circular de cores |
| `13` | `Explode` | Explosão de luz a partir da tecla digitada |
| `14` | `Launch` | Efeito de disparo de foguete |
| `15` | `Ripples` | Ondulações concêntricas ao pressionar teclas |
| `16` | `Flowing` | Fluxo contínuo de ondas RGB |
| `17` | `Pulsating` | Pulsação rítmica de intensidade |
| `18` | `Tilt` | Inclinação diagonal de cores |
| `19` | `Shuttle` | Efeito de vaivém (shuttle) lateral |

### `brightness`
Ajusta rapidamente o nível de brilho global sem alterar o modo atual.

**Sintaxe:**
```bash
f108-pro brightness <0-5>
```
**Exemplo:**
```bash
f108-pro brightness 3
```

### `off`
Desliga instantaneamente toda a iluminação RGB do teclado.

**Exemplo:**
```bash
f108-pro off
```

---

## 3. Sincronização de Relógio (`clock`)

O teclado AULA F108 Pro possui um display LCD capaz de exibir um relógio em tempo real. O subcomando `clock` captura o horário atual do sistema operacional e o sincroniza com a controladora do teclado.

**Sintaxe:**
```bash
f108-pro clock
```

**Exemplo:**
```bash
f108-pro clock
```
*Saída esperada:*
```text
[INFO] Obtendo horário do sistema... (2026-09-23 11:53:35)
[INFO] Enviando pacote de sincronização de relógio via Feature Report...
[SUCESSO] Relógio do display LCD sincronizado com sucesso!
```

---

## 4. Mapeamento de Teclas (`keys`)

Para auxiliar na configuração de iluminação por tecla (`perkey`) e remapeamento (`remap`), o subcomando `keys` exibe a tabela de índices físicos e nomes válidos de teclas do layout 108 teclas.

**Sintaxe:**
```bash
f108-pro keys
```

---

## 5. Iluminação Por Tecla (`perkey`)

Permite definir cores RGB personalizadas para teclas individuais ou aplicar uma cor padrão para todas as teclas com exceções específicas.

### Modo Linha de Comando (CLI)
**Exemplo:**
```bash
# Define fundo cinza escuro para todas as teclas, mas 'w' e 'a' em verde brilhante
f108-pro perkey --all 0 0 50 w 0 255 0 a 0 255 0
```

### Via Arquivo YAML
É possível definir layouts complexos por tecla através de um arquivo de configuração YAML.

**Exemplo de `layout.yaml`:**
```yaml
default:
  r: 10
  g: 10
  b: 30
keys:
  esc: [255, 0, 0]
  space: [0, 255, 255]
  w: [0, 255, 0]
  a: [0, 255, 0]
  s: [0, 255, 0]
  d: [0, 255, 0]
```

**Comando:**
```bash
f108-pro perkey layout.yaml
```

---

## 6. Remapeamento de Teclas (`remap`)

O subsistema de remap permite alterar a função de teclas individuais, adicionar atalhos multimídia ou remapear a camada FN.

### Exemplos na Linha de Comando
```bash
# Trocar CapsLock por Control Esquerdo
f108-pro remap capslock lctrl

# Remapear a combinação FN + F1 para Play/Pause multimídia
f108-pro remap --fn f1 media:play
```

### Via Arquivo YAML (`config.yaml`)
```yaml
mappings:
  - from: capslock
    to: lctrl
  - from: ralt
    to: menu
fn_mappings:
  - from: f1
    to: media:play
  - from: f2
    to: media:volume_down
  - from: f3
    to: media:volume_up
```

**Comando:**
```bash
f108-pro remap config.yaml
```

---

## 7. Gerenciamento do Display LCD (`lcd`)

O AULA F108 Pro possui um display TFT LCD colorido na Interface USB 2. O subcomando `lcd` transmite arquivos de animação binária customizados para a controladora.

### Sintaxe
```bash
f108-pro lcd <ARQUIVO_ANIMACAO.bin>
```

### Orientações de Segurança e Limites de Hardware
> [!IMPORTANT]
> - **Limite Estrito de Frames:** O firmware do display possui buffer e capacidade limitadas a **no máximo 141 frames** por animação.
> - **Alinhamento de Páginas:** Os pacotes de transferência utilizam blocos e páginas de **4096 bytes** (4 KB). O arquivo binário deve estar corretamente formatado com o cabeçalho esperado pelo protocolo.
> - **Confirmação de Endpoint:** A transmissão ocorre via Endpoint Interrupt OUT (EP 3), aguardando confirmação (ACK) no Endpoint Interrupt IN (EP 4). Certifique-se de não interromper o processo de gravação para evitar corromper a flash do display.

**Exemplo:**
```bash
f108-pro lcd custom_anim.bin
```

---

## 8. Ferramentas Auxiliares

### `mkimage`
Ferramenta utilitária para conversão de imagens estáticas (PNG/JPEG) ou arquivos GIF animados no formato binário (`.bin`) compatível com o protocolo do LCD do F108 Pro, além de geração de cores sólidas.

**Exemplo:**
```bash
# Converter um GIF animado em binário otimizado para o LCD (respeitando o limite de 141 frames)
f108-pro mkimage --input animation.gif --output custom_anim.bin
```

### `dumphid`
Utilitário de diagnóstico que inspeciona os descritores HID, interfaces USB e relatórios de controle disponíveis no teclado. Ideal para depuração de conectividade e validação de permissões de udev.

**Exemplo:**
```bash
f108-pro dumphid
```

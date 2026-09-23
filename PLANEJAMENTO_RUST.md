# Planejamento: Driver e Ferramenta Aula F108 Pro em Rust (`f108-pro-rust`)

Este documento descreve o planejamento técnico para recriar o driver e utilitário de configuração do teclado mecânico **Aula F108 Pro** em **Rust** dentro de um novo diretório `f108-pro-rust`, substituindo a implementação atual em Go.

---

## 1. Diretrizes Especiais Solicitadas pelo Usuário

1. **Comentários Extensivos no Código-Fonte**:
   - Cada arquivo e função gerada deverá conter comentários explicando detalhadamente a lógica utilizada.
   - Detalhar a razão dos valores mágicos (ex: `0x55AA`, `0x5A`, `0x04 0x18`), tempos de delay (35ms), protocolos de endpoint USB (Interface 3 para controle HID, Interface 2 para streaming LCD) e manipulações de bits/endianness.
2. **Documentação Markdown (`.md`) por Módulo**:
   - Será criada uma pasta `docs/` dentro de `f108-pro-rust/` com documentação técnica dedicada para cada subsistema do driver.
3. **Orquestração Hierárquica de Subagentes para Economia de Tokens**:
   - O agente orquestrador principal delega tarefas autocontidas para subagentes utilizando modelos mais leves (`flash_lite` e `flash`), reservando modelos mais potentes (`pro`) ou a execução direta do orquestrador apenas para arquitetura crítica de baixo nível USB e resolução de problemas do compilador Rust.
   - **Economia de Tokens**: Tarefas puramente descritivas (documentação em Markdown), arquivos de configuração declarativos e módulos utilitários consom uma fração mínima do consumo usual ao usar `flash_lite` e `flash`.
   - **Garantia de Qualidade**: O agente principal permanece responsável pela revisão da tipagem, integração dos módulos no `lib.rs` / `main.rs`, execução do `cargo check` / `cargo test` e validação no hardware real.

---

## 2. Decisão Arquitetural: Driver em Espaço de Usuário (*Userspace Driver*)

> [!IMPORTANT]
> **Por que Userspace Driver em vez de Kernel Module (`rust-for-linux`)?**
>
> Embora o Linux suporte Rust para módulos de Kernel desde o 6.1, periféricos com recursos complexos (telas LCD, matriz RGB individual, upload de animações GIF, remapeamento via YAML) são gerenciados no Linux em **Userspace** (padrão de projetos consolidados como OpenRGB, Piper/Libratbag e Razer Chroma).
>
> - **Acesso Direto sem Risco de Pânico**: Através de `rusb` (`libusb-1.0`) e regras `udev`, temos acesso direto e nativo aos endpoints USB sem risco de travar o sistema operacional.
> - **Funcionalidades de Usuário**: Processamento de imagens, parsing de YAML e CLI interativa não existem dentro do kernel.
> - **Portabilidade e Facilidade**: Não depende de compilação a cada atualização de kernel (`DKMS`).

---

## 3. Desafios do Ambiente e Soluções

> [!WARNING]
> 1. **Rust Toolchain**: `cargo` e `rustc` ainda não estão instalados no host. Faremos a instalação automática via `rustup`.
> 2. **Partição NFS com `noexec`**: `/mnt/Files/` está montada com `noexec`, impedindo execução direta de binários gerados nela.
>    - **Solução**: Configuraremos `.cargo/config.toml` com `target-dir = "/tmp/cargo-target-f108-pro"` (ou `~/.cache/f108-target`), permitindo que a compilação, os testes (`cargo test`) e os binários rodem sem bloqueios de permissão.
> 3. **Dongle Conectado**: O dongle wireless Aula (`05ac:024f`) já foi detectado no host (Bus 001 Device 005), viabilizando testes reais de hardware.

---

## 4. Estrutura do Novo Projeto (`f108-pro-rust`)

```text
f108-pro-rust/
├── Cargo.toml                  # Dependências e definição de binários
├── .cargo/
│   └── config.toml             # Redirecionamento de target-dir (evita noexec do NFS)
├── README.md                   # Documentação geral e guia rápido de uso
├── udev/
│   └── 99-aula.rules           # Regra udev para acesso USB sem sudo
├── docs/                       # Documentação .md detalhada por módulo
│   ├── 01_architecture.md      # Visão geral da arquitetura do driver e USB stack
│   ├── 02_transport.md         # Documentação de interfaces USB (2 e 3), endpoints e rusb
│   ├── 03_lighting.md          # Protocolo dos 20 modos de luz, brilho, velocidade e direção
│   ├── 04_perkey_rgb.md        # Protocolo e mapeamento da matriz Per-Key de 576 bytes
│   ├── 05_key_remap.md         # Protocolo de remapeamento (Key, Media, Mouse, Combos)
│   ├── 06_lcd_clock.md         # Protocolo de sincronização de relógio e upload LCD
│   └── 07_cli_usage.md         # Manual de uso da interface CLI e exemplos YAML
├── src/
│   ├── lib.rs                  # Exportação da biblioteca core (aula) com docstrings
│   ├── main.rs                 # CLI f108-pro (subcomandos)
│   ├── aula/
│   │   ├── mod.rs              # Módulo core
│   │   ├── constants.rs        # Constantes, VIDs/PIDs, delays, comandos de protocolo
│   │   ├── transport/
│   │   │   ├── mod.rs          # Trait Transport
│   │   │   └── linux_usb.rs    # Implementação rusb com auto-detach
│   │   ├── device.rs           # Gerenciador de transações (begin/apply/finalize)
│   │   ├── lighting.rs         # Lógica dos 20 modos de luz e configuração
│   │   ├── perkey.rs           # Matriz Per-Key RGB, mapa de teclas e strip preamble
│   │   ├── remap.rs            # Remapeamento camadas Normal e FN, conversões HID/Consumer/Mouse
│   │   ├── clock.rs            # Sincronização do relógio LCD (pacotes com magic 0x5A)
│   │   └── lcd.rs              # Streaming de páginas de 4096 bytes para o LCD via EP 3
│   ├── cli/
│   │   ├── mod.rs              # Definições clap para a CLI
│   │   └── yaml_config.rs      # Parsing de arquivos YAML de iluminação e remapeamento
│   └── tools/
│       ├── mkimage.rs          # Gerador/conversor de GIFs/cores para buffers LCD RGB565
│       └── dumphid.rs          # Leitor e analisador de descritores HID
└── tests/
    ├── config_tests.rs         # Testes unitários de parsing de YAML e combos
    └── hardware_tests.rs       # Testes de integração direta no dispositivo conectado
```

---

## 5. Padrão de Comentários e Documentação

Todos os arquivos Rust seguirão a seguinte convenção estrita:
- **Cabeçalho de arquivo**: Explicando o papel daquele arquivo no ecossistema e o fluxo de dados.
- **Doc-comments Rust (`///` e `//!`)**: Documentando todas as structs, enums, métodos públicos e traits.
- **Comentários de bloco e linha**:
  - Explicando cada byte montado no buffer USB (ex: `payload[0] = 0x04; // Opcode de controle`).
  - Explicando a necessidade de cada intervalo de espera (`std::thread::sleep(Duration::from_millis(35))`).
  - Explicando a estrutura de trailers como `0x55AA` em little-endian.

Para cada módulo em `src/aula/`, haverá um arquivo correspondente em `docs/` contendo:
- Propósito técnico do módulo.
- Detalhamento do frame/pacote USB HID.
- Diagrama ou tabela de campos de bits.
- Exemplos de código em Rust consumindo o módulo.

---

## 6. Matriz de Atribuição de Modelos e Subagentes

Para maximizar a economia de tokens mantendo robustez e confiabilidade, a execução do projeto adota uma divisão por camadas de complexidade cognitiva:

| Camada | Modelo Utilizado | Responsabilidade / Tarefas | Subagente |
|---|---|---|---|
| **Tier 1 (Scaffold & Docs)** | `flash_lite` | Criação de esqueletos de diretórios, regras `udev`, `.cargo/config.toml`, transcrição de tabelas puras de constantes/keycodes e escrita de arquivos de documentação Markdown (`docs/*.md`, `README.md`). | Subagente dedicado com ferramentas de leitura/escrita de arquivos. |
| **Tier 2 (Módulos & CLI)** | `flash` | Implementação de módulos autocontidos em Rust com lógica isolada: `lighting.rs`, `clock.rs`, `yaml_config.rs` (Serde), `dumphid.rs`, `mkimage.rs` e testes unitários (`config_tests.rs`). | Subagente de codificação com ferramentas de leitura e edição. |
| **Tier 3 (Core USB & Orquestração)** | `pro` / Principal | Arquitetura das traits (`Transport`), driver USB de baixo nível (`rusb`), detach de interfaces de kernel (Interface 2 e 3), `Device` transaction manager, resolução de conflitos de borrow-checker/compilador e testes com dispositivo real. | Agente principal (Orquestrador). |

---

## 7. Fases de Execução com Delegação Otimizada

### Fase 1: Ambiente e Scaffold
1. **Orquestrador**: Verificar/instalar o toolchain Rust estável (`rustup`, `cargo`).
2. **Subagente (`flash_lite`)**:
   - Criar estrutura de diretórios (`src/`, `docs/`, `udev/`, `tests/`).
   - Criar `Cargo.toml` e `.cargo/config.toml` (redirecionando `target-dir` para `/tmp/cargo-target-f108-pro` por conta do `noexec` no NFS).
   - Criar a regra de permissão `udev/99-aula.rules`.

### Fase 2: Módulos Core em Rust (comentados) + Documentação .md
1. **Subagente (`flash_lite`)**:
   - Transcrever tabelas de `keycodes.go` e constantes brutas para `src/aula/constants.rs`.
   - Gerar os rascunhos de documentação técnica: `docs/01_architecture.md`, `docs/02_transport.md`, `docs/03_lighting.md`, `docs/04_perkey_rgb.md`, `docs/05_key_remap.md` e `docs/06_lcd_clock.md`.
2. **Orquestrador (`pro`)**:
   - Implementar a trait `Transport` (`src/aula/transport/mod.rs`).
   - Implementar `LinuxUsbTransport` (`src/aula/transport/linux_usb.rs`) usando `rusb` (tratamento de interfaces HID, liberação de drivers do kernel e envio de pacotes brutos).
   - Implementar `device.rs` (handshake `begin_transaction`, `apply_transaction`, `finalize_transaction` com delays de 35ms).
3. **Subagente (`flash`)**:
   - Implementar `src/aula/lighting.rs` (protocolo dos 20 modos de luz, brilho, velocidade e direção).
   - Implementar `src/aula/clock.rs` (cálculo de checksum, formatação da data/hora e pacotes com prefixo `0x5A`).
   - Implementar `src/aula/perkey.rs` (montagem da matriz de 576 bytes e mapeamento de 108 teclas).
   - Implementar `src/aula/remap.rs` (definição de layers normal e FN, conversão de códigos HID/Consumer).
   - Implementar `src/aula/lcd.rs` (divisão de páginas de 4096 bytes para o LCD).

### Fase 3: CLI, Utilitários e YAML (`src/cli/` e `src/tools/`)
1. **Subagente (`flash`)**:
   - Implementar `src/cli/yaml_config.rs` usando `serde_yaml` para compatibilidade com layouts YAML do driver Go.
   - Implementar `src/tools/dumphid.rs` e `src/tools/mkimage.rs`.
   - Implementar `src/cli/mod.rs` e `src/main.rs` com `clap` (comandos `light`, `brightness`, `off`, `modes`, `clock`, `keys`, `perkey`, `remap`, `lcd`).
2. **Subagente (`flash_lite`)**:
   - Gerar `docs/07_cli_usage.md` e o `README.md` completo do `f108-pro-rust` com exemplos de uso.

### Fase 4: Testes, Integração e Verificação
1. **Subagente (`flash`)**:
   - Criar `tests/config_tests.rs` (testes de parsing de layouts YAML e mapeamento de teclas).
2. **Orquestrador (`pro`)**:
   - Executar `cargo check` e `cargo test`.
   - Resolver eventuais erros de compilação ou incompatibilidade entre tipos.
   - Realizar teste de comunicação real com o hardware conectado (`05ac:024f` / `0c45:800a`): leitura de status, ajuste de modo de iluminação e sincronização de relógio.

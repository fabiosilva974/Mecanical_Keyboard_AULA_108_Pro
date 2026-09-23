# Arquitetura do Driver Userspace em Rust (AULA F108 Pro)

Este documento descreve a arquitetura de alto nível do driver userspace em Rust para o teclado mecânico **AULA F108 Pro**, construído sobre a biblioteca `rusb` (libusb wrapper para Rust).

---

## 1. Visão Geral da Arquitetura

O driver opera no espaço do usuário (userspace) e interage diretamente com o dispositivo USB através de transferências de controle USB HID e transferências de interrupção (interrupt endpoints). O design é modular, desacoplando a camada de transporte USB, a lógica de protocolo, a máquina de estados de transação e as interfaces de alto nível (iluminação, remapeamento de teclas, relógio e streaming de LCD).

```mermaid
graph TD
    A[Aplicação / CLI / GUI] --> B[Driver Core / Session Manager]
    B --> C[Transport Layer: rusb / USB Device]
    C --> D[Interface 3: Control & Feature Reports (64 bytes)]
    C --> E[Interface 2: LCD Streaming (EP3 OUT / EP4 IN)]
    B --> F[Protocol Modules]
    F --> F1[Lighting Protocol]
    F --> F2[Per-Key RGB Matrix]
    F --> F3[Key Remap & Layers]
    F --> F4[LCD & Real-time Clock Sync]
```

---

## 2. Fluxo de Transação de Comandos (Begin -> Init -> Data -> Apply -> Finalize)

O firmware do teclado baseado no chipset Sonix exige uma sequência estrita de opcodes para qualquer alteração de estado (iluminação, macros, configurações de teclas ou LCD). Cada transação segue rigorosamente o ciclo de vida abaixo:

1. **Begin (`0x04 0x18`)**: Inicia uma sessão de configuração, preparando o buffer interno do MCU para receber novos comandos.
2. **Sub-Init / Header**: Envia o opcode específico do subsistema (ex: `0x04 0x13` para iluminação, `0x04 0x28` para relógio, `0x04 0x72` para LCD).
3. **Payload Data**: Transmissão de blocos de dados de 64 bytes (Feature Reports) contendo parâmetros, cores, matrizes ou blocos de imagem.
4. **Apply (`0x04 0x02`)**: Ordena ao firmware que aplique os dados recebidos na memória volátil/não-volátil (EEPROM/Flash).
5. **Finalize (`0x04 0xF0`) & Trailer (`0x55 0xAA`)**: Encerra a transação de forma segura.

---

## 3. O Requisito Crítico de Temporização: Delay de 35ms

O firmware do microcontrolador Sonix possui buffers limitados e processamento single-threaded para gerenciamento USB e varredura de matriz de LED/LCD. Enviar comandos em rajada sem pausas resulta em travamentos, perda de pacotes ou corrupção de estado no teclado.

- **`CMD_DELAY_MS: u64 = 35`**: É obrigatório aguardar no mínimo **35 milissegundos** entre cada transação de controle USB (`libusb_control_transfer` ou escrita em endpoint).
- O gerenciador de sessão em Rust implementa automaticamente esse delay nas chamadas de escrita síncrona/assíncrona.

# Protocolo de Iluminação RGB por Tecla (Per-Key RGB)

Este documento especifica o protocolo de mapeamento de cores individuais por tecla (matriz Per-Key RGB) no teclado AULA F108 Pro, suportando até 144 slots de teclas físicas com blocos de 576 bytes.

---

## 1. Organização da Matriz Per-Key

O teclado possui uma matriz física que mapeia até **144 slots de teclas** (abrangendo o layout completo de 108 teclas mais chaves multimídia e zonas estendidas).

- Cada tecla individual é configurada com um bloco de **4 bytes** contendo os valores de cor e intensidade:
  - `Byte 0`: Intensidade de Vermelho (`R` - 0 a 255)
  - `Byte 1`: Intensidade de Verde (`G` - 0 a 255)
  - `Byte 2`: Intensidade de Azul (`B` - 0 a 255)
  - `Byte 3`: Flags de controle de LED / Efeito por tecla (`0x01` = Ligado, `0x00` = Desligado / Inibido)

- **Tamanho Total do Buffer**: `144 slots × 4 bytes = 576 bytes`.

---

## 2. Fluxo de Transmissão por Blocos (Chunking)

Como cada Feature Report USB possui capacidade máxima de 64 bytes (`REPORT_SIZE = 64`), o buffer total de 576 bytes da matriz Per-Key é dividido em múltiplos pacotes transmitidos sequencialmente:

1. **Cabeçalho de Inicialização**: Comando `OPCODE_BEGIN` seguido de `0x04 0x13` (Init Per-Key).
2. **Streaming de Chunks**: O buffer de 576 bytes é fracionado em 10 pacotes de 56 bytes de dados úteis (mais metadados de índice de bloco/offset).
3. **Aplicações e Finalização**: Envio de `OPCODE_APPLY` (`0x04 0x02`), seguido por `OPCODE_FINALIZE` e o trailer `0x55AA`.
4. **Temporização**: O delay obrigatório de **35ms** (`CMD_DELAY_MS`) deve ser respeitado entre cada chunk para evitar estouro de buffer no MCU Sonix.

# Protocolo de Sincronização de Relógio e Streaming de LCD TFT (AULA F108 Pro)

Este documento detalha o protocolo de sincronização do relógio em tempo real (RTC) e o streaming de páginas de imagem de 4096 bytes para o display TFT colorido embutido no AULA F108 Pro.

---

## 1. Sincronização do Relógio (Clock Sync)

O teclado possui um relógio interno exibido no display TFT quando o modo ocioso ou widget de relógio está ativo.

- **Opcode de Inicialização**: `OPCODE_CLOCK_INIT` (`0x04 0x28`).
- **Marcador Mágico**: `MAGIC_CLOCK_MARKER` (`0x5A`).
- **Formato de Dados**: Os valores de ano, mês, dia, hora, minuto e segundo são transmitidos em formato BCD (Binary-Coded Decimal) ou decimal direto, precedidos pelo marcador `0x5A` e finalizados com o trailer `0x55AA`.

---

## 2. Streaming de Imagens para o Display TFT (Interface 2)

O display TFT colorido é atualizado enviando blocos de dados através da **Interface 2** (`INTERFACE_LCD = 2`) utilizando o endpoint de interrupção OUT (`LCD_OUT_EP = 3`) e recebendo confirmações no endpoint IN (`LCD_IN_EP = 4`).

- **Tamanho da Página**: `LCD_PAGE_SIZE = 4096` bytes por bloco/quadro de imagem comprimida ou buffer RGB565.
- **Cabeçalho de Quadro**: Iniciado com `OPCODE_LCD_HEADER` (`0x04 0x72`) informando o número de páginas e dimensões do frame.
- **Fluxo de Envio**:
  1. Envio do cabeçalho de LCD via Feature Report na Interface 3.
  2. Transmissão sequencial das páginas de 4096 bytes pelo endpoint OUT 3 da Interface 2.
  3. Leitura do ACK no endpoint IN 4 para garantir que o buffer do display processou a página sem erros.
  4. Finalização da sessão com `OPCODE_FINALIZE` e `0x55AA`.

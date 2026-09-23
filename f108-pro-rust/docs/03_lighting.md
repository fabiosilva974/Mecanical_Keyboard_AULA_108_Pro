# Protocolo de Iluminação e Efeitos RGB (AULA F108 Pro)

Este documento especifica o protocolo de controle de retroiluminação RGB do teclado AULA F108 Pro, cobrindo os 20 modos de iluminação, parâmetros de cor, brilho, velocidade, direção e o trailer de fechamento `0x55AA`.

---

## 1. Visão Geral dos Modos de Luz

O firmware suporta 20 modos distintos (numerados de `0` a `19`), representados no driver pelo enum `LightingMode`:

| ID | Nome do Modo | Descrição |
|----|--------------|-----------|
| 0 | `Static` | Cor sólida estática em todas as teclas |
| 1 | `Breathing` | Efeito de respiração (fade in/out) |
| 2 | `SpectrumCycle` | Ciclo contínuo de cores do arco-íris |
| 3 | `Wave` | Onda de cores percorrendo o teclado horizontalmente |
| 4 | `Ripple` | Ondulação espalhando-se a partir da tecla pressionada |
| 5 | `Raindrop` | Queda aleatória de pingo de luz por tecla |
| 6 | `Snake` | Efeito cobra / marquee percorrendo as bordas e linhas |
| 7 | `ReactivePress` | Acendimento reativo imediato ao toque |
| 8 | `Laser` | Linha de laser cruzando a fileira da tecla pressionada |
| 9 | `SineWave` | Onda senoidal suave de brilho/cor |
| 10 | `Starlight` | Efeito céu estrelado cintilante |
| 11 | `Neon` | Transmissão contínua em gradiente neon |
| 12 | `AmbientFollow` | Sincronização com luz ambiente / áudio |
| 13 | `AudioVisualizer` | Equalizador visual por espectro sonoro |
| 14-18 | `CustomPreset1-5` | 5 Memórias de perfis customizados pelo usuário |
| 19 | `Off` | Retroiluminação totalmente desativada |

---

## 2. Estrutura do Pacote de Iluminação

O comando de configuração de iluminação é transmitido em blocos de 64 bytes encapsulados na transação padrão:

```
[ OPCODE_BEGIN (0x04 0x18) ] -> [ OPCODE_LIGHT_INIT (0x04 0x13) ] -> [ Payload de Configuração ] -> [ OPCODE_APPLY (0x04 0x02) ] -> [ OPCODE_FINALIZE (0x04 0xF0) ] -> [ TRAILER_55AA (0x55 0xAA) ]
```

### Formato do Payload (64 bytes):
- **Byte 0**: Modo de Luz (`0x00` a `0x13`)
- **Byte 1**: Nível de Brilho (`0x00` = 0%, `0x04` = 100% ou níveis de 0 a 4)
- **Byte 2**: Velocidade do Efeito (`0x01` a `0x05`)
- **Byte 3**: Direção do Efeito (`0x00` = Esquerda/Cima, `0x01` = Direita/Baixo)
- **Bytes 4-6**: Cor Primária RGB (`R`, `G`, `B`) para modos estáticos ou coloridos
- **Bytes 7-9**: Cor Secundária RGB (`R`, `G`, `B`) para efeitos multi-cor
- **Bytes 10-63**: Preenchimento com zeros (`0x00`) e terminação com o trailer `0x55AA` nas últimas posições estruturais.

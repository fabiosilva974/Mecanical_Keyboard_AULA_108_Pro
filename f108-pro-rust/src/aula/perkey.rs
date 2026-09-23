//! Modulo de controle de iluminacao RGB individual por tecla (Per-Key RGB) para o Aula F108 Pro.
//!
//! Permite definir cores RGB customizadas individualmente para cada uma das teclas fisicas
//! da matriz do teclado (ate 144 posicoes mapeaveis x 4 bytes = 576 bytes de buffer).
//!
//! # Protocolo Per-Key RGB
//! O processo de definicao de cores individuais exige duas fases de comunicacao:
//!
//! ## Fase 1: Preamble (`led_strip_setup`)
//! Prepara a fita e matriz de LEDs para receber o modo de iluminacao customizado por tecla:
//! 1. `begin_transaction()`: Envia `[0x04, 0x18]`.
//! 2. `lighting_init()`: Envia `[0x04, 0x13]`, byte 8 = `0x01`.
//! 3. Envia pacote de comando (64 bytes): `data[0] = 0x80`, `data[9] = brightness`, trailer `0x55 0xAA` nos bytes 14..16 (`send_command(data, false)`).
//! 4. `apply_transaction()`: Envia `[0x04, 0x02]`.
//! 5. `finalize_transaction()`: Envia `[0x04, 0xF0]`.
//!
//! ## Fase 2: Transmissao da Matriz de Cores
//! 1. `begin_transaction()`: Envia `[0x04, 0x18]`.
//! 2. Pacote de inicializacao Per-Key: 64 bytes com `[0x04, 0x23]`, byte 8 = `0x09`, com readback (`send_command(init, true)`).
//! 3. Montagem do buffer de 576 bytes (144 slots x 4 bytes):
//!    - Para cada `KeyColor`, grava na posicao `light_index * 4`: `[light_index, r, g, b]`.
//!    - Trailer final nos bytes 574 e 575: `buf[574] = 0x55`, `buf[575] = 0xAA`.
//! 4. Envio do buffer completo segmentado em pacotes de 64 bytes via `send_multi_packet(&buf, true)`.
//! 5. `apply_transaction()`: Envia `[0x04, 0x02]`.
//! 6. Finalizacao com readback: `send_command(&[0x04, 0xF0], true)`.

use anyhow::{bail, Context, Result};
use super::constants::REPORT_SIZE;
use super::device::Device;

/// Tamanho total em bytes da matriz de cores Per-Key RGB (144 posicoes x 4 bytes = 576 bytes / 0x240).
pub const PER_KEY_RGB_SIZE: usize = 0x240;

/// Representa a cor individual atribuida a uma tecla fisica da matriz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyColor {
    /// Indice fisico ou de iluminacao da tecla (1..=143).
    pub light_index: u8,
    /// Intensidade do canal Vermelho (Red), de 0 a 255.
    pub r: u8,
    /// Intensidade do canal Verde (Green), de 0 a 255.
    pub g: u8,
    /// Intensidade do canal Azul (Blue), de 0 a 255.
    pub b: u8,
}

impl KeyColor {
    /// Cria uma nova definicao de cor para o indice de tecla indicado.
    pub fn new(light_index: u8, r: u8, g: u8, b: u8) -> Self {
        Self {
            light_index,
            r,
            g,
            b,
        }
    }
}

impl Device {
    /// Executa o preambulo de preparacao dos LEDs para receber configuracao Per-Key.
    ///
    /// Configura o controlador para o modo customizado (0x80) e define o nivel de brilho desejado.
    ///
    /// # Parametros
    /// - `brightness`: Nivel de brilho dos LEDs (0 a 5).
    pub fn led_strip_setup(&mut self, brightness: u8) -> Result<()> {
        if brightness > 5 {
            bail!(
                "Nivel de brilho {} invalido: o valor maximo permitido e 5",
                brightness
            );
        }

        self.begin_transaction()
            .context("Falha no begin_transaction do led_strip_setup")?;

        self.lighting_init()
            .context("Falha no lighting_init do led_strip_setup")?;

        let mut data = [0u8; REPORT_SIZE];
        data[0] = 0x80;
        data[9] = brightness;
        data[14] = 0x55;
        data[15] = 0xAA;

        self.send_command(&data, false)
            .context("Falha ao enviar comando de setup da fita de LED")?;

        self.apply_transaction()
            .context("Falha no apply_transaction do led_strip_setup")?;

        self.finalize_transaction()
            .context("Falha no finalize_transaction do led_strip_setup")?;

        Ok(())
    }

    /// Configura as cores individuais de cada tecla especificada na lista `keys`,
    /// aplicando o nivel de brilho informado.
    ///
    /// # Parametros
    /// - `keys`: Slice contendo os mapeamentos [`KeyColor`] a serem gravados.
    /// - `brightness`: Nivel de brilho global para o modo Per-Key (0 a 5).
    ///
    /// # Erros
    /// Retorna erro em caso de brilho invalido ou falha de comunicacao USB durante qualquer etapa.
    pub fn set_per_key_rgb(&mut self, keys: &[KeyColor], brightness: u8) -> Result<()> {
        // Fase 1: Preambulo de preparacao
        self.led_strip_setup(brightness)
            .context("Falha ao executar preambulo da fita de LED (led_strip_setup)")?;

        // Fase 2: Transmissao da matriz de cores por tecla
        self.begin_transaction()
            .context("Falha ao iniciar transacao Per-Key RGB")?;

        // Inicializacao do subsistema Per-Key (0x04 0x23, byte 8 = 0x09) com readback
        let mut init = [0u8; REPORT_SIZE];
        init[0] = 0x04;
        init[1] = 0x23;
        init[8] = 0x09;

        self.send_command(&init, true)
            .context("Falha ao inicializar subsistema Per-Key RGB (0x04 0x23)")?;

        // Montagem do buffer de 576 bytes
        let mut buf = [0u8; PER_KEY_RGB_SIZE];

        for k in keys {
            let idx = k.light_index as usize;
            let offset = idx * 4;
            if offset + 3 < PER_KEY_RGB_SIZE {
                buf[offset] = k.light_index;
                buf[offset + 1] = k.r;
                buf[offset + 2] = k.g;
                buf[offset + 3] = k.b;
            }
        }

        // Trailer de confirmacao obrigatorio nos dois ultimos bytes da matriz de 576 bytes
        buf[574] = 0x55;
        buf[575] = 0xAA;

        // Transmissao multi-pacote com confirmacao (readback) no ultimo bloco de 64 bytes
        self.send_multi_packet(&buf, true)
            .context("Falha ao transmitir matriz de cores Per-Key RGB via multi-pacote")?;

        self.apply_transaction()
            .context("Falha ao aplicar matriz de cores Per-Key RGB")?;

        // Finalizacao com confirmacao (readback ativo conforme especificacao Sonix)
        self.send_command(&[0x04, 0xF0], true)
            .context("Falha ao finalizar persistencia de cores Per-Key RGB")?;

        Ok(())
    }
}

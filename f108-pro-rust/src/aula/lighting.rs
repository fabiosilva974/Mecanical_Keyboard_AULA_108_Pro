//! Modulo de controle de iluminacao RGB global para o teclado Aula F108 Pro.
//!
//! Este modulo fornece a estrutura [`LightingConfig`] e os metodos necessarios
//! para controlar os modos de animacao, cores RGB, velocidade, brilho e direcao
//! da iluminacao embutida no teclado Aula F108 Pro.
//!
//! # Protocolo de Iluminacao Sonix / Microdia
//! A configuracao de iluminacao global segue o seguinte fluxo estrito de Feature Reports:
//! 1. `begin_transaction()`: Envio do opcode `0x04 0x18` com confirmacao (readback).
//! 2. `lighting_init()`: Envio do opcode `0x04 0x13`, byte 8 = `0x01` com confirmacao (readback).
//! 3. Transmissao do relatorio de 64 bytes contendo modo, cores e parametros, terminado com `0x55 0xAA`.
//! 4. `apply_transaction()`: Envio do opcode `0x04 0x02` com confirmacao (readback).
//! 5. `finalize_transaction()`: Envio do opcode `0x04 0xF0` sem confirmacao para persistir na flash.

use anyhow::{bail, Context, Result};
use super::constants::{LightingMode, REPORT_SIZE, TRAILER_55AA};
use super::device::Device;

/// Parametros de configuracao para a iluminacao RGB global do teclado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightingConfig {
    /// Modo de iluminacao ativo (efeito pre-definido ou desligado).
    pub mode: LightingMode,
    /// Intensidade do canal Vermelho (Red), de 0 a 255.
    pub r: u8,
    /// Intensidade do canal Verde (Green), de 0 a 255.
    pub g: u8,
    /// Intensidade do canal Azul (Blue), de 0 a 255.
    pub b: u8,
    /// Nivel de brilho da iluminacao (limite suportado pelo hardware: 0 a 5).
    pub brightness: u8,
    /// Velocidade do efeito de animacao luminosa (limite suportado pelo hardware: 0 a 5).
    pub speed: u8,
    /// Direcao da animacao luminosa (quando aplicavel ao modo).
    pub direction: u8,
    /// Modo multicolorido / arco-iris (`true`) ou uso de cor solida fixa RGB (`false`).
    pub colorful: bool,
}

impl Default for LightingConfig {
    /// Retorna uma configuracao padrao de iluminacao com efeito Colourful,
    /// brilho em nivel 4 e velocidade 3.
    fn default() -> Self {
        Self {
            mode: LightingMode::Colourful,
            r: 0xFF,
            g: 0xFF,
            b: 0xFF,
            brightness: 4,
            speed: 3,
            direction: 0,
            colorful: true,
        }
    }
}

impl LightingConfig {
    /// Cria uma nova instancia de `LightingConfig` baseada no modo fornecido,
    /// herdando os demais valores padrao.
    pub fn new(mode: LightingMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    /// Valida se os limites operacionais exigidos pelo hardware sao atendidos.
    ///
    /// # Erros
    /// - Retorna erro caso `brightness > 5`.
    /// - Retorna erro caso `speed > 5`.
    pub fn validate(&self) -> Result<()> {
        if self.brightness > 5 {
            bail!(
                "Nivel de brilho {} invalido: o valor maximo permitido pelo teclado e 5",
                self.brightness
            );
        }
        if self.speed > 5 {
            bail!(
                "Velocidade de iluminacao {} invalida: o valor maximo permitido pelo teclado e 5",
                self.speed
            );
        }
        Ok(())
    }
}

impl Device {
    /// Configura o efeito e os parametros de iluminacao global do teclado Aula F108 Pro.
    ///
    /// # Sequencia de Operacoes
    /// 1. Valida se os niveis de brilho e velocidade estao no intervalo `0..=5`.
    /// 2. Executa `begin_transaction()`.
    /// 3. Executa `lighting_init()` (opcode `0x04 0x13`).
    /// 4. Preenche o payload de 64 bytes:
    ///    - `payload[0]`: Codigo numerico do modo (`LightingMode as u8`).
    ///    - Se `mode != LightingMode::Off`:
    ///      - `payload[1..=3]`: Componentes R, G, B.
    ///      - `payload[8]`: Flag `colorful` (`1` se true, `0` se false).
    ///      - `payload[9]`: Nivel de brilho (0 a 5).
    ///      - `payload[10]`: Velocidade da animacao (0 a 5).
    ///      - `payload[11]`: Direcao da animacao.
    ///    - `payload[14..16]`: Trailer obrigatorio `0x55, 0xAA`.
    /// 5. Envia o payload com `send_command(payload, false)`.
    /// 6. Aplica a transacao com `apply_transaction()`.
    /// 7. Finaliza e salva na memoria flash interna com `finalize_transaction()`.
    ///
    /// # Erros
    /// Retorna erro em caso de valores fora do limite ou falha de comunicacao com a controladora USB.
    pub fn set_lighting(&mut self, cfg: &LightingConfig) -> Result<()> {
        cfg.validate().context("Configuracao de iluminacao invalida")?;

        self.begin_transaction()
            .context("Falha ao iniciar transacao para configuracao de iluminacao")?;

        self.lighting_init()
            .context("Falha ao inicializar o subsistema de iluminacao")?;

        let mut payload = [0u8; REPORT_SIZE];
        payload[0] = cfg.mode as u8;

        if cfg.mode != LightingMode::Off {
            payload[1] = cfg.r;
            payload[2] = cfg.g;
            payload[3] = cfg.b;
            payload[8] = if cfg.colorful { 1 } else { 0 };
            payload[9] = cfg.brightness;
            payload[10] = cfg.speed;
            payload[11] = cfg.direction;
        }

        payload[14] = TRAILER_55AA[0];
        payload[15] = TRAILER_55AA[1];

        self.send_command(&payload, false)
            .context("Falha ao enviar pacote de dados de iluminacao")?;

        self.apply_transaction()
            .context("Falha ao aplicar configuracao de iluminacao")?;

        self.finalize_transaction()
            .context("Falha ao finalizar e gravar configuracao de iluminacao")?;

        Ok(())
    }
}

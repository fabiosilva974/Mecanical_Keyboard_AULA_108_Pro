//! Trait de transporte para comunicacao de baixo nivel com o teclado Aula F108 Pro.
//!
//! Este modulo define a abstracao necessaria para envio e recebimento de Feature Reports
//! (Interface 3) e streaming de buffers graficos para o display LCD (Interface 2).

use std::time::Duration;
use anyhow::Result;

/// Tamanho fixo dos relatorios HID Feature (64 bytes sem prefixo de Report ID).
pub const REPORT_SIZE: usize = 64;

/// Trait que abstrai as operacoes de barramento USB/HID do teclado.
pub trait Transport: Send + Sync {
    /// Envia um relatorio HID Feature de 64 bytes para a Interface 3.
    fn set_feature_report(&mut self, data: &[u8; REPORT_SIZE]) -> Result<()>;

    /// Le um relatorio HID Feature de 64 bytes da Interface 3.
    fn get_feature_report(&mut self) -> Result<[u8; REPORT_SIZE]>;

    /// Prepara a interface de dados LCD (Interface 2) para transferencia.
    fn init_lcd(&mut self) -> Result<()>;

    /// Envia uma pagina de 4096 bytes para o endpoint OUT do LCD (Interface 2, EP 3).
    fn write_lcd_page(&mut self, data: &[u8]) -> Result<()>;

    /// Le a confirmacao (ACK) de 64 bytes do endpoint IN do LCD com timeout.
    fn read_lcd_ack(&mut self, timeout: Duration) -> Result<Vec<u8>>;
}

pub mod linux_usb;

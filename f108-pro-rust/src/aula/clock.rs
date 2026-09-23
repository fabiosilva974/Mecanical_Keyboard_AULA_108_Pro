//! Modulo de sincronizacao de data e hora do relogio interno / tela LCD do teclado Aula F108 Pro.
//!
//! O microcontrolador do teclado mantem um relogio em tempo real exibido no visor TFT.
//! Este modulo permite sincronizar os contadores de ano, mes, dia, hora, minuto, segundo
//! e dia da semana a partir do relogio do sistema operacional host.
//!
//! # Protocolo de Sincronizacao de Relogio
//! A sincronizacao de relogio segue a seguinte sequencia estrita de Feature Reports:
//! 1. `begin_transaction()`: Envio do opcode `0x04 0x18` com readback.
//! 2. Inicializacao do subsistema de relogio: Opcode `0x04 0x28`, byte 8 = `0x01` com readback.
//! 3. Transmissao do pacote de dados (64 bytes) com readback:
//!    - `data[0] = 0x00`
//!    - `data[1] = 0x01` (Indice do perfil / Profile 1)
//!    - `data[2] = 0x5A` (Marcador magico `MAGIC_CLOCK_MARKER`)
//!    - `data[3] = (ano % 2000) as u8`
//!    - `data[4] = mes as u8` (1..=12)
//!    - `data[5] = dia as u8` (1..=31)
//!    - `data[6] = hora as u8` (0..=23)
//!    - `data[7] = minuto as u8` (0..=59)
//!    - `data[8] = segundo as u8` (0..=59)
//!    - `data[10] = dia da semana as u8` (0 = Domingo, 1 = Segunda, ..., 6 = Sabado)
//!    - `data[62] = 0x55` (Trailer byte 0)
//!    - `data[63] = 0xAA` (Trailer byte 1)
//! 4. `apply_transaction()`: Envio do opcode `0x04 0x02` com readback.
//!
//! *Nota de projeto*: Ao contrario de outras operacoes de persistencia, a sincronizacao
//! de data/hora nao invoca `finalize_transaction()`, para que os registradores do RTC
//! entrem em operacao imediata sem reiniciar o estado do teclado.

use chrono::{DateTime, Datelike, Local, Timelike};
use anyhow::{Context, Result};
use super::constants::{MAGIC_CLOCK_MARKER, REPORT_SIZE};
use super::device::Device;

impl Device {
    /// Sincroniza o relogio em tempo real (RTC) do teclado com o timestamp local fornecido.
    ///
    /// # Parametros
    /// - `dt`: Data e hora locais (`DateTime<Local>`) a serem enviadas ao hardware.
    ///
    /// # Erros
    /// Retorna erro se qualquer etapa da transacao USB falhar ou se a confirmacao
    /// (readback) nao for validada pela controladora.
    pub fn sync_clock(&mut self, dt: DateTime<Local>) -> Result<()> {
        self.begin_transaction()
            .context("Falha ao iniciar transacao para sincronizacao do relogio")?;

        // 1. Pacote de inicializacao do subsistema de relogio (0x04 0x28, byte 8 = 0x01)
        let mut init_payload = [0u8; REPORT_SIZE];
        init_payload[0] = 0x04;
        init_payload[1] = 0x28;
        init_payload[8] = 0x01;

        self.send_command(&init_payload, true)
            .context("Falha ao inicializar subsistema de relogio (0x04 0x28)")?;

        // 2. Pacote com os valores de data e hora formatados para o RTC
        let mut clock_data = [0u8; REPORT_SIZE];
        clock_data[0] = 0x00;
        clock_data[1] = 0x01; // Profile 1
        clock_data[2] = MAGIC_CLOCK_MARKER; // 0x5A

        let year = dt.year();
        clock_data[3] = (year % 2000) as u8;
        clock_data[4] = dt.month() as u8;
        clock_data[5] = dt.day() as u8;
        clock_data[6] = dt.hour() as u8;
        clock_data[7] = dt.minute() as u8;
        clock_data[8] = dt.second() as u8;
        // clock_data[9] mantido como 0x00 (reservado pelo firmware)
        clock_data[10] = dt.weekday().num_days_from_sunday() as u8; // 0=Domingo..6=Sabado

        clock_data[62] = 0x55;
        clock_data[63] = 0xAA;

        self.send_command(&clock_data, true)
            .context("Falha ao transmitir dados de data/hora do relogio")?;

        // 3. Aplica a transacao para atualizar os registradores RTC
        self.apply_transaction()
            .context("Falha ao aplicar sincronizacao de relogio")?;

        Ok(())
    }
}

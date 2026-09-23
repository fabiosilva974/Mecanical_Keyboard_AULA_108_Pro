//! Gerenciador de conexao e transacoes USB para o teclado Aula F108 Pro.
//!
//! Este modulo encapsula a maquina de estados de comandos do teclado, garantindo que
//! cada operacao de escrita respeite a sequencia exigida pelo firmware da Sonix/Microdia:
//!
//! 1. `begin_transaction`  -> [0x04, 0x18] com readback (GET_REPORT)
//! 2. Inicializacao especifica (ex: `lighting_init` [0x04, 0x13])
//! 3. Envio de dados (64 bytes por pacote ou multi-pacotes de 576 bytes)
//! 4. `apply_transaction`  -> [0x04, 0x02] com readback (GET_REPORT)
//! 5. `finalize_transaction` -> [0x04, 0xF0] sem readback
//!
//! Em todas as operacoes, e obrigatorio um delay minimo de 35ms entre pacotes.

use std::thread::sleep;
use std::time::Duration;
use anyhow::{Context, Result};

use super::transport::{linux_usb::LinuxUsbTransport, Transport, REPORT_SIZE};

/// Delay obrigatorio de 35ms exigido pelo firmware entre comandos USB Feature.
pub const CMD_DELAY: Duration = Duration::from_millis(35);

/// Representa uma conexao ativa com o teclado Aula F108 Pro.
pub struct Device {
    transport: Box<dyn Transport>,
}

impl Device {
    /// Inicializa a conexao com o teclado utilizando o transporte USB do Linux.
    pub fn open() -> Result<Self> {
        let transport = LinuxUsbTransport::open().context("Falha ao abrir transporte USB")?;
        Ok(Self {
            transport: Box::new(transport),
        })
    }

    /// Cria uma instancia de `Device` a partir de uma implementacao customizada de `Transport` (util para mocks e testes).
    pub fn from_transport(transport: Box<dyn Transport>) -> Self {
        Self { transport }
    }

    /// Retorna uma referencia mutavel ao transporte subjacente (utilizado pelo modulo LCD).
    pub fn transport_mut(&mut self) -> &mut dyn Transport {
        self.transport.as_mut()
    }

    /// Envia um relatorio Feature de 64 bytes diretamente para a Interface 3.
    pub fn set_feature_report(&mut self, data: &[u8; REPORT_SIZE]) -> Result<()> {
        self.transport.set_feature_report(data)
    }

    /// Le um relatorio Feature de 64 bytes diretamente da Interface 3.
    pub fn get_feature_report(&mut self) -> Result<[u8; REPORT_SIZE]> {
        self.transport.get_feature_report()
    }

    /// Constroi um relatorio Feature de 64 bytes a partir de um slice de payload e envia para o teclado.
    /// Respeita o delay de 35ms apos o envio e realiza leitura de confirmacao (readback) caso solicitado.
    pub fn send_command(&mut self, payload: &[u8], readback: bool) -> Result<()> {
        let mut report = [0u8; REPORT_SIZE];
        let copy_len = payload.len().min(REPORT_SIZE);
        report[..copy_len].copy_from_slice(&payload[..copy_len]);

        self.set_feature_report(&report)
            .context("Falha ao enviar comando via Feature Report")?;

        sleep(CMD_DELAY);

        if readback {
            let _ = self.get_feature_report()
                .context("Falha ao ler confirmacao (readback) do teclado")?;
            sleep(CMD_DELAY);
        }

        Ok(())
    }

    /// Inicia uma transacao de configuracao (Opcode 0x04 0x18 com readback).
    pub fn begin_transaction(&mut self) -> Result<()> {
        self.send_command(&[0x04, 0x18], true)
            .context("Falha na instrucao begin_transaction (0x04 0x18)")
    }

    /// Aplica as configuracoes enviadas na transacao atual (Opcode 0x04 0x02 com readback).
    pub fn apply_transaction(&mut self) -> Result<()> {
        self.send_command(&[0x04, 0x02], true)
            .context("Falha na instrucao apply_transaction (0x04 0x02)")
    }

    /// Finaliza e persiste as alteracoes na memoria flash do teclado (Opcode 0x04 0xF0 sem readback).
    pub fn finalize_transaction(&mut self) -> Result<()> {
        self.send_command(&[0x04, 0xF0], false)
            .context("Falha na instrucao finalize_transaction (0x04 0xF0)")
    }

    /// Inicializacao de comando de iluminacao (Opcode 0x04 0x13, byte[8]=0x01 com readback).
    pub fn lighting_init(&mut self) -> Result<()> {
        let mut payload = [0u8; REPORT_SIZE];
        payload[0] = 0x04;
        payload[1] = 0x13;
        payload[8] = 0x01;
        self.send_command(&payload, true)
            .context("Falha na inicializacao de iluminacao (0x04 0x13)")
    }

    /// Divide um buffer de dados extenso (ex: matriz de 576 bytes de per-key ou remap) em blocos
    /// consecutivos de 64 bytes e os envia em sequencia com intervalo de 35ms.
    pub fn send_multi_packet(&mut self, data: &[u8], readback_last: bool) -> Result<()> {
        let total_packets = (data.len() + REPORT_SIZE - 1) / REPORT_SIZE;

        for i in 0..total_packets {
            let mut report = [0u8; REPORT_SIZE];
            let start = i * REPORT_SIZE;
            let end = (start + REPORT_SIZE).min(data.len());
            report[..end - start].copy_from_slice(&data[start..end]);

            self.set_feature_report(&report)
                .context(format!("Falha no envio do pacote {}/{}", i + 1, total_packets))?;

            sleep(CMD_DELAY);
        }

        if readback_last {
            let _ = self.get_feature_report()
                .context("Falha no readback apos transmissao multi-pacote")?;
            sleep(CMD_DELAY);
        }

        Ok(())
    }
}

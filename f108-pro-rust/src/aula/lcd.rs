//! Modulo de transmissao e streaming de imagens/animacoes para a tela LCD TFT do Aula F108 Pro.
//!
//! A tela colorida do teclado suporta a gravacao de imagens estaticas ou sequencias de quadros
//! (GIF animado) atraves de uma interface dedicada de alta velocidade (Interface USB 2).
//!
//! # Protocolo de Gravacao do Display LCD
//! As imagens sao empacotadas em paginas binarias de exatamente 4096 bytes (4 KB).
//! O fluxo de envio envolve sincronizacao via canal de controle (Interface 3) e streaming
//! via canal dedicado (Interface 2):
//!
//! 1. Validacao: O tamanho do buffer deve ser maior que zero e multiplo exato de 4096 bytes.
//! 2. Inicializacao do transporte LCD: `self.transport_mut().init_lcd()?` (reivindica Interface 2).
//! 3. `begin_transaction()`: Envia `[0x04, 0x18]` com readback.
//! 4. Header de imagem (64 bytes na Interface 3):
//!    - `hdr[0] = 0x04`
//!    - `hdr[1] = 0x72` (Opcode LCD Header)
//!    - `hdr[2] = image_number` (Indice do slot de imagem/quadro, ex: 0 ou 1)
//!    - `hdr[8] = (page_count & 0xFF) as u8` (Byte inferior do total de paginas)
//!    - `hdr[9] = ((page_count >> 8) & 0xFF) as u8` (Byte superior do total de paginas)
//!    Enviado com `send_command(&hdr, true)`.
//! 5. Para cada pagina `i` de `0..page_count`:
//!    - Envia 4096 bytes pelo endpoint OUT do LCD (Interface 2, EP 3): `write_lcd_page(page)`.
//!    - Aguarda confirmacao (ACK de 64 bytes) com timeout de 300ms (erros de timeout sao tolerados
//!      para evitar interrupcao em displays que processam o frame de forma assincrona).
//!    - Invoca a callback de progresso: `progress(i + 1, page_count)`.
//! 6. `apply_transaction()`: Envia `[0x04, 0x02]` com readback para comitar os dados na tela.

use std::time::Duration;
use anyhow::{bail, Context, Result};
use super::constants::REPORT_SIZE;
use super::device::Device;

/// Tamanho fixo de cada bloco / pagina de transferencia para o display LCD (4096 bytes).
pub const LCD_PAGE_SIZE: usize = 4096;

impl Device {
    /// Transmite um buffer de dados de imagem para o display LCD colorido do teclado.
    ///
    /// # Parametros
    /// - `buf`: Slice contendo os dados binarios da imagem. Deve ter tamanho maior que 0
    ///   e ser multiplo exato de 4096 bytes (`LCD_PAGE_SIZE`).
    /// - `image_number`: Indice de slot ou numero de identificacao da imagem no firmware (ex: 0 para tela principal).
    /// - `progress`: Funcao de callback executada apos o envio de cada pagina com a assinatura `(paginas_enviadas, total_paginas)`.
    ///
    /// # Erros
    /// Retorna erro se o buffer tiver tamanho invalido, ou se ocorrer erro fatal
    /// no barramento USB ao inicializar a interface, gravar comandos ou transmitir paginas.
    pub fn upload_lcd_image<F>(&mut self, buf: &[u8], image_number: u8, mut progress: F) -> Result<()>
    where
        F: FnMut(usize, usize),
    {
        // 1. Validacao de tamanho
        if buf.is_empty() || buf.len() % LCD_PAGE_SIZE != 0 {
            bail!(
                "Tamanho de buffer LCD invalido ({} bytes): o buffer deve ser nao vazio e multiplo de {} bytes",
                buf.len(),
                LCD_PAGE_SIZE
            );
        }

        let page_count = buf.len() / LCD_PAGE_SIZE;

        // 2. Prepara e reivindica a Interface 2 do LCD
        self.transport_mut()
            .init_lcd()
            .context("Falha ao inicializar interface dedicada do LCD (Interface 2)")?;

        // 3. Inicia a transacao de controle
        self.begin_transaction()
            .context("Falha ao iniciar transacao para upload de imagem LCD")?;

        // 4. Monta e transmite o cabecalho de imagem (64 bytes)
        let mut hdr = [0u8; REPORT_SIZE];
        hdr[0] = 0x04;
        hdr[1] = 0x72; // OPCODE_LCD_HEADER
        hdr[2] = image_number;
        hdr[8] = (page_count & 0xFF) as u8;
        hdr[9] = ((page_count >> 8) & 0xFF) as u8;

        self.send_command(&hdr, true)
            .context("Falha ao enviar cabecalho de upload da imagem LCD (0x04 0x72)")?;

        // 5. Transmissao pagina a pagina pelo endpoint OUT da Interface 2
        for i in 0..page_count {
            let start = i * LCD_PAGE_SIZE;
            let end = start + LCD_PAGE_SIZE;
            let page = &buf[start..end];

            self.transport_mut()
                .write_lcd_page(page)
                .context(format!("Falha ao enviar pagina {}/{} do LCD", i + 1, page_count))?;

            // Aguarda leitura de confirmacao (ACK) de 64 bytes com timeout de 300ms.
            // Timeout e deliberadamente ignorado para compatibilidade com firmware assincrono.
            let _ = self.transport_mut().read_lcd_ack(Duration::from_millis(300));

            // Notifica o progresso da operacao
            progress(i + 1, page_count);
        }

        // 6. Aplica a transacao para finalizar a renderizacao no visor
        self.apply_transaction()
            .context("Falha ao aplicar transacao de imagem LCD")?;

        Ok(())
    }
}

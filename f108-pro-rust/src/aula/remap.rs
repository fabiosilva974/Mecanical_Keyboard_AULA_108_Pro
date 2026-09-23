//! Modulo de remapeamento de teclas fisicas e camada FN para o teclado Aula F108 Pro.
//!
//! Permite reatribuir qualquer tecla fisica da matriz para outra tecla padrao (HID),
//! combinacao com teclas modificadoras (Ctrl/Alt/Shift/GUI), funcoes multimidia (Consumer Control),
//! emulacao de mouse ou macros.
//!
//! O firmware mantem duas tabelas de remapeamento independentes de 576 bytes (144 slots x 4 bytes):
//! - Camada Normal (Base Layer): Inicializada com opcode `0x04 0x11`.
//! - Camada de Funcao (FN Layer): Inicializada com opcode `0x04 0x27`.
//!
//! # Protocolo de Gravacao da Tabela de Remapeamento
//! 1. `begin_transaction()`: Envia `[0x04, 0x18]` com readback.
//! 2. Inicializacao: Envia 64 bytes com `[0x04, 0x11]` (Normal) ou `[0x04, 0x27]` (FN), byte 8 = `0x09` com readback.
//! 3. Buffer de 576 bytes:
//!    - Para cada `KeyRemap`, grava na posicao `source_index * 4`: `[action, param1, param2, param3]`.
//!    - Trailer nos bytes 574 e 575: `buf[574] = 0xAA`, `buf[575] = 0x55` (representacao little-endian de `0x55AA`).
//! 4. Transmissao multi-pacote: `send_multi_packet(&buf, false)`.
//! 5. `apply_transaction()`: Envia `[0x04, 0x02]` com readback.
//! 6. Finalizacao com readback: `send_command(&[0x04, 0xF0], true)`.

use anyhow::{Context, Result};
use super::constants::REPORT_SIZE;
use super::device::Device;

/// Tamanho em bytes da tabela de remapeamento na controladora (144 posicoes x 4 bytes = 576 bytes).
pub const REMAP_TABLE_SIZE: usize = 576;

/// Categoria de acao a ser executada ao pressionar a tecla fisica remapeada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RemapAction {
    /// Sem acao / tecla desabilitada.
    None = 0x00,
    /// Tecla de funcao especial (ex: FN, modo de iluminacao).
    Special = 0x01,
    /// Tecla padrao do teclado HID (Usage Page 0x07) com modificadores opcionais.
    Key = 0x02,
    /// Controle multimidia ou de consumo (Consumer Control / Page 0x0C).
    Consumer = 0x03,
    /// Troca rapida de perfil de configuracao de hardware.
    Profile = 0x05,
    /// Execucao de sequencia gravada de macro.
    Macro = 0x06,
    /// Emulacao de clique ou rolagem do mouse.
    Mouse = 0x07,
}

impl RemapAction {
    /// Converte um codigo numerico `u8` para a variante correspondente de `RemapAction`.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::None),
            0x01 => Some(Self::Special),
            0x02 => Some(Self::Key),
            0x03 => Some(Self::Consumer),
            0x05 => Some(Self::Profile),
            0x06 => Some(Self::Macro),
            0x07 => Some(Self::Mouse),
            _ => None,
        }
    }
}

/// Define o remapeamento de uma tecla de origem para uma acao de destino e seus parametros.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyRemap {
    /// Indice fisico da tecla de origem na matriz (1..=143).
    pub source_index: u8,
    /// Tipo de acao correspondente a ser disparada.
    pub action: RemapAction,
    /// Primeiro parametro da acao (ex: codigo HID da tecla ou botao do mouse).
    pub param1: u8,
    /// Segundo parametro da acao (ex: mascara de modificadores ou acao de clique).
    pub param2: u8,
    /// Terceiro parametro da acao (ex: delta da roda de rolagem do mouse).
    pub param3: u8,
}

impl KeyRemap {
    /// Cria uma regra de substituicao direta de tecla por outro codigo HID.
    ///
    /// # Parametros
    /// - `source_index`: Indice da tecla fisica (ex: `KEY_CAPSLOCK`).
    /// - `target_hid`: Codigo HID de destino (ex: `0x29` para ESC).
    pub fn new_key_swap(source_index: u8, target_hid: u8) -> Self {
        Self {
            source_index,
            action: RemapAction::Key,
            param1: target_hid,
            param2: 0,
            param3: 0,
        }
    }

    /// Cria uma regra de combinacao de tecla (atalho com modificadores).
    ///
    /// # Parametros
    /// - `source_index`: Indice da tecla fisica.
    /// - `target_hid`: Codigo HID da tecla principal.
    /// - `modifiers`: Mascara binaria de teclas modificadoras (Ctrl/Shift/Alt/GUI).
    pub fn new_key_combo(source_index: u8, target_hid: u8, modifiers: u8) -> Self {
        Self {
            source_index,
            action: RemapAction::Key,
            param1: target_hid,
            param2: modifiers,
            param3: 0,
        }
    }

    /// Cria uma regra de remapeamento para controle de consumidor/multimidia (volume, play/pause, etc.).
    ///
    /// # Parametros
    /// - `source_index`: Indice da tecla fisica.
    /// - `consumer_code`: Codigo do comando multimidia (ex: `0xE2` para Mute).
    pub fn new_consumer_remap(source_index: u8, consumer_code: u8) -> Self {
        Self {
            source_index,
            action: RemapAction::Consumer,
            param1: consumer_code,
            param2: 0,
            param3: 0,
        }
    }

    /// Cria uma regra de emulacao de mouse (botoes esquerdo, direito, central ou rolagem de scroll).
    ///
    /// # Parametros
    /// - `source_index`: Indice da tecla fisica.
    /// - `button`: Mascara do botao (1=Esquerdo, 2=Direito, 4=Meio).
    /// - `action_type`: Tipo de evento (1=Clique).
    /// - `wheel`: Deslocamento da roda (0=Nenhum, 1=Cima, 0xFF=Baixo).
    pub fn new_mouse_remap(source_index: u8, button: u8, action_type: u8, wheel: u8) -> Self {
        Self {
            source_index,
            action: RemapAction::Mouse,
            param1: button,
            param2: action_type,
            param3: wheel,
        }
    }
}

impl Device {
    /// Transmite a tabela completa de 576 bytes contendo os mapeamentos fornecidos
    /// para a camada normal (`fn_layer = false`) ou camada FN (`fn_layer = true`).
    pub fn send_remap_table(&mut self, remaps: &[KeyRemap], fn_layer: bool) -> Result<()> {
        self.begin_transaction()
            .context("Falha no begin_transaction do remapeamento")?;

        // Inicializacao da camada correspondente: 0x11 para camada Normal, 0x27 para camada FN
        let mut init = [0u8; REPORT_SIZE];
        init[0] = 0x04;
        init[1] = if fn_layer { 0x27 } else { 0x11 };
        init[8] = 0x09;

        self.send_command(&init, true)
            .context("Falha na inicializacao da tabela de remapeamento")?;

        // Montagem do buffer de 576 bytes (144 slots x 4 bytes)
        let mut buf = [0u8; REMAP_TABLE_SIZE];

        for r in remaps {
            let idx = r.source_index as usize;
            let offset = idx * 4;
            if offset + 3 < REMAP_TABLE_SIZE {
                buf[offset] = r.action as u8;
                buf[offset + 1] = r.param1;
                buf[offset + 2] = r.param2;
                buf[offset + 3] = r.param3;
            }
        }

        // Trailer nos dois ultimos bytes: 0xAA, 0x55 (little-endian de 0x55AA exigido pelo firmware de remap)
        buf[574] = 0xAA;
        buf[575] = 0x55;

        // Envio segmentado em blocos de 64 bytes sem readback a cada pacote intermediario
        self.send_multi_packet(&buf, false)
            .context("Falha ao transmitir tabela de remapeamento via multi-pacote")?;

        self.apply_transaction()
            .context("Falha ao aplicar tabela de remapeamento")?;

        // Finalizacao e persistencia na flash com readback
        self.send_command(&[0x04, 0xF0], true)
            .context("Falha ao persistir tabela de remapeamento na flash")?;

        Ok(())
    }

    /// Grava a lista de remapeamentos na camada de teclas padrao (Base / Normal Layer).
    pub fn set_key_remap(&mut self, remaps: &[KeyRemap]) -> Result<()> {
        self.send_remap_table(remaps, false)
    }

    /// Grava a lista de remapeamentos na camada de teclas de funcao (FN Layer).
    pub fn set_fn_key_remap(&mut self, remaps: &[KeyRemap]) -> Result<()> {
        self.send_remap_table(remaps, true)
    }

    /// Restaura a camada padrao de teclas para os valores originais de fabrica (limpa remapeamentos).
    pub fn reset_key_remap(&mut self) -> Result<()> {
        self.set_key_remap(&[])
    }

    /// Restaura a camada de funcao FN para os valores originais de fabrica (limpa remapeamentos).
    pub fn reset_fn_key_remap(&mut self) -> Result<()> {
        self.set_fn_key_remap(&[])
    }
}

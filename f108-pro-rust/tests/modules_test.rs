//! Testes de integracao e logica de modulos usando transporte simulado (mock).
//!
//! Este modulo verifica se a implementacao da API do dispositivo constroi e envia os
//! comandos USB (Feature Reports e streaming para LCD) rigorosamente de acordo 
//! com o protocolo exigido pelo firmware do teclado Aula F108 Pro (Sonix/Microdia), 
//! sem necessidade de ter hardware real conectado ao barramento USB durante a execucao dos testes.

use std::sync::{Arc, Mutex};
use std::time::Duration;
use anyhow::Result;
use chrono::TimeZone;

use aula::aula::constants::*;
use aula::aula::device::Device;
use aula::aula::transport::{Transport, REPORT_SIZE};
use aula::aula::lighting::LightingConfig;
use aula::aula::perkey::KeyColor;
use aula::aula::remap::KeyRemap;
use aula::aula::lcd::LCD_PAGE_SIZE;

/// Implementacao simulada de `Transport` (`MockTransport`) projetada para interceptar e armazenar
/// as transacoes geradas pelos metodos do teclado em vetores de memoria concorrente.
/// Isso permite validar a exatidao dos opcodes e da ordem de pacotes gerados pela logica do driver.
#[derive(Default, Clone)]
struct MockTransport {
    pub feature_reports_sent: Arc<Mutex<Vec<[u8; REPORT_SIZE]>>>,
    pub lcd_pages_sent: Arc<Mutex<Vec<Vec<u8>>>>,
    pub lcd_inited: Arc<Mutex<bool>>,
}

impl Transport for MockTransport {
    fn set_feature_report(&mut self, data: &[u8; REPORT_SIZE]) -> Result<()> {
        self.feature_reports_sent.lock().unwrap().push(*data);
        Ok(())
    }

    fn get_feature_report(&mut self) -> Result<[u8; REPORT_SIZE]> {
        // Retorna confirmacao padrao (readback)
        Ok([0u8; REPORT_SIZE])
    }

    fn init_lcd(&mut self) -> Result<()> {
        *self.lcd_inited.lock().unwrap() = true;
        Ok(())
    }

    fn write_lcd_page(&mut self, data: &[u8]) -> Result<()> {
        self.lcd_pages_sent.lock().unwrap().push(data.to_vec());
        Ok(())
    }

    fn read_lcd_ack(&mut self, _timeout: Duration) -> Result<Vec<u8>> {
        Ok(vec![0x04, 0x00])
    }
}

#[test]
fn test_lighting_config_validation_and_protocol() {
    let mock = MockTransport::default();
    let reports = mock.feature_reports_sent.clone();
    let mut dev = Device::from_transport(Box::new(mock));

    // 1. Validacao de erro
    let mut invalid_cfg = LightingConfig::default();
    invalid_cfg.brightness = 6;
    assert!(dev.set_lighting(&invalid_cfg).is_err());

    invalid_cfg.brightness = 3;
    invalid_cfg.speed = 7;
    assert!(dev.set_lighting(&invalid_cfg).is_err());

    // 2. Envio de configuracao valida
    let cfg = LightingConfig {
        mode: LightingMode::Breath,
        r: 0x12,
        g: 0x34,
        b: 0x56,
        brightness: 4,
        speed: 2,
        direction: 1,
        colorful: false,
    };
    assert!(dev.set_lighting(&cfg).is_ok());

    let sent = reports.lock().unwrap();
    // Sequencia esperada:
    // 0: begin_transaction [0x04, 0x18]
    // 1: lighting_init [0x04, 0x13, ..., byte 8 = 0x01]
    // 2: payload 64 bytes (mode=7, r=0x12, g=0x34, b=0x56, colorful=0, br=4, sp=2, dir=1, trailer=0x55 0xAA)
    // 3: apply_transaction [0x04, 0x02]
    // 4: finalize_transaction [0x04, 0xF0]
    assert_eq!(sent.len(), 5);
    assert_eq!(&sent[0][..2], &[0x04, 0x18]);
    assert_eq!(&sent[1][..2], &[0x04, 0x13]);
    assert_eq!(sent[1][8], 0x01);

    let p = &sent[2];
    assert_eq!(p[0], LightingMode::Breath as u8);
    assert_eq!(p[1], 0x12);
    assert_eq!(p[2], 0x34);
    assert_eq!(p[3], 0x56);
    assert_eq!(p[8], 0x00); // colorful false
    assert_eq!(p[9], 4);    // brightness
    assert_eq!(p[10], 2);   // speed
    assert_eq!(p[11], 1);   // direction
    assert_eq!(p[14], 0x55);
    assert_eq!(p[15], 0xAA);

    assert_eq!(&sent[3][..2], &[0x04, 0x02]);
    assert_eq!(&sent[4][..2], &[0x04, 0xF0]);
}

#[test]
fn test_clock_sync_protocol() {
    let mock = MockTransport::default();
    let reports = mock.feature_reports_sent.clone();
    let mut dev = Device::from_transport(Box::new(mock));

    // Data de teste: 2026-09-23 11:45:30 (Quarta-feira -> num_days_from_sunday = 3)
    let dt = chrono::Local.with_ymd_and_hms(2026, 9, 23, 11, 45, 30).unwrap();
    assert!(dev.sync_clock(dt).is_ok());

    let sent = reports.lock().unwrap();
    // Sequencia:
    // 0: begin_transaction [0x04, 0x18]
    // 1: clock_init [0x04, 0x28, byte 8 = 0x01]
    // 2: clock_data (profile 1, magic 0x5A, year=26, month=9, day=23, hour=11, min=45, sec=30, weekday=3, trailer 0x55 0xAA nos bytes 62..64)
    // 3: apply_transaction [0x04, 0x02] (sem finalize!)
    assert_eq!(sent.len(), 4);
    assert_eq!(&sent[0][..2], &[0x04, 0x18]);
    assert_eq!(&sent[1][..2], &[0x04, 0x28]);
    assert_eq!(sent[1][8], 0x01);

    let d = &sent[2];
    assert_eq!(d[0], 0x00);
    assert_eq!(d[1], 0x01);
    assert_eq!(d[2], MAGIC_CLOCK_MARKER);
    assert_eq!(d[3], 26);
    assert_eq!(d[4], 9);
    assert_eq!(d[5], 23);
    assert_eq!(d[6], 11);
    assert_eq!(d[7], 45);
    assert_eq!(d[8], 30);
    assert_eq!(d[10], 3); // quarta-feira = 3 dias apos domingo
    assert_eq!(d[62], 0x55);
    assert_eq!(d[63], 0xAA);

    assert_eq!(&sent[3][..2], &[0x04, 0x02]);
}

#[test]
fn test_per_key_rgb_protocol() {
    let mock = MockTransport::default();
    let reports = mock.feature_reports_sent.clone();
    let mut dev = Device::from_transport(Box::new(mock));

    let keys = vec![
        KeyColor::new(KEY_ESC, 0xFF, 0x00, 0x00),
        KeyColor::new(KEY_SPACE, 0x00, 0xFF, 0x00),
    ];

    assert!(dev.set_per_key_rgb(&keys, 5).is_ok());

    let sent = reports.lock().unwrap();
    // Preamble:
    // 0: begin [0x04, 0x18]
    // 1: light_init [0x04, 0x13, byte 8 = 0x01]
    // 2: strip data [0x80, br=5, trailer 0x55 0xAA em 14..16]
    // 3: apply [0x04, 0x02]
    // 4: finalize [0x04, 0xF0]
    assert_eq!(&sent[0][..2], &[0x04, 0x18]);
    assert_eq!(&sent[1][..2], &[0x04, 0x13]);
    assert_eq!(sent[2][0], 0x80);
    assert_eq!(sent[2][9], 5);
    assert_eq!(sent[2][14], 0x55);
    assert_eq!(sent[2][15], 0xAA);
    assert_eq!(&sent[3][..2], &[0x04, 0x02]);
    assert_eq!(&sent[4][..2], &[0x04, 0xF0]);

    // Per-Key phase:
    // 5: begin [0x04, 0x18]
    // 6: init perkey [0x04, 0x23, byte 8 = 0x09]
    // 7..=15: 9 pacotes multi-packet de 64 bytes (576 bytes no total)
    // 16: apply [0x04, 0x02]
    // 17: finalize with readback [0x04, 0xF0]
    assert_eq!(&sent[5][..2], &[0x04, 0x18]);
    assert_eq!(&sent[6][..2], &[0x04, 0x23]);
    assert_eq!(sent[6][8], 0x09);

    // Multi-packet: 576 / 64 = 9 pacotes
    // O primeiro pacote contem KEY_ESC (idx 1 -> offset 4..8)
    let p0 = &sent[7];
    assert_eq!(p0[4], KEY_ESC);
    assert_eq!(p0[5], 0xFF);
    assert_eq!(p0[6], 0x00);
    assert_eq!(p0[7], 0x00);

    // O ultimo pacote do multi-packet (indice 15) contem o trailer em 574..576 (offset 62..64 do nono pacote)
    let p_last = &sent[15];
    assert_eq!(p_last[62], 0x55);
    assert_eq!(p_last[63], 0xAA);

    assert_eq!(&sent[16][..2], &[0x04, 0x02]);
    assert_eq!(&sent[17][..2], &[0x04, 0xF0]);
}

#[test]
fn test_remap_protocol() {
    let mock = MockTransport::default();
    let reports = mock.feature_reports_sent.clone();
    let mut dev = Device::from_transport(Box::new(mock));

    let remaps = vec![
        KeyRemap::new_key_swap(KEY_CAPSLOCK, 0x29), // CapsLock -> ESC
        KeyRemap::new_consumer_remap(KEY_F1, 0xE2), // F1 -> Mute
    ];

    // Testa Base layer
    assert!(dev.set_key_remap(&remaps).is_ok());

    let sent = reports.lock().unwrap();
    // 0: begin [0x04, 0x18]
    // 1: init normal [0x04, 0x11, byte 8 = 0x09]
    // 2..=10: 9 pacotes de 64 bytes com trailer 0xAA, 0x55
    // 11: apply [0x04, 0x02]
    // 12: finalize [0x04, 0xF0]
    assert_eq!(&sent[0][..2], &[0x04, 0x18]);
    assert_eq!(&sent[1][..2], &[0x04, 0x11]);
    assert_eq!(sent[1][8], 0x09);

    let p_last = &sent[10];
    assert_eq!(p_last[62], 0xAA); // Little-endian 0x55AA
    assert_eq!(p_last[63], 0x55);

    assert_eq!(&sent[11][..2], &[0x04, 0x02]);
    assert_eq!(&sent[12][..2], &[0x04, 0xF0]);
}

#[test]
fn test_fn_remap_protocol() {
    let mock = MockTransport::default();
    let reports = mock.feature_reports_sent.clone();
    let mut dev = Device::from_transport(Box::new(mock));

    assert!(dev.reset_fn_key_remap().is_ok());

    let sent = reports.lock().unwrap();
    // 0: begin [0x04, 0x18]
    // 1: init fn [0x04, 0x27, byte 8 = 0x09]
    assert_eq!(&sent[0][..2], &[0x04, 0x18]);
    assert_eq!(&sent[1][..2], &[0x04, 0x27]);
    assert_eq!(sent[1][8], 0x09);
}

#[test]
fn test_lcd_upload_protocol() {
    let mock = MockTransport::default();
    let reports = mock.feature_reports_sent.clone();
    let lcd_pages = mock.lcd_pages_sent.clone();
    let lcd_inited = mock.lcd_inited.clone();
    let mut dev = Device::from_transport(Box::new(mock));

    // 1. Buffer vazio deve falhar
    assert!(dev.upload_lcd_image(&[], 0, |_, _| {}).is_err());

    // 2. Buffer com tamanho nao multiplo de 4096 deve falhar
    assert!(dev.upload_lcd_image(&[0u8; 100], 0, |_, _| {}).is_err());

    // 3. Buffer de 2 paginas (8192 bytes)
    let buffer = vec![0x42u8; LCD_PAGE_SIZE * 2];
    let mut progress_calls = Vec::new();

    assert!(dev.upload_lcd_image(&buffer, 1, |cur, total| {
        progress_calls.push((cur, total));
    }).is_ok());

    assert!(*lcd_inited.lock().unwrap());
    assert_eq!(progress_calls, vec![(1, 2), (2, 2)]);
    assert_eq!(lcd_pages.lock().unwrap().len(), 2);

    let sent = reports.lock().unwrap();
    // 0: begin [0x04, 0x18]
    // 1: lcd header [0x04, 0x72, img=1, ..., page_count low=2, high=0]
    // 2: apply [0x04, 0x02]
    assert_eq!(&sent[0][..2], &[0x04, 0x18]);
    assert_eq!(&sent[1][..3], &[0x04, 0x72, 0x01]);
    assert_eq!(sent[1][8], 0x02);
    assert_eq!(sent[1][9], 0x00);
    assert_eq!(&sent[2][..2], &[0x04, 0x02]);
}

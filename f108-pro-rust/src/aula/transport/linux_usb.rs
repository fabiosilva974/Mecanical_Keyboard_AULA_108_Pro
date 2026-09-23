//! Implementacao do transporte USB no Linux utilizando rusb (libusb-1.0).
//!
//! Gerencia o ciclo de vida da conexao USB com o hardware Aula F108 Pro:
//! - Detach automatico do driver do kernel (hid-generic / usbhid).
//! - Reclamacao (claim) da Interface 3 para relatorios HID Feature (64 bytes).
//! - Reclamacao sob demanda (lazy claim) da Interface 2 para streaming LCD.
//! - Suporte tanto a cabo USB direto (0c45:800a) quanto ao dongle 2.4G (05ac:024f).

use std::time::Duration;
use rusb::{Context, DeviceHandle, UsbContext};
use anyhow::{anyhow, Context as AnyhowContext, Result};

use super::{Transport, REPORT_SIZE};

/// Vendor ID oficial (Sonix/Microdia) para conexao via cabo USB.
pub const VENDOR_ID_WIRED: u16 = 0x0C45;
/// Product ID oficial para o teclado AULA F108 Pro.
pub const PRODUCT_ID_WIRED: u16 = 0x800A;

/// Vendor ID do Dongle sem fio 2.4 GHz.
pub const VENDOR_ID_WIRELESS: u16 = 0x05AC;
/// Product ID do Dongle sem fio 2.4 GHz.
pub const PRODUCT_ID_WIRELESS: u16 = 0x024F;

/// Interface USB 3: Canal HID de controle e comandos Feature (64 bytes).
pub const INTERFACE_CONTROL: u8 = 3;
/// Interface USB 2: Canal dedicado a transferencia de dados do LCD.
pub const INTERFACE_LCD: u8 = 2;

/// Endpoint OUT do LCD (Interface 2): Envio de paginas de 4096 bytes.
pub const LCD_OUT_EP: u8 = 0x03;
/// Endpoint IN do LCD (Interface 2): Leitura de ACKs de 64 bytes (0x80 | 4).
pub const LCD_IN_EP: u8 = 0x84;

/// Constantes do protocolo de controle USB HID
const REQ_TYPE_OUT: u8 = 0x21; // Host-to-Device | Class | Interface
const REQ_TYPE_IN: u8 = 0xA1;  // Device-to-Host | Class | Interface
const REQ_SET_REPORT: u8 = 0x09;
const REQ_GET_REPORT: u8 = 0x01;
const FEATURE_REPORT_TYPE: u16 = 0x0300; // Tipo 0x03 (Feature) no byte superior

/// Estrutura de transporte USB para Linux implementando `Transport`.
pub struct LinuxUsbTransport {
    _context: Context,
    handle: DeviceHandle<Context>,
    lcd_initialized: bool,
}

impl LinuxUsbTransport {
    /// Tenta localizar e abrir o dispositivo Aula F108 Pro (via cabo ou dongle sem fio).
    pub fn open() -> Result<Self> {
        let context = Context::new().context("Falha ao inicializar contexto libusb (rusb)")?;
        
        // 1. Tenta abrir via cabo USB direto (0c45:800a)
        let handle_opt = match context.open_device_with_vid_pid(VENDOR_ID_WIRED, PRODUCT_ID_WIRED) {
            Some(h) => Some(h),
            None => {
                // 2. Se nao encontrou no cabo, tenta abrir pelo dongle sem fio (05ac:024f)
                context.open_device_with_vid_pid(VENDOR_ID_WIRELESS, PRODUCT_ID_WIRELESS)
            }
        };

        let handle = handle_opt.ok_or_else(|| {
            anyhow!(
                "Teclado Aula F108 Pro nao encontrado. Verifique se o cabo (VID: {:04x}, PID: {:04x}) \
                 ou o dongle (VID: {:04x}, PID: {:04x}) estao conectados e se as regras udev foram aplicadas.",
                VENDOR_ID_WIRED, PRODUCT_ID_WIRED, VENDOR_ID_WIRELESS, PRODUCT_ID_WIRELESS
            )
        })?;

        // Ativa auto-detach do driver de kernel para que possamos controlar os endpoints HID diretamente
        let _ = handle.set_auto_detach_kernel_driver(true);

        // Se o kernel estiver associado a interface 3 e o auto-detach nao tiver liberado, forca detach
        if let Ok(true) = handle.kernel_driver_active(INTERFACE_CONTROL) {
            let _ = handle.detach_kernel_driver(INTERFACE_CONTROL);
        }

        // Reclama a Interface 3 para transmissao de Feature Reports
        handle.claim_interface(INTERFACE_CONTROL)
            .context(format!("Falha ao reivindicar (claim) interface de controle HID {}", INTERFACE_CONTROL))?;

        Ok(Self {
            _context: context,
            handle,
            lcd_initialized: false,
        })
    }
}

impl Transport for LinuxUsbTransport {
    fn set_feature_report(&mut self, data: &[u8; REPORT_SIZE]) -> Result<()> {
        let timeout = Duration::from_millis(1000);
        let written = self.handle.write_control(
            REQ_TYPE_OUT,
            REQ_SET_REPORT,
            FEATURE_REPORT_TYPE,
            INTERFACE_CONTROL as u16,
            data,
            timeout,
        ).context("Erro ao executar SET_REPORT na Interface 3")?;

        if written != REPORT_SIZE {
            return Err(anyhow!("SET_REPORT enviou {} bytes, esperado {}", written, REPORT_SIZE));
        }

        Ok(())
    }

    fn get_feature_report(&mut self) -> Result<[u8; REPORT_SIZE]> {
        let mut buf = [0u8; REPORT_SIZE];
        let timeout = Duration::from_millis(1000);
        let read = self.handle.read_control(
            REQ_TYPE_IN,
            REQ_GET_REPORT,
            FEATURE_REPORT_TYPE,
            INTERFACE_CONTROL as u16,
            &mut buf,
            timeout,
        ).context("Erro ao executar GET_REPORT na Interface 3")?;

        if read != REPORT_SIZE {
            return Err(anyhow!("GET_REPORT recebeu {} bytes, esperado {}", read, REPORT_SIZE));
        }

        Ok(buf)
    }

    fn init_lcd(&mut self) -> Result<()> {
        if self.lcd_initialized {
            return Ok(());
        }

        // Libera driver de kernel se estiver ocupando a Interface 2 (LCD)
        if let Ok(true) = self.handle.kernel_driver_active(INTERFACE_LCD) {
            let _ = self.handle.detach_kernel_driver(INTERFACE_LCD);
        }

        self.handle.claim_interface(INTERFACE_LCD)
            .context(format!("Falha ao reivindicar (claim) interface LCD {}", INTERFACE_LCD))?;

        self.lcd_initialized = true;
        Ok(())
    }

    fn write_lcd_page(&mut self, data: &[u8]) -> Result<()> {
        if !self.lcd_initialized {
            self.init_lcd()?;
        }

        let timeout = Duration::from_millis(1000);
        // O envio de paginas para o LCD ocorre atraves de interrupcao OUT no EP 3
        self.handle.write_interrupt(LCD_OUT_EP, data, timeout)
            .context("Falha ao gravar pagina LCD via EP 3 OUT")?;

        Ok(())
    }

    fn read_lcd_ack(&mut self, timeout: Duration) -> Result<Vec<u8>> {
        if !self.lcd_initialized {
            self.init_lcd()?;
        }

        let mut buf = vec![0u8; REPORT_SIZE];
        match self.handle.read_interrupt(LCD_IN_EP, &mut buf, timeout) {
            Ok(bytes_read) => {
                buf.truncate(bytes_read);
                Ok(buf)
            }
            Err(e) => {
                // Timeout em leitura de ACK do display nao deve quebrar o streaming
                Err(anyhow::Error::new(e))
            }
        }
    }
}

impl Drop for LinuxUsbTransport {
    fn drop(&mut self) {
        if self.lcd_initialized {
            let _ = self.handle.release_interface(INTERFACE_LCD);
        }
        let _ = self.handle.release_interface(INTERFACE_CONTROL);
    }
}

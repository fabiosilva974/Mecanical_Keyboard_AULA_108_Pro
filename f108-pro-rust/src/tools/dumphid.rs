//! Ferramenta de diagnostico para inspecao de descritores HID do teclado Aula F108 Pro.
//!
//! Esta ferramenta conecta-se diretamente ao teclado via `rusb` nos modos cabeado (`0c45:800a`)
//! ou receptor 2.4GHz (`05ac:024f`), itera sobre as interfaces 0 a 4 e envia a solicitacao de
//! controle padrao USB `GET_DESCRIPTOR` para descritor de relatorio HID (`0x2200`).
//!
//! Em seguida, formata o dump hexadecimal de 16 em 16 bytes e analisa os itens criticos:
//! - Usage Pages (0x05 / 0x06)
//! - Report IDs (0x85)
//! - Feature items (0xB1 / 0xB2)

use std::time::Duration;
use anyhow::{bail, Context, Result};
use rusb::{Context as UsbContext, DeviceHandle, UsbContext as _};

const VENDOR_ID_WIRED: u16 = 0x0C45;
const PRODUCT_ID_WIRED: u16 = 0x800A;
const VENDOR_ID_WIRELESS: u16 = 0x05AC;
const PRODUCT_ID_WIRELESS: u16 = 0x024F;

/// Identifica nome amigavel para as principais Usage Pages da especificacao USB HID.
fn usage_page_name(page: u16) -> &'static str {
    match page {
        0x01 => "Generic Desktop Controls (Teclado, Mouse, Joystick)",
        0x02 => "Simulation Controls",
        0x03 => "VR Controls",
        0x04 => "Sport Controls",
        0x05 => "Game Controls",
        0x06 => "Generic Device Controls",
        0x07 => "Keyboard / Keypad (Teclas HID)",
        0x08 => "LEDs (NumLock, CapsLock, ScrollLock)",
        0x09 => "Button",
        0x0A => "Ordinal",
        0x0B => "Telephony Device",
        0x0C => "Consumer (Controle Multimidia / Audio)",
        0x0D => "Digitizers",
        0x0E => "Haptics",
        0x0F => "Physical Input Device",
        0x10 => "Unicode",
        0x14 => "Auxiliary Display",
        0x20 => "Sensors",
        0x80..=0x83 => "Monitor / Power Device",
        0xFF00..=0xFFFF => "Vendor-Defined (Proprietario Sonix / Microdia)",
        _ => "Reservado / Nao documentado",
    }
}

/// Encontra e abre o handle para o dispositivo de teclado Aula F108 Pro.
fn open_keyboard_device(context: &UsbContext) -> Result<DeviceHandle<UsbContext>> {
    let devices = context.devices().context("Falha ao enumerar dispositivos USB")?;

    for device in devices.iter() {
        let desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        let vid = desc.vendor_id();
        let pid = desc.product_id();

        if (vid == VENDOR_ID_WIRED && pid == PRODUCT_ID_WIRED)
            || (vid == VENDOR_ID_WIRELESS && pid == PRODUCT_ID_WIRELESS)
        {
            println!(
                "Dispositivo compativel encontrado: VID=0x{:04X} PID=0x{:04X} (Bus {:03}, Dev {:03})",
                vid,
                pid,
                device.bus_number(),
                device.address()
            );

            match device.open() {
                Ok(handle) => {
                    let _ = handle.set_auto_detach_kernel_driver(true);
                    return Ok(handle);
                }
                Err(err) => {
                    eprintln!("Aviso: Falha ao abrir dispositivo {:04X}:{:04X}: {}", vid, pid, err);
                }
            }
        }
    }

    bail!(
        "Nenhum teclado Aula F108 Pro encontrado (procurado 0c45:800a ou 05ac:024f). Verifique permissoes udev ou execute com sudo."
    );
}

/// Imprime o dump hexadecimal formatado de 16 em 16 bytes acompanhado de caracteres ASCII.
fn print_hex_dump(data: &[u8]) {
    for (i, chunk) in data.chunks(16).enumerate() {
        let offset = i * 16;
        let mut hex_part = String::new();
        let mut ascii_part = String::new();

        for (j, byte) in chunk.iter().enumerate() {
            if j == 8 {
                hex_part.push(' ');
            }
            hex_part.push_str(&format!("{:02X} ", byte));

            if byte.is_ascii_graphic() || *byte == b' ' {
                ascii_part.push(*byte as char);
            } else {
                ascii_part.push('.');
            }
        }

        // Alinhamento caso o ultimo chunk tenha menos de 16 bytes
        let pad_len = if chunk.len() <= 8 {
            (16 - chunk.len()) * 3 + 1
        } else {
            (16 - chunk.len()) * 3
        };

        println!(
            "  {:04X}: {:<width$} |{}|",
            offset,
            hex_part,
            ascii_part,
            width = 48 + 1 + pad_len
        );
    }
}

/// Analisa os bytes do descritor HID e extrai Report IDs, Feature Items e Usage Pages.
fn parse_hid_descriptor(data: &[u8]) {
    let mut offset = 0;
    let mut usage_pages = Vec::new();
    let mut report_ids = Vec::new();
    let mut feature_count = 0;

    while offset < data.len() {
        let prefix = data[offset];

        // Item longo (0xFE)
        if prefix == 0xFE {
            if offset + 2 < data.len() {
                let data_len = data[offset + 1] as usize;
                offset += 3 + data_len;
            } else {
                break;
            }
            continue;
        }

        // Itens curtos: Tag (4b), Type (2b), Size (2b)
        let b_size = prefix & 0x03;
        let item_len = match b_size {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 4,
            _ => 0,
        };

        if offset + 1 + item_len > data.len() {
            break;
        }

        let item_data = &data[offset + 1..offset + 1 + item_len];

        // 1. Usage Page: prefix 0x05 (1 byte) ou 0x06 (2 bytes)
        if prefix == 0x05 && !item_data.is_empty() {
            let page = item_data[0] as u16;
            usage_pages.push(page);
        } else if prefix == 0x06 && item_data.len() >= 2 {
            let page = (item_data[0] as u16) | ((item_data[1] as u16) << 8);
            usage_pages.push(page);
        }

        // 2. Report ID: prefix 0x85 (1 byte)
        if prefix == 0x85 && !item_data.is_empty() {
            report_ids.push(item_data[0]);
        }

        // 3. Feature: prefix 0xB1 (1 byte) ou 0xB2 (2 bytes)
        if prefix == 0xB1 || prefix == 0xB2 {
            feature_count += 1;
        }

        offset += 1 + item_len;
    }

    // Exibicao dos resultados sumarizados
    println!("  -> Sumario do Descritor HID:");

    if !usage_pages.is_empty() {
        println!("     Usage Pages detectadas (0x05 / 0x06):");
        for page in &usage_pages {
            println!("       - 0x{:04X}: {}", page, usage_page_name(*page));
        }
    } else {
        println!("     Usage Pages: nenhuma encontrada.");
    }

    if !report_ids.is_empty() {
        println!("     Report IDs declarados (0x85):");
        for id in &report_ids {
            println!("       - Report ID: 0x{:02X} ({})", id, id);
        }
    } else {
        println!("     Report IDs (0x85): nenhum explicito (Report ID 0 padrao).");
    }

    println!("     Itens Feature declarados (0xB1 / 0xB2): {}", feature_count);
}

fn main() -> Result<()> {
    println!("============================================================");
    println!("   AULA F108 Pro - HID Descriptor Dumper (dumphid)");
    println!("============================================================");

    let context = UsbContext::new().context("Falha ao inicializar contexto USB libusb")?;
    let handle = open_keyboard_device(&context)?;

    let mut buf = [0u8; 1024];

    for iface in 0..=4u8 {
        println!("\n------------------------------------------------------------");
        println!("Consultando Interface USB {}...", iface);

        // GET_DESCRIPTOR: bmRequestType = 0x81 (In/Standard/Interface)
        // bRequest = 0x06 (GET_DESCRIPTOR)
        // wValue = 0x2200 (Descriptor Type 0x22 = HID Report Descriptor, Index 0)
        // wIndex = iface (Numero da interface)
        match handle.read_control(
            0x81,
            0x06,
            0x2200,
            iface as u16,
            &mut buf,
            Duration::from_millis(800),
        ) {
            Ok(bytes_read) if bytes_read > 0 => {
                println!(
                    "Interface {}: Descritor HID recebido com sucesso ({} bytes)",
                    iface, bytes_read
                );
                let desc = &buf[..bytes_read];
                print_hex_dump(desc);
                println!();
                parse_hid_descriptor(desc);
            }
            Ok(_) => {
                println!("Interface {}: Respondeu 0 bytes (sem relatorio HID associado).", iface);
            }
            Err(err) => {
                println!(
                    "Interface {}: Sem descritor de relatorio HID ou requisicao rejeitada ({})",
                    iface, err
                );
            }
        }
    }

    println!("\n============================================================");
    println!("Inspecao concluida.");
    Ok(())
}

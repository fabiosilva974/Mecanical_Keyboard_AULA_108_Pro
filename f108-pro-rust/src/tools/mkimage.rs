//! Ferramenta de geracao de arquivos binarios para a tela LCD colorida do Aula F108 Pro.
//!
//! A tela do Aula F108 Pro possui resolucao de 240x135 pixels em formato RGB565 little-endian.
//! Esta ferramenta empacota imagens estaticas (cores solidas) ou animacoes GIF em um arquivo `.bin`
//! compativel com o firmware do teclado e alinhado em paginas de 4096 bytes (4 KB).
//!
//! # Estrutura do Arquivo Binario:
//! - Cabecalho de 256 bytes:
//!   - `header[0]`: Quantidade de quadros (`num_frames`, maximo 255)
//!   - `header[1..=num_frames]`: Delay de cada quadro em centissegundos (1 centissegundo = 10ms)
//!   - `header[1 + num_frames .. 256]`: Preenchido com `0xFF`
//! - Dados de imagem:
//!   - Sequencia de quadros, cada um contendo exatamente `240 * 135 * 2 = 64800` bytes em RGB565 little-endian
//! - Padding final:
//!   - Alinhamento obrigatorio para multiplo de 4096 bytes com bytes `0xFF`.

use std::fs::{self, File};
use std::path::Path;
use anyhow::{bail, Context, Result};
use clap::Parser;
use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, DynamicImage};

const LCD_WIDTH: u32 = 240;
const LCD_HEIGHT: u32 = 135;
const FRAME_PIXELS: usize = (LCD_WIDTH * LCD_HEIGHT) as usize; // 32.400 pixels
const BYTES_PER_FRAME: usize = FRAME_PIXELS * 2;              // 64.800 bytes
const HEADER_SIZE: usize = 256;
const PAGE_ALIGNMENT: usize = 4096;

#[derive(Parser, Debug)]
#[command(
    name = "mkimage",
    author = "fjorge",
    version = "0.1.0",
    about = "Gera arquivos binarios compativeis com o visor LCD TFT (240x135) do teclado Aula F108 Pro"
)]
struct Args {
    /// Arquivo binario de saida
    #[arg(short = 'o', long = "output", default_value = "lcd-image.bin")]
    output: String,

    /// Canal Vermelho (Red, 0..=255) para geracao de quadro de cor solida
    #[arg(short = 'r', long = "red")]
    red: Option<u8>,

    /// Canal Verde (Green, 0..=255) para geracao de quadro de cor solida
    #[arg(short = 'g', long = "green")]
    green: Option<u8>,

    /// Canal Azul (Blue, 0..=255) para geracao de quadro de cor solida
    #[arg(short = 'b', long = "blue")]
    blue: Option<u8>,

    /// Cor solida em formato hexadecimal (ex: "FF00FF" ou "#00FF00")
    #[arg(long = "hex")]
    hex_color: Option<String>,

    /// Arquivo de animacao GIF para conversao de quadros
    #[arg(long = "gif")]
    gif: Option<String>,
}

/// Converte componentes RGB888 (0..=255) para valor de 16 bits no padrao RGB565.
#[inline(always)]
fn rgb888_to_rgb565(r: u8, g: u8, b: u8) -> u16 {
    (((r as u16) >> 3) << 11) | (((g as u16) >> 2) << 5) | ((b as u16) >> 3)
}

/// Interpreta string hexadecimal como componentes (R, G, B).
fn parse_hex(hex: &str) -> Result<(u8, u8, u8)> {
    let s = hex.trim().trim_start_matches('#');
    if s.len() != 6 {
        bail!("Codigo hexadecimal deve conter 6 caracteres: '{}'", hex);
    }
    let r = u8::from_str_radix(&s[0..2], 16).context("Erro no componente R do hex")?;
    let g = u8::from_str_radix(&s[2..4], 16).context("Erro no componente G do hex")?;
    let b = u8::from_str_radix(&s[4..6], 16).context("Erro no componente B do hex")?;
    Ok((r, g, b))
}

/// Estrutura interna para representar um quadro com seu delay em centissegundos.
struct FrameData {
    delay_cs: u8,
    bytes: Vec<u8>,
}

/// Decodifica um arquivo GIF e converte seus quadros para a resolucao de 240x135 em RGB565.
fn process_gif(path: &str) -> Result<Vec<FrameData>> {
    let file = File::open(Path::new(path))
        .with_context(|| format!("Nao foi possivel abrir o arquivo GIF: {}", path))?;

    let decoder = GifDecoder::new(file)
        .context("Falha ao inicializar o decodificador GIF")?;

    let raw_frames = decoder.into_frames().collect_frames()
        .context("Falha ao extrair quadros da animacao GIF")?;

    if raw_frames.is_empty() {
        bail!("O arquivo GIF informado nao contem quadros validos.");
    }

    let mut out_frames = Vec::new();
    let num_frames = raw_frames.len().min(255);

    if raw_frames.len() > 255 {
        eprintln!(
            "Aviso: O GIF possui {} quadros, mas o cabecalho suporta no maximo 255. Truncando para 255 quadros.",
            raw_frames.len()
        );
    }

    println!("Processando {} quadros do GIF...", num_frames);

    for (idx, frame) in raw_frames.into_iter().take(num_frames).enumerate() {
        // Calcula delay em centissegundos (1 centissegundo = 10ms)
        let (numer, denom) = frame.delay().numer_denom_ms();
        let ms = if denom > 0 { numer / denom } else { 100 };
        let delay_cs = (ms / 10).clamp(1, 255) as u8;

        let rgba_buf = frame.into_buffer();
        let dynamic_img = DynamicImage::ImageRgba8(rgba_buf);

        // Redimensiona o quadro caso nao coincida com a resolucao do LCD (240x135)
        let rgb_img = if dynamic_img.width() != LCD_WIDTH || dynamic_img.height() != LCD_HEIGHT {
            dynamic_img.resize_exact(LCD_WIDTH, LCD_HEIGHT, image::imageops::FilterType::Triangle).to_rgb8()
        } else {
            dynamic_img.to_rgb8()
        };

        let mut frame_bytes = Vec::with_capacity(BYTES_PER_FRAME);

        for y in 0..LCD_HEIGHT {
            for x in 0..LCD_WIDTH {
                let p = rgb_img.get_pixel(x, y);
                let rgb565 = rgb888_to_rgb565(p[0], p[1], p[2]);
                frame_bytes.extend_from_slice(&rgb565.to_le_bytes());
            }
        }

        out_frames.push(FrameData {
            delay_cs,
            bytes: frame_bytes,
        });

        if (idx + 1) % 10 == 0 || idx + 1 == num_frames {
            println!("  Quadro {}/{} convertido com sucesso (delay: {} cs)", idx + 1, num_frames, delay_cs);
        }
    }

    Ok(out_frames)
}

/// Gera um quadro unico de cor solida com delay padrao de 1 segundo (100 cs).
fn process_solid_color(r: u8, g: u8, b: u8) -> Vec<FrameData> {
    let rgb565 = rgb888_to_rgb565(r, g, b);
    let le_bytes = rgb565.to_le_bytes();

    let mut frame_bytes = Vec::with_capacity(BYTES_PER_FRAME);
    for _ in 0..FRAME_PIXELS {
        frame_bytes.extend_from_slice(&le_bytes);
    }

    vec![FrameData {
        delay_cs: 100, // 100 centissegundos = 1000ms
        bytes: frame_bytes,
    }]
}

fn main() -> Result<()> {
    let args = Args::parse();

    let frames = if let Some(gif_path) = args.gif {
        process_gif(&gif_path)?
    } else {
        // Determinacao de cor solida
        let (r, g, b) = if let Some(hex_str) = args.hex_color {
            parse_hex(&hex_str)?
        } else if args.red.is_some() || args.green.is_some() || args.blue.is_some() {
            (
                args.red.unwrap_or(0),
                args.green.unwrap_or(0),
                args.blue.unwrap_or(0),
            )
        } else {
            println!("Nenhuma imagem ou cor especificada. Gerando quadro padrao preto (0, 0, 0).");
            (0, 0, 0)
        };

        println!("Gerando quadro estatico de cor solida: RGB({}, {}, {})", r, g, b);
        process_solid_color(r, g, b)
    };

    let num_frames = frames.len();
    if num_frames == 0 || num_frames > 255 {
        bail!("Quantidade de quadros invalida: {}", num_frames);
    }

    // 1. Constroi o cabecalho de 256 bytes
    let mut header = [0xFFu8; HEADER_SIZE];
    header[0] = num_frames as u8;

    for (i, frame) in frames.iter().enumerate() {
        header[1 + i] = frame.delay_cs;
    }

    // 2. Monta o buffer binario completo
    let mut output_data = Vec::new();
    output_data.extend_from_slice(&header);

    for frame in frames {
        output_data.extend_from_slice(&frame.bytes);
    }

    // 3. Aplica alinhamento para multiplo de 4096 bytes com bytes 0xFF
    let current_size = output_data.len();
    let remainder = current_size % PAGE_ALIGNMENT;
    if remainder != 0 {
        let padding = PAGE_ALIGNMENT - remainder;
        output_data.extend(std::iter::repeat(0xFF).take(padding));
    }

    let final_size = output_data.len();
    let total_pages = final_size / PAGE_ALIGNMENT;

    // 4. Salva o arquivo em disco
    fs::write(Path::new(&args.output), &output_data)
        .with_context(|| format!("Falha ao gravar arquivo binario de saida em: {}", args.output))?;

    println!("\n============================================================");
    println!("Arquivo gerado com sucesso: {}", args.output);
    println!("  Total de quadros: {}", num_frames);
    println!("  Tamanho final:    {} bytes ({} KB)", final_size, final_size / 1024);
    println!("  Paginas LCD (4KB): {} paginas", total_pages);
    println!("============================================================");

    Ok(())
}

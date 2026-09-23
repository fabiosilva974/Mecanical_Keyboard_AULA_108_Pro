//! Modulo CLI para interface de linha de comando do teclado Aula F108 Pro.
//!
//! Implementa os subcomandos disponiveis via `clap`:
//! - `Light`: Configuracao de efeitos RGB, velocidade, brilho e cores customizadas.
//! - `Brightness`: Ajuste rapido de nivel de brilho global (0 a 5).
//! - `Off`: Desativa a iluminacao RGB do teclado.
//! - `Modes`: Lista os 20 modos de iluminacao suportados pelo hardware.
//! - `Clock`: Sincroniza o relogio do visor TFT LCD com o horario do sistema operacional.
//! - `Keys`: Exibe todas as teclas fisicas mapeaveis organizadas visualmente por linhas.
//! - `Perkey`: Customizacao de cor individual por tecla (via YAML ou argumentos CLI).
//! - `Remap`: Remapeamento de funcoes e camadas Normal/FN (via YAML ou pares CLI).
//! - `Lcd`: Gravacao de arquivos binarios de imagem/animacao no visor colorido.

pub mod yaml_config;

use std::fs;
use std::io::{self, Write};
use std::path::Path;
use anyhow::{bail, Context, Result};
use chrono::Local;
use clap::{Parser, Subcommand};

use crate::aula::constants::{key_name_to_index, LightingMode};
use crate::aula::device::Device;
use crate::aula::lighting::LightingConfig;
use crate::aula::perkey::KeyColor;

#[derive(Parser, Debug)]
#[command(
    name = "f108-pro",
    author = "fjorge",
    version = "0.1.0",
    about = "Utilitario de controle para teclado mecanico Aula F108 Pro no Linux"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Configura modo e parametros de iluminacao RGB global
    Light {
        /// Nome do modo (ex: "Colourful", "Breath", "Static") ou indice numerico (0 a 19)
        mode: String,

        /// Nivel de brilho (0 a 5, opcional, padrao: 4)
        brightness: Option<u8>,

        /// Velocidade da animacao (0 a 5, opcional, padrao: 3)
        speed: Option<u8>,

        /// Cores RGB em componentes "R G B" (0..255) ou string hexadecimal "#RRGGBB"
        #[arg(num_args = 0..=3)]
        color: Vec<String>,

        /// Direcao do efeito luminoso (quando suportado pelo modo)
        #[arg(short = 'd', long = "direction", default_value_t = 0)]
        direction: u8,
    },

    /// Ajusta rapidamente o nivel de brilho da iluminacao (0 a 5)
    Brightness {
        /// Nivel de brilho desejado (0 a 5)
        level: u8,
    },

    /// Desliga completamente a iluminacao RGB do teclado
    Off,

    /// Lista os 20 modos de iluminacao disponiveis e seus respectivos nomes
    Modes,

    /// Sincroniza o relogio do display LCD TFT com a data e hora do sistema
    Clock,

    /// Imprime os nomes das teclas disponiveis organizados pelas linhas do teclado
    Keys,

    /// Configura iluminacao individual tecla por tecla (Per-Key RGB)
    Perkey {
        /// Nivel de brilho para os LEDs (0 a 5)
        #[arg(short = 'b', long = "brightness")]
        brightness: Option<u8>,

        /// Aplica cor base uniforme a todas as teclas: --all <R> <G> <B>
        #[arg(long = "all", num_args = 3, value_names = ["R", "G", "B"])]
        all: Option<Vec<u8>>,

        /// Arquivo YAML de configuracao OU quartetos no formato: <tecla> <R> <G> <B> ...
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Remapeia teclas fisicas na camada normal ou camada de funcao (FN)
    Remap {
        /// Aplica a regra na camada FN (por padrao aplica na camada normal)
        #[arg(long = "fn")]
        fn_layer: bool,

        /// Restaura a camada informada para os valores padrao de fabrica
        #[arg(long = "reset")]
        reset: bool,

        /// Arquivo YAML de remapeamento OU pares de substituicao: <origem> <destino> ...
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Grava imagem ou animacao binaria (240x135) na tela LCD colorida
    Lcd {
        /// Caminho do arquivo binario gerado pelo mkimage (.bin)
        file: String,

        /// Confirma a gravacao sem solicitar confirmacao interativa
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },
}

/// Converte string textual ou indice numerico para o enum `LightingMode`.
fn parse_lighting_mode(s: &str) -> Result<LightingMode> {
    if let Ok(num) = s.parse::<u8>() {
        LightingMode::from_u8(num)
            .ok_or_else(|| anyhow::anyhow!("Modo numerico {} invalido: escolha entre 0 e 19", num))
    } else {
        match s.to_lowercase().replace(['_', '-'], "").as_str() {
            "off" => Ok(LightingMode::Off),
            "static" => Ok(LightingMode::Static),
            "singleon" => Ok(LightingMode::SingleOn),
            "singleoff" => Ok(LightingMode::SingleOff),
            "glittering" => Ok(LightingMode::Glittering),
            "falling" => Ok(LightingMode::Falling),
            "colourful" | "colorful" => Ok(LightingMode::Colourful),
            "breath" => Ok(LightingMode::Breath),
            "spectrum" => Ok(LightingMode::Spectrum),
            "outward" => Ok(LightingMode::Outward),
            "scrolling" => Ok(LightingMode::Scrolling),
            "rolling" => Ok(LightingMode::Rolling),
            "rotating" => Ok(LightingMode::Rotating),
            "explode" => Ok(LightingMode::Explode),
            "launch" => Ok(LightingMode::Launch),
            "ripples" => Ok(LightingMode::Ripples),
            "flowing" => Ok(LightingMode::Flowing),
            "pulsating" => Ok(LightingMode::Pulsating),
            "tilt" => Ok(LightingMode::Tilt),
            "shuttle" => Ok(LightingMode::Shuttle),
            _ => bail!(
                "Modo de iluminacao desconhecido: '{}'. Execute 'f108-pro modes' para ver os disponiveis.",
                s
            ),
        }
    }
}

/// Extrai componentes de cor (R, G, B) e flag multicolorida a partir dos argumentos CLI.
fn parse_cli_color(color_args: &[String]) -> Result<(u8, u8, u8, bool)> {
    if color_args.is_empty() {
        return Ok((0xFF, 0xFF, 0xFF, true));
    }

    if color_args.len() == 1 {
        let val = &color_args[0];
        if val.eq_ignore_ascii_case("rainbow") || val.eq_ignore_ascii_case("colorful") {
            return Ok((0xFF, 0xFF, 0xFF, true));
        }
        let clean = val.trim().trim_start_matches('#');
        if clean.len() == 6 {
            let r = u8::from_str_radix(&clean[0..2], 16).context("Componente R invalido no hex")?;
            let g = u8::from_str_radix(&clean[2..4], 16).context("Componente G invalido no hex")?;
            let b = u8::from_str_radix(&clean[4..6], 16).context("Componente B invalido no hex")?;
            return Ok((r, g, b, false));
        }
        bail!("Formato de cor invalido: '{}'. Use componentes 'R G B' ou hex '#RRGGBB'", val);
    }

    if color_args.len() == 3 {
        let r: u8 = color_args[0].parse().context("Componente R deve ser um numero de 0 a 255")?;
        let g: u8 = color_args[1].parse().context("Componente G deve ser um numero de 0 a 255")?;
        let b: u8 = color_args[2].parse().context("Componente B deve ser um numero de 0 a 255")?;
        return Ok((r, g, b, false));
    }

    bail!("Numero incorreto de argumentos para cor: esperado 1 hex ou 3 valores R G B");
}

/// Imprime na tela os 20 modos de iluminacao suportados pelo hardware Sonix.
fn print_lighting_modes() {
    println!("============================================================");
    println!("   AULA F108 Pro - Modos de Iluminacao RGB Suportados");
    println!("============================================================");
    println!("  ID  | Nome do Efeito      | Descricao");
    println!("  ----+---------------------+---------------------------------");
    let descriptions = [
        "Desligado (LEDs apagados)",
        "Estatico / Cor fixa",
        "Ativacao ao pressionar tecla",
        "Apaga ao pressionar tecla",
        "Cintilante / Glittering",
        "Queda de luzes / Falling",
        "Multicolorido dinâmico",
        "Respiracao / Fade suave",
        "Espectro ciclico continuo",
        "Expansao para fora",
        "Rolagem lateral / Scrolling",
        "Onda rotativa / Rolling",
        "Rotacao / Rotating",
        "Explosao radial",
        "Lancamento / Launch",
        "Ondulacoes / Ripples",
        "Fluxo continuo / Flowing",
        "Pulsacao uniforme",
        "Inclinacao / Tilt",
        "Vai-e-vem / Shuttle",
    ];

    for id in 0..=19u8 {
        let mode = LightingMode::from_u8(id).unwrap();
        println!("  {:2}  | {:<19} | {}", id, mode.name(), descriptions[id as usize]);
    }
    println!("============================================================");
}

/// Imprime todas as teclas fisicas mapeaveis organizadas pelas linhas reais do teclado.
fn print_keyboard_keys() {
    println!("================================================================================");
    println!("   AULA F108 Pro - Mapeamento de Teclas Fisicas (Layout ABNT2/ANSI 108 Teclas)");
    println!("================================================================================");
    println!("\n[Linha 1 - Teclas de Funcao e Topo]");
    println!("  ESC, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, PRINT, SCROLL, PAUSE");
    println!("\n[Linha 2 - Linha Numerica]");
    println!("  GRAVE (`), 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, MINUS (-), EQUALS (=), BACKSPACE");
    println!("  Navegacao: INSERT, HOME, PAGEUP");
    println!("  Teclado Numerico: NUMLOCK, KP_DIVIDE (/), KP_MULTIPLY (*), KP_MINUS (-)");
    println!("\n[Linha 3 - Linha Superior (QWERTY)]");
    println!("  TAB, Q, W, E, R, T, Y, U, I, O, P, LBRACKET ([), RBRACKET (]), BACKSLASH (\\)");
    println!("  Navegacao: DELETE, END, PAGEDOWN");
    println!("  Teclado Numerico: KP_7, KP_8, KP_9, KP_PLUS (+)");
    println!("\n[Linha 4 - Linha Central (Home Row)]");
    println!("  CAPSLOCK, A, S, D, F, G, H, J, K, L, SEMICOLON (;), QUOTE ('), ENTER");
    println!("  Teclado Numerico: KP_4, KP_5, KP_6");
    println!("\n[Linha 5 - Linha Inferior]");
    println!("  LSHIFT, Z, X, C, V, B, N, M, COMMA (,), DOT (.), SLASH (/), RSHIFT");
    println!("  Navegacao: UP");
    println!("  Teclado Numerico: KP_1, KP_2, KP_3, KP_ENTER");
    println!("\n[Linha 6 - Modificadores e Barra de Espaco]");
    println!("  LCTRL, LWIN, LALT, SPACE, RALT, RWIN, FN, RCTRL");
    println!("  Setas: LEFT, DOWN, RIGHT");
    println!("  Teclado Numerico: KP_0, KP_DOT (.)");
    println!("\n[Linha 7 - Teclas Multimidia Dedicadas]");
    println!("  MUTE, VOL_DOWN, VOL_UP, CALCULATOR");
    println!("================================================================================");
}

/// Executa a aplicacao CLI a partir dos argumentos fornecidos.
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Modes => {
            print_lighting_modes();
            Ok(())
        }

        Commands::Keys => {
            print_keyboard_keys();
            Ok(())
        }

        Commands::Off => {
            println!("Desligando iluminacao RGB...");
            let mut dev = Device::open().context("Falha ao conectar com o teclado")?;
            let cfg = LightingConfig {
                mode: LightingMode::Off,
                ..Default::default()
            };
            dev.set_lighting(&cfg)
                .context("Falha ao desligar LEDs do teclado")?;
            println!("Iluminacao desligada com sucesso.");
            Ok(())
        }

        Commands::Brightness { level } => {
            if level > 5 {
                bail!("Nivel de brilho {} invalido: escolha entre 0 (min) e 5 (max)", level);
            }
            println!("Ajustando nivel de brilho para {}...", level);
            let mut dev = Device::open().context("Falha ao conectar com o teclado")?;
            dev.led_strip_setup(level)
                .context("Falha ao configurar brilho dos LEDs")?;
            println!("Nivel de brilho atualizado para {}.", level);
            Ok(())
        }

        Commands::Light {
            mode,
            brightness,
            speed,
            color,
            direction,
        } => {
            let parsed_mode = parse_lighting_mode(&mode)?;
            let (r, g, b, colorful) = parse_cli_color(&color)?;
            let b_val = brightness.unwrap_or(4);
            let s_val = speed.unwrap_or(3);

            let cfg = LightingConfig {
                mode: parsed_mode,
                r,
                g,
                b,
                brightness: b_val,
                speed: s_val,
                direction,
                colorful,
            };

            println!(
                "Aplicando modo '{}': Brilho={}, Velocidade={}, Cor={}{}, Direcao={}",
                parsed_mode.name(),
                b_val,
                s_val,
                if colorful {
                    "Multicolorido (Rainbow)".to_string()
                } else {
                    format!("RGB({}, {}, {})", r, g, b)
                },
                "",
                direction
            );

            let mut dev = Device::open().context("Falha ao conectar com o teclado")?;
            dev.set_lighting(&cfg)
                .context("Falha ao aplicar configuracao de iluminacao no teclado")?;
            println!("Efeito de iluminacao aplicado com sucesso.");
            Ok(())
        }

        Commands::Clock => {
            let now = Local::now();
            println!(
                "Sincronizando relogio do teclado com horario do sistema: {}",
                now.format("%Y-%m-%d %H:%M:%S")
            );

            let mut dev = Device::open().context("Falha ao conectar com o teclado")?;
            dev.sync_clock(now)
                .context("Falha ao enviar comando de sincronizacao de relogio")?;
            println!("Relogio interno do teclado sincronizado com sucesso.");
            Ok(())
        }

        Commands::Perkey {
            brightness,
            all,
            args,
        } => {
            let mut dev = Device::open().context("Falha ao conectar com o teclado")?;
            let mut keys: Vec<KeyColor> = Vec::new();
            let mut final_brightness = brightness;

            // 1. Caso seja fornecido um unico arquivo YAML existente
            let is_yaml_file = args.len() == 1
                && (args[0].ends_with(".yaml") || args[0].ends_with(".yml") || Path::new(&args[0]).is_file());

            if is_yaml_file {
                let (loaded_keys, yaml_b) = yaml_config::load_per_key_yaml(&args[0])
                    .with_context(|| format!("Falha ao carregar configuracao YAML Per-Key de: {}", args[0]))?;
                keys = loaded_keys;
                if final_brightness.is_none() {
                    final_brightness = yaml_b;
                }
            } else {
                // 2. Caso use flag --all ou argumentos CLI
                if let Some(all_rgb) = &all {
                    for idx in 1..=108u8 {
                        keys.push(KeyColor::new(idx, all_rgb[0], all_rgb[1], all_rgb[2]));
                    }
                }

                if !args.is_empty() {
                    if args.len() % 4 != 0 {
                        bail!(
                            "Argumentos CLI invalidos: informe um arquivo YAML ou quartetos '<tecla> <r> <g> <b>' (fornecidos {} argumentos)",
                            args.len()
                        );
                    }

                    for chunk in args.chunks(4) {
                        let name = &chunk[0];
                        let light_idx = key_name_to_index(name)
                            .ok_or_else(|| anyhow::anyhow!("Nome de tecla desconhecido: '{}'", name))?;
                        let r: u8 = chunk[1].parse().context("Componente R deve ser 0..255")?;
                        let g: u8 = chunk[2].parse().context("Componente G deve ser 0..255")?;
                        let b: u8 = chunk[3].parse().context("Componente B deve ser 0..255")?;

                        if let Some(pos) = keys.iter().position(|k| k.light_index == light_idx) {
                            keys[pos] = KeyColor::new(light_idx, r, g, b);
                        } else {
                            keys.push(KeyColor::new(light_idx, r, g, b));
                        }
                    }
                }
            }

            if keys.is_empty() {
                bail!("Nenhuma cor informada. Especifique um arquivo YAML, a opcao --all ou quartetos 'tecla r g b'");
            }

            let b = final_brightness.unwrap_or(4);
            println!(
                "Transmitindo configuracao Per-Key RGB ({} teclas, brilho: {})...",
                keys.len(),
                b
            );

            dev.set_per_key_rgb(&keys, b)
                .context("Falha ao gravar cores individuais no teclado")?;
            println!("Iluminacao Per-Key aplicada e gravada na memoria com sucesso.");
            Ok(())
        }

        Commands::Remap {
            fn_layer,
            reset,
            args,
        } => {
            let mut dev = Device::open().context("Falha ao conectar com o teclado")?;

            if reset {
                let layer_name = if fn_layer { "FN" } else { "Normal" };
                println!("Restaurando camada de remapeamento '{}' para os padroes de fabrica...", layer_name);
                if fn_layer {
                    dev.reset_fn_key_remap()
                        .context("Falha ao restaurar camada FN")?;
                } else {
                    dev.reset_key_remap()
                        .context("Falha ao restaurar camada Normal")?;
                }
                println!("Camada '{}' restaurada com sucesso.", layer_name);
                return Ok(());
            }

            let is_yaml_file = args.len() == 1
                && (args[0].ends_with(".yaml") || args[0].ends_with(".yml") || Path::new(&args[0]).is_file());

            let (remaps, target_fn) = if is_yaml_file {
                let (loaded_remaps, yaml_fn) = yaml_config::load_remap_yaml(&args[0])
                    .with_context(|| format!("Falha ao carregar arquivo de remapeamento: {}", args[0]))?;
                // Se a flag CLI --fn foi informada, ela sobrescreve a declaracao do YAML
                (loaded_remaps, if fn_layer { true } else { yaml_fn })
            } else if args.len() >= 2 && args.len() % 2 == 0 {
                let mut list = Vec::new();
                for chunk in args.chunks(2) {
                    let src = &chunk[0];
                    let dst = &chunk[1];
                    let src_idx = key_name_to_index(src)
                        .ok_or_else(|| anyhow::anyhow!("Tecla de origem desconhecida: '{}'", src))?;
                    let remap = yaml_config::parse_remap_target(src_idx, dst)
                        .with_context(|| format!("Erro na regra '{}' -> '{}'", src, dst))?;
                    list.push(remap);
                }
                (list, fn_layer)
            } else {
                bail!(
                    "Argumentos de remapeamento invalidos: informe um arquivo YAML, a opcao --reset ou pares '<origem> <destino>'"
                );
            };

            let layer_name = if target_fn { "FN" } else { "Normal" };
            println!(
                "Gravando {} regras de remapeamento na camada '{}'...",
                remaps.len(),
                layer_name
            );

            if target_fn {
                dev.set_fn_key_remap(&remaps)
                    .context("Falha ao enviar remapeamento para a camada FN")?;
            } else {
                dev.set_key_remap(&remaps)
                    .context("Falha ao enviar remapeamento para a camada Normal")?;
            }

            println!("Tabela de remapeamento persistida com sucesso na camada '{}'.", layer_name);
            Ok(())
        }

        Commands::Lcd { file, yes } => {
            let path = Path::new(&file);
            let data = fs::read(path)
                .with_context(|| format!("Nao foi possivel ler o arquivo LCD em: {}", file))?;

            if data.is_empty() || data.len() % 4096 != 0 {
                bail!(
                    "Arquivo LCD invalido ({} bytes): o arquivo deve ser nao-vazio e multiplo de 4096 bytes (4KB)",
                    data.len()
                );
            }

            let total_pages = data.len() / 4096;

            println!("======================================================================");
            println!("  AVISO DE SEGURANCA - GRAVACAO NA TELA LCD TFT");
            println!("======================================================================");
            println!("  A gravacao transmite paginas binarias diretamente para a Interface USB 2");
            println!("  do visor. Arquivos invalidos ou malformados podem travar a tela ou o");
            println!("  microcontrolador ate o teclado ser desconectado e reconectado.");
            println!("----------------------------------------------------------------------");
            println!("  Arquivo selecionado: {} ({} bytes, {} paginas)", file, data.len(), total_pages);
            println!("======================================================================");

            if !yes {
                print!("Deseja continuar com a transmissao para o visor? [s/N]: ");
                io::stdout().flush().context("Falha ao descarregar stdout")?;

                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .context("Falha ao ler confirmacao do usuario")?;

                let trimmed = input.trim().to_lowercase();
                if trimmed != "s" && trimmed != "sim" && trimmed != "y" && trimmed != "yes" {
                    println!("Operacao cancelada pelo usuario.");
                    return Ok(());
                }
            }

            println!("Iniciando transferencia para a Interface 2 do LCD...");
            let mut dev = Device::open().context("Falha ao conectar com o teclado")?;

            dev.upload_lcd_image(&data, 0, |done, total| {
                let pct = (done as f64 / total as f64) * 100.0;
                print!("\rTransmitindo LCD: pagina {}/{} [{:.1}%]", done, total, pct);
                let _ = io::stdout().flush();
            })
            .context("Falha durante a transmissao de imagem para o display LCD")?;

            println!("\nGravacao da tela LCD concluida com sucesso!");
            Ok(())
        }
    }
}

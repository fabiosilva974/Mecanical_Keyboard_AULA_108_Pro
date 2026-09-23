//! Modulo de carregamento e processamento de configuracoes em formato YAML para o Aula F108 Pro.
//!
//! Fornece parsers robustos para:
//! - Iluminacao individual por tecla (Per-Key RGB): permite definir cor base (`all`),
//!   brilho global (`brightness`) e customizacao por tecla (`keys`).
//! - Remapeamento de matriz de teclas (Remap): permite remapear teclas da camada normal ou FN
//!   para codigos HID, atalhos com modificadores (combos), funcoes multimidia e cliques/scroll de mouse.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use anyhow::{bail, Context, Result};
use serde_yaml::Value;

use crate::aula::constants::{
    consumer_name_to_code, key_name_to_hid, key_name_to_index, mouse_name_to_params,
};
use crate::aula::perkey::KeyColor;
use crate::aula::remap::{KeyRemap, RemapAction};

/// Converte um valor serde_yaml arbitrario para String.
/// Suporta strings, numeros e caracteres literais comuns do teclado.
fn yaml_val_to_string(val: &Value) -> Result<String> {
    match val {
        Value::String(s) => Ok(s.trim().to_string()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        _ => bail!("Esperado valor textual ou numerico, encontrado: {:?}", val),
    }
}

/// Converte formato RGB [r, g, b] ou string hexadecimal ("#RRGGBB" / "RRGGBB") em tupla (u8, u8, u8).
fn parse_rgb_value(val: &Value) -> Result<(u8, u8, u8)> {
    match val {
        Value::Sequence(seq) => {
            if seq.len() != 3 {
                bail!("Cor RGB deve conter exatamente 3 elementos [R, G, B], encontrados {}", seq.len());
            }
            let r = seq[0].as_u64().context("Valor Red (R) deve ser um numero inteiro")?;
            let g = seq[1].as_u64().context("Valor Green (G) deve ser um numero inteiro")?;
            let b = seq[2].as_u64().context("Valor Blue (B) deve ser um numero inteiro")?;

            if r > 255 || g > 255 || b > 255 {
                bail!("Valores de cor RGB devem estar no intervalo de 0 a 255. Recebido: [{}, {}, {}]", r, g, b);
            }
            Ok((r as u8, g as u8, b as u8))
        }
        Value::String(s) => parse_hex_color(s),
        _ => bail!("Formato de cor invalido: esperado lista [R, G, B] ou string hex, encontrado: {:?}", val),
    }
}

/// Interpreta uma string hexadecimal como uma tupla (r, g, b).
fn parse_hex_color(hex: &str) -> Result<(u8, u8, u8)> {
    let s = hex.trim().trim_start_matches('#');
    if s.len() != 6 {
        bail!("Cor hexadecimal deve conter 6 digitos (ex: 'FF00FF' ou '#00FF00'), recebido: '{}'", hex);
    }
    let r = u8::from_str_radix(&s[0..2], 16)
        .context("Falha ao processar componente Red em hexadecimal")?;
    let g = u8::from_str_radix(&s[2..4], 16)
        .context("Falha ao processar componente Green em hexadecimal")?;
    let b = u8::from_str_radix(&s[4..6], 16)
        .context("Falha ao processar componente Blue em hexadecimal")?;
    Ok((r, g, b))
}

/// Carrega e processa a configuracao de iluminacao Per-Key RGB a partir do conteudo em string YAML.
///
/// # Formato esperado:
/// ```yaml
/// all: [0, 0, 50]       # Opcional: cor base para todas as 108 teclas fisicas
/// brightness: 5         # Opcional: nivel de brilho (0 a 5)
/// keys:                 # Opcional se 'all' estiver presente
///   w: [0, 255, 0]
///   a: [0, 255, 0]
///   s: [0, 255, 0]
///   d: [0, 255, 0]
/// ```
///
/// Se `all` estiver presente, aplica a cor base a todas as teclas mapeadas (indices 1..=108).
/// Em seguida, aplica os overrides individuais informados no mapa `keys`.
/// Se `all` nao for informado, aplica apenas as teclas listadas em `keys`.
pub fn parse_per_key_yaml_str(content: &str) -> Result<(Vec<KeyColor>, Option<u8>)> {
    let root: Value = serde_yaml::from_str(content)
        .context("Falha de sintaxe ao interpretar arquivo YAML de Per-Key")?;

    let root_map = root.as_mapping()
        .context("O documento YAML deve ser um mapa de chave-valor na raiz")?;

    // Brilho opcional
    let mut brightness = None;
    if let Some(b_val) = root_map.get(&Value::String("brightness".to_string())) {
        let b = b_val.as_u64()
            .context("Campo 'brightness' deve ser um numero inteiro de 0 a 5")?;
        if b > 5 {
            bail!("Nivel de brilho {} invalido: o valor maximo suportado pelo hardware e 5", b);
        }
        brightness = Some(b as u8);
    }

    let mut key_map: BTreeMap<u8, KeyColor> = BTreeMap::new();

    // 1. Cor base 'all' (se fornecida)
    if let Some(all_val) = root_map.get(&Value::String("all".to_string())) {
        let (r, g, b) = parse_rgb_value(all_val)
            .context("Falha ao processar cor base 'all'")?;

        // Aplica a todas as 108 teclas fisicas conhecidas da matriz
        for idx in 1..=108u8 {
            key_map.insert(idx, KeyColor::new(idx, r, g, b));
        }
    }

    // 2. Overrides individuais em 'keys'
    if let Some(keys_val) = root_map.get(&Value::String("keys".to_string())) {
        let keys_mapping = keys_val.as_mapping()
            .context("A secao 'keys' deve ser um mapa de teclas e cores")?;

        for (k_val, v_val) in keys_mapping {
            let key_str = yaml_val_to_string(k_val)?;
            let light_idx = key_name_to_index(&key_str)
                .ok_or_else(|| anyhow::anyhow!("Nome de tecla desconhecido no mapeamento per-key: '{}'", key_str))?;

            let (r, g, b) = parse_rgb_value(v_val)
                .with_context(|| format!("Falha ao interpretar cor para a tecla '{}'", key_str))?;

            key_map.insert(light_idx, KeyColor::new(light_idx, r, g, b));
        }
    }

    if key_map.is_empty() {
        bail!("Nenhuma cor definida no YAML: forneca 'all' ou a secao 'keys'");
    }

    let result = key_map.into_values().collect();
    Ok((result, brightness))
}

/// Carrega e processa a configuracao de iluminacao Per-Key RGB a partir de um arquivo em disco.
pub fn load_per_key_yaml(path: &str) -> Result<(Vec<KeyColor>, Option<u8>)> {
    let content = fs::read_to_string(Path::new(path))
        .with_context(|| format!("Nao foi possivel abrir o arquivo YAML em: {}", path))?;
    parse_per_key_yaml_str(&content)
}

/// Identifica codigo de modificador HID a partir de nome textual.
fn modifier_name_to_code(name: &str) -> Option<u8> {
    match name.to_uppercase().as_str() {
        "CTRL" | "LCTRL" | "LEFT_CTRL" => Some(0xE0),
        "SHIFT" | "LSHIFT" | "LEFT_SHIFT" => Some(0xE1),
        "ALT" | "LALT" | "LEFT_ALT" => Some(0xE2),
        "WIN" | "GUI" | "LWIN" | "LGUI" | "SUPER" | "CMD" => Some(0xE3),
        "RCTRL" | "RIGHT_CTRL" => Some(0xE4),
        "RSHIFT" | "RIGHT_SHIFT" => Some(0xE5),
        "RALT" | "ALTGR" | "RIGHT_ALT" => Some(0xE6),
        "RWIN" | "RGUI" | "RIGHT_GUI" => Some(0xE7),
        _ => None,
    }
}

/// Converte nome de comando multimidia para o codigo da controladora Sonix.
fn parse_consumer_action(name: &str) -> Option<u8> {
    let clean = name.trim().to_uppercase();
    match clean.as_str() {
        "PLAY" | "PAUSE" | "PLAY_PAUSE" | "PLAYPAUSE" => Some(0xCD),
        "VOL_UP" | "VOLUP" | "VOLUMEUP" | "VOLUME_UP" => Some(0xE9),
        "VOL_DOWN" | "VOLDOWN" | "VOLUMEDOWN" | "VOLUME_DOWN" => Some(0xEA),
        "MUTE" => Some(0xE2),
        "STOP" => Some(0xB7),
        "NEXT" | "NEXT_TRACK" | "NEXTTRACK" => Some(0xB5),
        "PREV" | "PREV_TRACK" | "PREVTRACK" => Some(0xB6),
        "CALCULATOR" | "CALC" => Some(0x92),
        "MY_COMPUTER" | "MYCOMPUTER" | "COMPUTER" => Some(0x94),
        "EMAIL" | "MAIL" => Some(0x8A),
        "WWW_BROWSER" | "BROWSER" | "WWW" => Some(0x23),
        _ => consumer_name_to_code(&clean),
    }
}

/// Converte acao de mouse textual para parametros [botao, acao, scroll_wheel].
fn parse_mouse_action(name: &str) -> Option<[u8; 3]> {
    let clean = name.trim().to_uppercase();
    match clean.as_str() {
        "LCLICK" | "LEFT" | "MOUSE_LEFT" | "CLICK_LEFT" | "BTN1" => Some([0x01, 0x01, 0x00]),
        "RCLICK" | "RIGHT" | "MOUSE_RIGHT" | "CLICK_RIGHT" | "BTN2" => Some([0x02, 0x01, 0x00]),
        "MCLICK" | "MIDDLE" | "MOUSE_MIDDLE" | "CLICK_MIDDLE" | "BTN3" => Some([0x04, 0x01, 0x00]),
        "SCROLLUP" | "WHEELUP" | "WHEEL_UP" | "SCROLL_UP" => Some([0x00, 0x00, 0x01]),
        "SCROLLDOWN" | "WHEELDOWN" | "WHEEL_DOWN" | "SCROLL_DOWN" => Some([0x00, 0x00, 0xFF]),
        _ => mouse_name_to_params(&clean),
    }
}

/// Interpreta o valor de destino de uma regra de remapeamento (simples, combo, media ou mouse).
pub fn parse_remap_target(source_index: u8, target: &str) -> Result<KeyRemap> {
    let trimmed = target.trim();

    // 1. Acao multimidia (ex: "media:play", "media:volup", "consumer:mute")
    if let Some(rest) = trimmed.strip_prefix("media:").or_else(|| trimmed.strip_prefix("consumer:")) {
        let code = parse_consumer_action(rest)
            .ok_or_else(|| anyhow::anyhow!("Comando multimidia desconhecido: '{}'", rest))?;
        return Ok(KeyRemap::new_consumer_remap(source_index, code));
    }

    // 2. Acao de mouse (ex: "mouse:lclick", "mouse:scrollup")
    if let Some(rest) = trimmed.strip_prefix("mouse:") {
        let params = parse_mouse_action(rest)
            .ok_or_else(|| anyhow::anyhow!("Acao de mouse desconhecida: '{}'", rest))?;
        return Ok(KeyRemap::new_mouse_remap(source_index, params[0], params[1], params[2]));
    }

    // 3. Combos com modificadores (ex: "ctrl+c", "alt+f4", "win+d", "ctrl+shift+esc")
    if trimmed.contains('+') {
        let parts: Vec<&str> = trimmed.split('+').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        if parts.is_empty() {
            bail!("Expressao de combo vazia ou invalida: '{}'", trimmed);
        }

        let mut mod_mask: u8 = 0;
        let mut base_key_name: Option<&str> = None;

        for part in parts {
            if let Some(mod_code) = modifier_name_to_code(part) {
                let bit = match mod_code {
                    0xE0 => 0x01, // LCtrl
                    0xE1 => 0x02, // LShift
                    0xE2 => 0x04, // LAlt
                    0xE3 => 0x08, // LGUI/Win
                    0xE4 => 0x10, // RCtrl
                    0xE5 => 0x20, // RShift
                    0xE6 => 0x40, // RAlt
                    0xE7 => 0x80, // RGUI/Win
                    _ => 0x00,
                };
                mod_mask |= bit;
            } else {
                if base_key_name.is_some() {
                    bail!("Combo contem multiplas teclas base nao modificadoras: '{}'", trimmed);
                }
                base_key_name = Some(part);
            }
        }

        let key_name = base_key_name
            .ok_or_else(|| anyhow::anyhow!("Combo precisa conter uma tecla base (ex: 'ctrl+c'): '{}'", trimmed))?;

        let hid_code = key_name_to_hid(key_name)
            .ok_or_else(|| anyhow::anyhow!("Codigo HID nao encontrado para a tecla base '{}' no combo '{}'", key_name, trimmed))?;

        return Ok(KeyRemap::new_key_combo(source_index, hid_code, mod_mask));
    }

    // 4. Tecla modificadora isolada (ex: "capslock: lctrl", "capslock: rshift")
    if let Some(mod_code) = modifier_name_to_code(trimmed) {
        return Ok(KeyRemap::new_key_swap(source_index, mod_code));
    }

    // 5. Tecla padrao HID simples (ex: "capslock: esc", "a: b")
    if let Some(hid) = key_name_to_hid(trimmed) {
        return Ok(KeyRemap::new_key_swap(source_index, hid));
    }

    // 6. Tecla desabilitada (ex: "none", "disabled")
    if trimmed.eq_ignore_ascii_case("none") || trimmed.eq_ignore_ascii_case("disabled") {
        return Ok(KeyRemap {
            source_index,
            action: RemapAction::None,
            param1: 0,
            param2: 0,
            param3: 0,
        });
    }

    bail!("Destino de remapeamento nao reconhecido: '{}'", trimmed);
}

/// Carrega e processa a tabela de remapeamento de teclas a partir do conteudo em string YAML.
///
/// # Formato esperado:
/// ```yaml
/// layer: normal # ou "fn"
/// keys:
///   capslock: lctrl
///   f1: media:play
///   f2: mouse:lclick
///   f3: ctrl+c
/// ```
///
/// Retorna a lista de [`KeyRemap`] e uma flag booleana `fn_layer` (`true` para camada FN).
pub fn parse_remap_yaml_str(content: &str) -> Result<(Vec<KeyRemap>, bool)> {
    let root: Value = serde_yaml::from_str(content)
        .context("Falha de sintaxe ao interpretar arquivo YAML de Remapeamento")?;

    let root_map = root.as_mapping()
        .context("O documento YAML deve ser um mapa de chave-valor na raiz")?;

    // Determina a camada ("normal" ou "fn")
    let mut fn_layer = false;
    if let Some(layer_val) = root_map.get(&Value::String("layer".to_string())) {
        let layer_str = yaml_val_to_string(layer_val)?.to_lowercase();
        match layer_str.as_str() {
            "fn" => fn_layer = true,
            "normal" | "base" | "default" => fn_layer = false,
            other => bail!("Camada desconhecida: '{}'. Valores permitidos: 'normal' ou 'fn'", other),
        }
    }

    let mut remap_map: BTreeMap<u8, KeyRemap> = BTreeMap::new();

    if let Some(keys_val) = root_map.get(&Value::String("keys".to_string())) {
        let keys_mapping = keys_val.as_mapping()
            .context("A secao 'keys' de remapeamento deve ser um mapa de chave-valor")?;

        for (k_val, v_val) in keys_mapping {
            let src_str = yaml_val_to_string(k_val)?;
            let src_index = key_name_to_index(&src_str)
                .ok_or_else(|| anyhow::anyhow!("Tecla de origem desconhecida no remapeamento: '{}'", src_str))?;

            let dst_str = yaml_val_to_string(v_val)?;
            let remap = parse_remap_target(src_index, &dst_str)
                .with_context(|| format!("Erro na regra '{}' -> '{}'", src_str, dst_str))?;

            remap_map.insert(src_index, remap);
        }
    } else {
        bail!("O arquivo de remapeamento deve conter a secao 'keys'");
    }

    let result = remap_map.into_values().collect();
    Ok((result, fn_layer))
}

/// Carrega e processa a tabela de remapeamento de teclas a partir de um arquivo em disco.
pub fn load_remap_yaml(path: &str) -> Result<(Vec<KeyRemap>, bool)> {
    let content = fs::read_to_string(Path::new(path))
        .with_context(|| format!("Nao foi possivel abrir o arquivo YAML em: {}", path))?;
    parse_remap_yaml_str(&content)
}

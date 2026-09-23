//! Testes unitarios para os parsers YAML de iluminacao Per-Key e Remapeamento de teclas.

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use aula::constants::{KEY_CAPSLOCK, KEY_ESC, KEY_F1, KEY_F2, KEY_F3, KEY_F4, KEY_SPACE, KEY_W};
    use aula::remap::RemapAction;
    use aula::cli::yaml_config::{
        load_per_key_yaml, load_remap_yaml, parse_per_key_yaml_str, parse_remap_yaml_str,
    };

    #[test]
    fn test_per_key_all_with_overrides() {
        let yaml = r#"
all: [0, 0, 50]
brightness: 5
keys:
  w: [0, 255, 0]
  a: [0, 255, 0]
"#;
        let (keys, brightness) = parse_per_key_yaml_str(yaml).expect("Falha ao processar YAML Per-Key");

        // Deve conter todas as 108 teclas fisicas da matriz
        assert_eq!(keys.len(), 108);
        assert_eq!(brightness, Some(5));

        // Tecla W deve ter recebido a cor customizada (0, 255, 0)
        let w_key = keys.iter().find(|k| k.light_index == KEY_W).expect("Tecla W nao encontrada");
        assert_eq!(w_key.r, 0);
        assert_eq!(w_key.g, 255);
        assert_eq!(w_key.b, 0);

        // Tecla ESC deve manter a cor base de 'all' (0, 0, 50)
        let esc_key = keys.iter().find(|k| k.light_index == KEY_ESC).expect("Tecla ESC nao encontrada");
        assert_eq!(esc_key.r, 0);
        assert_eq!(esc_key.g, 0);
        assert_eq!(esc_key.b, 50);
    }

    #[test]
    fn test_per_key_keys_only_without_all() {
        let yaml = r#"
keys:
  esc: [255, 0, 0]
  space: [0, 0, 255]
"#;
        let (keys, brightness) = parse_per_key_yaml_str(yaml).expect("Falha ao processar YAML");
        assert_eq!(keys.len(), 2);
        assert_eq!(brightness, None);

        let esc = keys.iter().find(|k| k.light_index == KEY_ESC).unwrap();
        assert_eq!((esc.r, esc.g, esc.b), (255, 0, 0));

        let space = keys.iter().find(|k| k.light_index == KEY_SPACE).unwrap();
        assert_eq!((space.r, space.g, space.b), (0, 0, 255));
    }

    #[test]
    fn test_per_key_hex_colors() {
        let yaml = r##"
keys:
  esc: "#FF00FF"
  enter: "00FF00"
"##;
        let (keys, _) = parse_per_key_yaml_str(yaml).expect("Falha ao processar hex em YAML");
        assert_eq!(keys.len(), 2);

        let esc = keys.iter().find(|k| k.light_index == KEY_ESC).unwrap();
        assert_eq!((esc.r, esc.g, esc.b), (0xFF, 0x00, 0xFF));
    }

    #[test]
    fn test_per_key_invalid_brightness() {
        let yaml = r#"
brightness: 6
keys:
  w: [255, 255, 255]
"#;
        assert!(parse_per_key_yaml_str(yaml).is_err());
    }

    #[test]
    fn test_per_key_invalid_key_name() {
        let yaml = r#"
keys:
  tecla_fantasma_xyz: [255, 0, 0]
"#;
        assert!(parse_per_key_yaml_str(yaml).is_err());
    }

    #[test]
    fn test_remap_normal_layer_all_target_types() {
        let yaml = r#"
layer: normal
keys:
  capslock: lctrl
  f1: media:play
  f2: mouse:lclick
  f3: ctrl+c
  f4: alt+f4
"#;
        let (remaps, fn_layer) = parse_remap_yaml_str(yaml).expect("Falha ao processar YAML de remap");
        assert!(!fn_layer, "Camada esperada: normal (false)");
        assert_eq!(remaps.len(), 5);

        // 1. capslock -> lctrl (Key Swap com HID 0xE0)
        let capslock = remaps.iter().find(|r| r.source_index == KEY_CAPSLOCK).unwrap();
        assert_eq!(capslock.action, RemapAction::Key);
        assert_eq!(capslock.param1, 0xE0); // LCtrl HID

        // 2. f1 -> media:play (Consumer Action 0xCD)
        let f1 = remaps.iter().find(|r| r.source_index == KEY_F1).unwrap();
        assert_eq!(f1.action, RemapAction::Consumer);
        assert_eq!(f1.param1, 0xCD);

        // 3. f2 -> mouse:lclick (Mouse Action: Botao 1, Acao 1)
        let f2 = remaps.iter().find(|r| r.source_index == KEY_F2).unwrap();
        assert_eq!(f2.action, RemapAction::Mouse);
        assert_eq!(f2.param1, 0x01); // Botao Esquerdo
        assert_eq!(f2.param2, 0x01); // Acao Clique
        assert_eq!(f2.param3, 0x00); // Sem scroll

        // 4. f3 -> ctrl+c (Key Combo: HID 0x06 para C, Modificador 0x01 para Ctrl)
        let f3 = remaps.iter().find(|r| r.source_index == KEY_F3).unwrap();
        assert_eq!(f3.action, RemapAction::Key);
        assert_eq!(f3.param1, 0x06); // HID da tecla 'C'
        assert_eq!(f3.param2, 0x01); // Mascara bit LCtrl

        // 5. f4 -> alt+f4 (Key Combo: HID 0x3D para F4, Modificador 0x04 para Alt)
        let f4 = remaps.iter().find(|r| r.source_index == KEY_F4).unwrap();
        assert_eq!(f4.action, RemapAction::Key);
        assert_eq!(f4.param1, 0x3D); // HID de F4
        assert_eq!(f4.param2, 0x04); // Mascara bit LAlt
    }

    #[test]
    fn test_remap_fn_layer() {
        let yaml = r#"
layer: fn
keys:
  f1: media:volup
  f2: mouse:scrollup
"#;
        let (remaps, fn_layer) = parse_remap_yaml_str(yaml).expect("Falha ao processar YAML");
        assert!(fn_layer, "Camada esperada: FN (true)");
        assert_eq!(remaps.len(), 2);

        let f1 = remaps.iter().find(|r| r.source_index == KEY_F1).unwrap();
        assert_eq!(f1.action, RemapAction::Consumer);
        assert_eq!(f1.param1, 0xE9); // Volume UP

        let f2 = remaps.iter().find(|r| r.source_index == KEY_F2).unwrap();
        assert_eq!(f2.action, RemapAction::Mouse);
        assert_eq!(f2.param3, 0x01); // Scroll UP
    }

    #[test]
    fn test_load_per_key_yaml_file() {
        let mut tmp_file = PathBuf::from(std::env::temp_dir());
        tmp_file.push(format!("test_per_key_{}.yaml", std::process::id()));

        let yaml_content = "brightness: 3\nkeys:\n  esc: [100, 150, 200]\n";
        fs::write(&tmp_file, yaml_content).expect("Falha ao criar arquivo temporario");

        let (keys, b) = load_per_key_yaml(tmp_file.to_str().unwrap()).expect("Falha ao carregar arquivo");
        let _ = fs::remove_file(&tmp_file);

        assert_eq!(b, Some(3));
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].light_index, KEY_ESC);
        assert_eq!((keys[0].r, keys[0].g, keys[0].b), (100, 150, 200));
    }

    #[test]
    fn test_load_remap_yaml_file() {
        let mut tmp_file = PathBuf::from(std::env::temp_dir());
        tmp_file.push(format!("test_remap_{}.yaml", std::process::id()));

        let yaml_content = "layer: normal\nkeys:\n  capslock: lctrl\n";
        fs::write(&tmp_file, yaml_content).expect("Falha ao criar arquivo temporario");

        let (remaps, fn_layer) = load_remap_yaml(tmp_file.to_str().unwrap()).expect("Falha ao carregar arquivo");
        let _ = fs::remove_file(&tmp_file);

        assert!(!fn_layer);
        assert_eq!(remaps.len(), 1);
        assert_eq!(remaps[0].source_index, KEY_CAPSLOCK);
        assert_eq!(remaps[0].param1, 0xE0);
    }
}

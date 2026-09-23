//! Ponto de entrada principal do utilitario de linha de comando `f108-pro`.
//!
//! Controla o teclado mecanico Aula F108 Pro no Linux, fornecendo acesso a iluminacao RGB,
//! sincronizacao do relogio no visor TFT, gravacao de imagens no display LCD,
//! remapeamento de matriz de teclas e iluminacao customizada tecla a tecla (Per-Key).

use aula::cli;

use std::process::exit;

fn main() {
    if let Err(err) = cli::run() {
        eprintln!("\n[f108-pro] Erro na execucao:");
        eprintln!("  {}", err);

        let mut source = err.source();
        while let Some(cause) = source {
            eprintln!("  -> Causa detalhada: {}", cause);
            source = cause.source();
        }

        exit(1);
    }
}

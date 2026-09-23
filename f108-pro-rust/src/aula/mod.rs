//! Modulo central da biblioteca `aula` para controle do teclado Aula F108 Pro.

pub mod constants;
pub mod transport;
pub mod device;
pub mod lighting;
pub mod clock;
pub mod perkey;
pub mod remap;
pub mod lcd;

// Re-exportacoes convenientes
pub use constants::*;
pub use device::Device;
pub use transport::Transport;
pub use lighting::LightingConfig;
pub use perkey::{KeyColor, PER_KEY_RGB_SIZE};
pub use remap::{KeyRemap, RemapAction, REMAP_TABLE_SIZE};
pub use lcd::LCD_PAGE_SIZE;

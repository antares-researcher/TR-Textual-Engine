mod document;
mod layout;
mod editor;
mod types;

// Nova implementação V2
mod piece_table;
mod document_v2;
mod layout_v2;
mod editor_v2;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Exportar versão antiga para compatibilidade
pub use document::*;
pub use layout::*;
pub use editor::*;
pub use types::*;

// Exportar nova versão V2
pub use piece_table::*;
pub use document_v2::*;
pub use layout_v2::*;
pub use editor_v2::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_log!("Trevia Editor Engine V2 initialized - Using Piece Table");
}
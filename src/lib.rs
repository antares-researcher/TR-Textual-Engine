mod document;
mod layout;
mod editor;
mod types;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

pub use document::*;
pub use layout::*;
pub use editor::*;
pub use types::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_log!("Trevia Editor Engine initialized");
}
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[wasm_bindgen]
impl Position {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64) -> Position {
        Position { x, y }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

#[wasm_bindgen]
impl Size {
    #[wasm_bindgen(constructor)]
    pub fn new(width: f64, height: f64) -> Size {
        Size { width, height }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[wasm_bindgen]
impl Rect {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Rect {
        Rect { x, y, width, height }
    }

    pub fn contains_point(&self, pos: &Position) -> bool {
        pos.x >= self.x && pos.x <= self.x + self.width &&
        pos.y >= self.y && pos.y <= self.y + self.height
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyle {
    pub font_family: String,
    pub font_size: f64,
    pub font_weight: String,
    pub color: String,
    pub italic: bool,
    pub underline: bool,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: "Arial".to_string(),
            font_size: 12.0,
            font_weight: "normal".to_string(),
            color: "#000000".to_string(),
            italic: false,
            underline: false,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CursorPosition {
    pub offset: usize,
    pub page: usize,
}

#[wasm_bindgen]
impl CursorPosition {
    #[wasm_bindgen(constructor)]
    pub fn new(offset: usize, page: usize) -> CursorPosition {
        CursorPosition { offset, page }
    }
}
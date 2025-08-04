use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRun {
    pub text: String,
    pub style: TextStyle,
    pub start: usize,
    pub end: usize,
}

impl TextRun {
    pub fn new(text: String, style: TextStyle, start: usize) -> Self {
        let end = start + text.len();
        Self { text, style, start, end }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    pub runs: Vec<TextRun>,
    pub start_offset: usize,
    pub end_offset: usize,
}

impl Paragraph {
    pub fn new() -> Self {
        Self {
            runs: Vec::new(),
            start_offset: 0,
            end_offset: 0,
        }
    }

    pub fn add_run(&mut self, mut run: TextRun) {
        run.start = self.end_offset;
        run.end = run.start + run.text.len();
        self.end_offset = run.end;
        self.runs.push(run);
    }

    pub fn get_text(&self) -> String {
        self.runs.iter().map(|run| run.text.as_str()).collect::<String>()
    }
}

#[wasm_bindgen]
pub struct DocumentModel {
    content: String,
    style_map: HashMap<usize, TextStyle>,
}

#[wasm_bindgen]
impl DocumentModel {
    #[wasm_bindgen(constructor)]
    pub fn new() -> DocumentModel {
        DocumentModel {
            content: String::new(),
            style_map: HashMap::new(),
        }
    }

    pub fn insert_text(&mut self, position: usize, text: &str) {
        if text.is_empty() {
            return;
        }

        let pos = position.min(self.content.len());
        self.content.insert_str(pos, text);
    }

    pub fn delete_text(&mut self, start: usize, length: usize) {
        if length == 0 || start >= self.content.len() {
            return;
        }

        let end = (start + length).min(self.content.len());
        self.content.drain(start..end);
    }

    pub fn get_text(&self) -> String {
        self.content.clone()
    }

    pub fn get_text_length(&self) -> usize {
        self.content.len()
    }

    pub fn get_paragraph_count(&self) -> usize {
        self.content.lines().count().max(1)
    }

    #[wasm_bindgen(getter)]
    pub fn paragraphs_js(&self) -> JsValue {
        // Criar paragraphs simulados para compatibilidade
        let paragraphs: Vec<Paragraph> = self.content
            .lines()
            .enumerate()
            .map(|(i, line)| {
                let mut paragraph = Paragraph::new();
                if !line.is_empty() {
                    let run = TextRun::new(line.to_string(), TextStyle::default(), i * (line.len() + 1));
                    paragraph.add_run(run);
                }
                paragraph
            })
            .collect();
        
        serde_wasm_bindgen::to_value(&paragraphs).unwrap_or(JsValue::NULL)
    }
}

impl DocumentModel {
    fn get_style_at_position(&self, _position: usize) -> TextStyle {
        TextStyle::default()
    }
}
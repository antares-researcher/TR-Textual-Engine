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
    paragraphs: Vec<Paragraph>,
    total_length: usize,
    style_map: HashMap<usize, TextStyle>,
}

#[wasm_bindgen]
impl DocumentModel {
    #[wasm_bindgen(constructor)]
    pub fn new() -> DocumentModel {
        DocumentModel {
            paragraphs: vec![Paragraph::new()],
            total_length: 0,
            style_map: HashMap::new(),
        }
    }

    pub fn insert_text(&mut self, position: usize, text: &str) {
        let style = self.get_style_at_position(position);
        self.insert_text_with_style(position, text, &style);
    }

    fn insert_text_with_style(&mut self, position: usize, text: &str, style: &TextStyle) {
        if text.is_empty() {
            return;
        }

        let (paragraph_index, local_offset) = self.find_paragraph_at_position(position);
        
        if let Some(paragraph) = self.paragraphs.get_mut(paragraph_index) {
            let run = TextRun::new(text.to_string(), style.clone(), position);
            
            if paragraph.runs.is_empty() {
                paragraph.add_run(run);
            } else {
                self.split_and_insert_run(paragraph_index, local_offset, run);
            }
            
            self.update_offsets_after_insertion(position, text.len());
            self.total_length += text.len();
        }
    }

    pub fn delete_text(&mut self, start: usize, length: usize) {
        if length == 0 || start >= self.total_length {
            return;
        }

        let end = (start + length).min(self.total_length);
        self.remove_text_range(start, end);
        self.update_offsets_after_deletion(start, length);
        self.total_length -= end - start;
    }

    pub fn get_text(&self) -> String {
        self.paragraphs
            .iter()
            .map(|p| p.get_text())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn get_text_length(&self) -> usize {
        self.total_length
    }

    pub fn get_paragraph_count(&self) -> usize {
        self.paragraphs.len()
    }

    #[wasm_bindgen(getter)]
    pub fn paragraphs_js(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.paragraphs).unwrap_or(JsValue::NULL)
    }
}

impl DocumentModel {
    fn find_paragraph_at_position(&self, position: usize) -> (usize, usize) {
        let mut current_offset = 0;
        
        for (i, paragraph) in self.paragraphs.iter().enumerate() {
            let paragraph_length = paragraph.end_offset - paragraph.start_offset;
            if position <= current_offset + paragraph_length {
                return (i, position - current_offset);
            }
            current_offset += paragraph_length + 1; // +1 for newline
        }
        
        (self.paragraphs.len() - 1, 0)
    }

    fn split_and_insert_run(&mut self, paragraph_index: usize, local_offset: usize, new_run: TextRun) {
        if let Some(paragraph) = self.paragraphs.get_mut(paragraph_index) {
            let mut insert_index = 0;
            let mut current_offset = 0;

            for (i, run) in paragraph.runs.iter().enumerate() {
                if local_offset <= current_offset + run.text.len() {
                    insert_index = i;
                    break;
                }
                current_offset += run.text.len();
            }

            paragraph.runs.insert(insert_index, new_run);
        }
    }

    fn remove_text_range(&mut self, start: usize, end: usize) {
        for paragraph in &mut self.paragraphs {
            paragraph.runs.retain(|run| {
                !(run.start >= start && run.end <= end)
            });
            
            for run in &mut paragraph.runs {
                if run.start < start && run.end > start {
                    let keep_length = start - run.start;
                    run.text.truncate(keep_length);
                    run.end = run.start + run.text.len();
                }
                
                if run.start < end && run.end > end {
                    let remove_from_start = end - run.start;
                    run.text = run.text[remove_from_start..].to_string();
                    run.start = end;
                    run.end = run.start + run.text.len();
                }
            }
        }
    }

    fn update_offsets_after_insertion(&mut self, position: usize, length: usize) {
        for paragraph in &mut self.paragraphs {
            for run in &mut paragraph.runs {
                if run.start >= position {
                    run.start += length;
                    run.end += length;
                } else if run.end > position {
                    run.end += length;
                }
            }
            
            if paragraph.start_offset >= position {
                paragraph.start_offset += length;
            }
            if paragraph.end_offset >= position {
                paragraph.end_offset += length;
            }
        }
    }

    fn update_offsets_after_deletion(&mut self, position: usize, length: usize) {
        for paragraph in &mut self.paragraphs {
            for run in &mut paragraph.runs {
                if run.start >= position + length {
                    run.start -= length;
                    run.end -= length;
                } else if run.end > position {
                    run.end = (run.end - length).max(position);
                }
            }
            
            if paragraph.start_offset >= position + length {
                paragraph.start_offset -= length;
            }
            if paragraph.end_offset > position {
                paragraph.end_offset = (paragraph.end_offset - length).max(position);
            }
        }
    }

    fn get_style_at_position(&self, position: usize) -> TextStyle {
        if let Some(style) = self.style_map.get(&position) {
            return style.clone();
        }

        let (paragraph_index, local_offset) = self.find_paragraph_at_position(position);
        
        if let Some(paragraph) = self.paragraphs.get(paragraph_index) {
            let mut current_offset = 0;
            for run in &paragraph.runs {
                if local_offset <= current_offset + run.text.len() {
                    return run.style.clone();
                }
                current_offset += run.text.len();
            }
        }

        TextStyle::default()
    }
}
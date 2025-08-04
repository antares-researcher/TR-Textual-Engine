use crate::types::*;
use crate::document::{DocumentModel, Paragraph, TextRun};
use wasm_bindgen::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LineLayout {
    pub text: String,
    pub y_position: f64,
    pub height: f64,
    pub runs: Vec<TextRun>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PageLayout {
    pub page_number: usize,
    pub lines: Vec<LineLayout>,
    pub bounds: Rect,
    pub text_bounds: Rect,
}

impl PageLayout {
    pub fn new(page_number: usize, bounds: Rect, margins: Rect) -> Self {
        let text_bounds = Rect::new(
            bounds.x + margins.x,
            bounds.y + margins.y,
            bounds.width - margins.x - margins.width,
            bounds.height - margins.y - margins.height,
        );

        Self {
            page_number,
            lines: Vec::new(),
            bounds,
            text_bounds,
        }
    }
}

#[wasm_bindgen]
pub struct LayoutEngine {
    page_width: f64,
    page_height: f64,
    margins: Rect,
    line_height: f64,
    pages: Vec<PageLayout>,
    char_metrics: HashMap<String, f64>,
}

#[wasm_bindgen]
impl LayoutEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(page_width: f64, page_height: f64) -> LayoutEngine {
        let margins = Rect::new(60.0, 60.0, 60.0, 60.0); // Margens mais apropriadas para A4
        
        LayoutEngine {
            page_width,
            page_height,
            margins,
            line_height: 20.0, // Altura de linha mais confortável
            pages: Vec::new(),
            char_metrics: HashMap::new(),
        }
    }

    pub fn set_margins(&mut self, top: f64, right: f64, bottom: f64, left: f64) {
        self.margins = Rect::new(left, top, right, bottom);
        self.invalidate_layout();
    }

    pub fn set_line_height(&mut self, height: f64) {
        self.line_height = height;
        self.invalidate_layout();
    }

    pub fn compute_layout(&mut self, document: &DocumentModel) {
        self.pages.clear();
        
        if document.get_paragraph_count() == 0 {
            return;
        }

        let page_bounds = Rect::new(0.0, 0.0, self.page_width, self.page_height);
        let mut current_page = PageLayout::new(0, page_bounds, self.margins);
        let mut current_y = current_page.text_bounds.y;

        for paragraph_data in self.get_paragraphs_data(document) {
            let lines = self.break_paragraph_into_lines(&paragraph_data, current_page.text_bounds.width);
            
            for line in lines {
                if current_y + self.line_height > current_page.text_bounds.y + current_page.text_bounds.height {
                    self.pages.push(current_page);
                    current_page = PageLayout::new(self.pages.len(), page_bounds, self.margins);
                    current_y = current_page.text_bounds.y;
                }

                let line_layout = LineLayout {
                    text: line.text,
                    y_position: current_y,
                    height: self.line_height,
                    runs: line.runs,
                };

                current_page.lines.push(line_layout);
                current_y += self.line_height;
            }

            current_y += self.line_height * 0.5;
        }

        if !current_page.lines.is_empty() || self.pages.is_empty() {
            self.pages.push(current_page);
        }
    }

    pub fn get_page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn get_page_layout(&self, page_index: usize) -> JsValue {
        if let Some(page) = self.pages.get(page_index) {
            serde_wasm_bindgen::to_value(page).unwrap_or(JsValue::NULL)
        } else {
            JsValue::NULL
        }
    }

    pub fn find_position_at_point(&self, page: usize, x: f64, y: f64) -> Option<usize> {
        if let Some(page_layout) = self.pages.get(page) {
            for line in &page_layout.lines {
                if y >= line.y_position && y <= line.y_position + line.height {
                    return Some(self.find_position_in_line(line, x));
                }
            }
        }
        None
    }

    pub fn get_cursor_position(&self, offset: usize) -> Option<Position> {
        let mut current_offset = 0;
        
        for page in &self.pages {
            for line in &page.lines {
                let line_length = line.text.chars().count();
                
                if offset <= current_offset + line_length {
                    let line_offset = offset - current_offset;
                    let x = self.calculate_x_position_in_line(line, line_offset);
                    return Some(Position::new(x, line.y_position));
                }
                
                current_offset += line_length;
            }
        }
        
        None
    }

    pub fn reflow_from_position(&mut self, _position: usize, document: &DocumentModel) -> Vec<usize> {
        self.compute_layout(document);
        (0..self.pages.len()).collect()
    }
}

impl LayoutEngine {
    fn invalidate_layout(&mut self) {
        self.pages.clear();
    }

    fn get_paragraphs_data(&self, document: &DocumentModel) -> Vec<ParagraphData> {
        if let Ok(paragraphs_js) = serde_wasm_bindgen::from_value::<Vec<Paragraph>>(document.paragraphs_js()) {
            paragraphs_js.into_iter().map(|p| ParagraphData {
                text: p.get_text(),
                runs: p.runs,
            }).collect()
        } else {
            Vec::new()
        }
    }

    fn break_paragraph_into_lines(&self, paragraph: &ParagraphData, max_width: f64) -> Vec<LineData> {
        if paragraph.text.is_empty() {
            return vec![LineData {
                text: String::new(),
                runs: Vec::new(),
            }];
        }

        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_runs = Vec::new();
        let mut current_width = 0.0;

        let words: Vec<&str> = paragraph.text.split_whitespace().collect();
        
        for word in words {
            let word_width = self.estimate_text_width(word, 12.0);
            let space_width = self.estimate_text_width(" ", 12.0);
            
            if current_width + word_width + space_width > max_width && !current_line.is_empty() {
                lines.push(LineData {
                    text: current_line.trim().to_string(),
                    runs: current_runs.clone(),
                });
                
                current_line.clear();
                current_runs.clear();
                current_width = 0.0;
            }
            
            if !current_line.is_empty() {
                current_line.push(' ');
                current_width += space_width;
            }
            
            current_line.push_str(word);
            current_width += word_width;
        }
        
        if !current_line.is_empty() {
            lines.push(LineData {
                text: current_line,
                runs: current_runs,
            });
        }

        if lines.is_empty() {
            lines.push(LineData {
                text: String::new(),
                runs: Vec::new(),
            });
        }
        
        lines
    }

    fn estimate_text_width(&self, text: &str, font_size: f64) -> f64 {
        // Estimativa mais precisa baseada em caracteres
        let mut width = 0.0;
        for ch in text.chars() {
            width += match ch {
                'i' | 'l' | '1' | '!' | '|' => font_size * 0.3,
                'I' | 'j' | 't' => font_size * 0.4,
                'm' | 'w' | 'M' | 'W' => font_size * 0.8,
                ' ' => font_size * 0.3,
                _ => font_size * 0.55, // Largura média
            };
        }
        width
    }

    fn find_position_in_line(&self, line: &LineLayout, x: f64) -> usize {
        let mut current_x = 0.0;
        let chars: Vec<char> = line.text.chars().collect();
        
        for (i, ch) in chars.iter().enumerate() {
            let char_width = self.estimate_text_width(&ch.to_string(), 12.0);
            
            if x <= current_x + char_width / 2.0 {
                return i;
            }
            
            current_x += char_width;
        }
        
        chars.len()
    }

    fn calculate_x_position_in_line(&self, line: &LineLayout, offset: usize) -> f64 {
        let chars: Vec<char> = line.text.chars().collect();
        let mut x = 0.0;
        
        for i in 0..offset.min(chars.len()) {
            x += self.estimate_text_width(&chars[i].to_string(), 12.0);
        }
        
        x
    }
}

#[derive(Debug, Clone)]
struct ParagraphData {
    text: String,
    runs: Vec<TextRun>,
}

#[derive(Debug, Clone)]
struct LineData {
    text: String,
    runs: Vec<TextRun>,
}
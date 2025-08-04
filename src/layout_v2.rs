use crate::document_v2::DocumentV2;
use crate::types::*;
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub text: String,
    pub start_position: usize,
    pub end_position: usize,
    pub y: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub number: usize,
    pub lines: Vec<Line>,
    pub bounds: Rect,
    pub content_bounds: Rect,
}

#[wasm_bindgen]
pub struct LayoutEngineV2 {
    page_width: f64,
    page_height: f64,
    margins: Margins,
    line_height: f64,
    char_width: f64, // Largura média de caractere
    chars_per_line: usize,
    lines_per_page: usize,
    pages: Vec<Page>,
}

#[derive(Debug, Clone, Copy)]
struct Margins {
    top: f64,
    right: f64,
    bottom: f64,
    left: f64,
}

#[wasm_bindgen]
impl LayoutEngineV2 {
    #[wasm_bindgen(constructor)]
    pub fn new(page_width: f64, page_height: f64) -> Self {
        let margins = Margins {
            top: 60.0,
            right: 60.0,
            bottom: 60.0,
            left: 60.0,
        };
        
        let line_height = 24.0; // Altura de linha confortável
        let char_width = 8.5; // Largura média para fonte de 14px
        
        let content_width = page_width - margins.left - margins.right;
        let content_height = page_height - margins.top - margins.bottom;
        
        let chars_per_line = (content_width / char_width) as usize;
        let lines_per_page = (content_height / line_height) as usize;
        
        Self {
            page_width,
            page_height,
            margins,
            line_height,
            char_width,
            chars_per_line,
            lines_per_page,
            pages: Vec::new(),
        }
    }

    pub fn set_margins(&mut self, top: f64, right: f64, bottom: f64, left: f64) {
        self.margins = Margins { top, right, bottom, left };
        self.update_metrics();
    }

    pub fn set_line_height(&mut self, height: f64) {
        self.line_height = height;
        self.update_metrics();
    }

    pub fn set_char_width(&mut self, width: f64) {
        self.char_width = width;
        self.update_metrics();
    }

    fn update_metrics(&mut self) {
        let content_width = self.page_width - self.margins.left - self.margins.right;
        let content_height = self.page_height - self.margins.top - self.margins.bottom;
        
        self.chars_per_line = (content_width / self.char_width) as usize;
        self.lines_per_page = (content_height / self.line_height) as usize;
    }

    pub fn layout(&mut self, document: &DocumentV2) {
        self.pages.clear();
        
        let text = document.get_text();
        if text.is_empty() {
            // Criar página vazia
            self.create_empty_page(0);
            return;
        }

        let mut current_page = 0;
        let mut current_line_in_page = 0;
        let mut position = 0;
        let mut lines_for_current_page = Vec::new();

        // Processar linha por linha
        for (line_num, line_text) in text.lines().enumerate() {
            // Quebrar linha longa em múltiplas linhas visuais
            let visual_lines = self.wrap_line(line_text);
            
            for visual_line in visual_lines {
                // Verificar se precisa de nova página
                if current_line_in_page >= self.lines_per_page {
                    self.create_page(current_page, lines_for_current_page);
                    lines_for_current_page = Vec::new();
                    current_page += 1;
                    current_line_in_page = 0;
                }

                let line = Line {
                    text: visual_line.clone(),
                    start_position: position,
                    end_position: position + visual_line.len(),
                    y: self.margins.top + (current_line_in_page as f64 * self.line_height),
                    height: self.line_height,
                };

                position += visual_line.len();
                lines_for_current_page.push(line);
                current_line_in_page += 1;
            }

            // Adicionar caractere de nova linha
            if line_num < text.lines().count() - 1 {
                position += 1; // \n
            }
        }

        // Criar última página se houver linhas pendentes
        if !lines_for_current_page.is_empty() {
            self.create_page(current_page, lines_for_current_page);
        }

        // Garantir pelo menos uma página
        if self.pages.is_empty() {
            self.create_empty_page(0);
        }
    }

    fn wrap_line(&self, line: &str) -> Vec<String> {
        if line.is_empty() {
            return vec![String::new()];
        }

        let mut result = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0.0;

        for word in line.split_whitespace() {
            let word_width = word.len() as f64 * self.char_width;
            let space_width = if current_line.is_empty() { 0.0 } else { self.char_width };

            if current_width + space_width + word_width > (self.page_width - self.margins.left - self.margins.right) {
                if !current_line.is_empty() {
                    result.push(current_line);
                    current_line = String::new();
                    current_width = 0.0;
                }
            }

            if !current_line.is_empty() {
                current_line.push(' ');
                current_width += space_width;
            }

            current_line.push_str(word);
            current_width += word_width;
        }

        if !current_line.is_empty() || result.is_empty() {
            result.push(current_line);
        }

        result
    }

    fn create_page(&mut self, page_number: usize, lines: Vec<Line>) {
        let bounds = Rect::new(0.0, 0.0, self.page_width, self.page_height);
        let content_bounds = Rect::new(
            self.margins.left,
            self.margins.top,
            self.page_width - self.margins.left - self.margins.right,
            self.page_height - self.margins.top - self.margins.bottom,
        );

        self.pages.push(Page {
            number: page_number,
            lines,
            bounds,
            content_bounds,
        });
    }

    fn create_empty_page(&mut self, page_number: usize) {
        self.create_page(page_number, vec![
            Line {
                text: String::new(),
                start_position: 0,
                end_position: 0,
                y: self.margins.top,
                height: self.line_height,
            }
        ]);
    }

    pub fn get_page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn get_page(&self, index: usize) -> JsValue {
        if let Some(page) = self.pages.get(index) {
            serde_wasm_bindgen::to_value(page).unwrap_or(JsValue::NULL)
        } else {
            JsValue::NULL
        }
    }

    pub fn get_position_from_point(&self, page_index: usize, x: f64, y: f64) -> Option<usize> {
        let page = self.pages.get(page_index)?;

        // Encontrar linha mais próxima
        let mut closest_line = None;
        let mut min_distance = f64::MAX;

        for line in &page.lines {
            let line_center_y = line.y + line.height / 2.0;
            let distance = (y - line_center_y).abs();

            if distance < min_distance {
                min_distance = distance;
                closest_line = Some(line);
            }
        }

        let line = closest_line?;

        // Calcular posição dentro da linha
        let relative_x = x - self.margins.left;
        let char_index = ((relative_x / self.char_width).round() as usize).min(line.text.len());

        Some(line.start_position + char_index)
    }

    pub fn get_position_coords(&self, position: usize) -> Option<Vec<f64>> {
        for (page_index, page) in self.pages.iter().enumerate() {
            for line in &page.lines {
                if position >= line.start_position && position <= line.end_position {
                    let char_offset = position - line.start_position;
                    let x = self.margins.left + (char_offset as f64 * self.char_width);
                    let y = line.y;
                    return Some(vec![page_index as f64, x, y]);
                }
            }
        }

        // Se posição está no final do documento
        if let Some(last_page) = self.pages.last() {
            if let Some(last_line) = last_page.lines.last() {
                if position == last_line.end_position {
                    let x = self.margins.left + (last_line.text.len() as f64 * self.char_width);
                    let y = last_line.y;
                    return Some(vec![(self.pages.len() - 1) as f64, x, y]);
                }
            }
        }

        None
    }

    pub fn get_line_at_position(&self, position: usize) -> Option<usize> {
        let mut line_number = 0;

        for page in &self.pages {
            for line in &page.lines {
                if position >= line.start_position && position <= line.end_position {
                    return Some(line_number);
                }
                line_number += 1;
            }
        }

        None
    }

    #[wasm_bindgen(getter)]
    pub fn debug_info(&self) -> JsValue {
        let info = serde_json::json!({
            "page_count": self.pages.len(),
            "chars_per_line": self.chars_per_line,
            "lines_per_page": self.lines_per_page,
            "page_dimensions": {
                "width": self.page_width,
                "height": self.page_height
            },
            "margins": {
                "top": self.margins.top,
                "right": self.margins.right,
                "bottom": self.margins.bottom,
                "left": self.margins.left
            }
        });

        serde_wasm_bindgen::to_value(&info).unwrap_or(JsValue::NULL)
    }
}
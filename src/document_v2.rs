use crate::piece_table::PieceTable;
use crate::types::*;
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    pub start: usize,
    pub end: usize,
}

impl Selection {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn is_collapsed(&self) -> bool {
        self.start == self.end
    }

    pub fn normalize(&self) -> (usize, usize) {
        if self.start <= self.end {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }
}

#[wasm_bindgen]
pub struct DocumentV2 {
    piece_table: PieceTable,
    cursor_position: usize,
    selection: Option<Selection>,
}

#[wasm_bindgen]
impl DocumentV2 {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            piece_table: PieceTable::new(),
            cursor_position: 0,
            selection: None,
        }
    }

    pub fn from_text(text: String) -> Self {
        let piece_table = PieceTable::from_text(text);
        Self {
            piece_table,
            cursor_position: 0,
            selection: None,
        }
    }

    pub fn insert_text(&mut self, text: &str) {
        // Se há seleção, deletar primeiro
        if let Some(sel) = &self.selection {
            let (start, end) = sel.normalize();
            self.piece_table.delete(start, end - start);
            self.cursor_position = start;
            self.selection = None;
        }

        self.piece_table.insert(self.cursor_position, text);
        self.cursor_position += text.len();
    }

    pub fn insert_text_at(&mut self, position: usize, text: &str) {
        let pos = position.min(self.piece_table.length());
        self.piece_table.insert(pos, text);
        
        // Ajustar cursor se necessário
        if self.cursor_position >= pos {
            self.cursor_position += text.len();
        }
    }

    pub fn delete_backward(&mut self, count: usize) {
        if let Some(sel) = &self.selection {
            // Deletar seleção
            let (start, end) = sel.normalize();
            self.piece_table.delete(start, end - start);
            self.cursor_position = start;
            self.selection = None;
        } else if self.cursor_position > 0 {
            // Deletar caracteres antes do cursor
            let delete_count = count.min(self.cursor_position);
            let new_position = self.cursor_position - delete_count;
            self.piece_table.delete(new_position, delete_count);
            self.cursor_position = new_position;
        }
    }

    pub fn delete_forward(&mut self, count: usize) {
        if let Some(sel) = &self.selection {
            // Deletar seleção
            let (start, end) = sel.normalize();
            self.piece_table.delete(start, end - start);
            self.cursor_position = start;
            self.selection = None;
        } else {
            // Deletar caracteres depois do cursor
            let remaining = self.piece_table.length() - self.cursor_position;
            let delete_count = count.min(remaining);
            if delete_count > 0 {
                self.piece_table.delete(self.cursor_position, delete_count);
            }
        }
    }

    pub fn delete_range(&mut self, start: usize, end: usize) {
        let (start, end) = if start <= end { (start, end) } else { (end, start) };
        let length = end - start;
        
        if length > 0 {
            self.piece_table.delete(start, length);
            
            // Ajustar cursor
            if self.cursor_position > end {
                self.cursor_position -= length;
            } else if self.cursor_position > start {
                self.cursor_position = start;
            }
            
            self.selection = None;
        }
    }

    pub fn get_text(&self) -> String {
        self.piece_table.get_text()
    }

    pub fn get_text_range(&self, start: usize, end: usize) -> String {
        let text = self.piece_table.get_text();
        let start = start.min(text.len());
        let end = end.min(text.len());
        
        if start <= end {
            text[start..end].to_string()
        } else {
            String::new()
        }
    }

    pub fn get_line(&self, line_number: usize) -> Option<String> {
        self.piece_table.get_line(line_number)
    }

    pub fn get_line_at_position(&self, position: usize) -> usize {
        let text = self.piece_table.get_text();
        let position = position.min(text.len());
        
        text[..position].chars().filter(|&c| c == '\n').count() + 1
    }

    pub fn get_column_at_position(&self, position: usize) -> usize {
        let text = self.piece_table.get_text();
        let position = position.min(text.len());
        
        // Encontrar o início da linha
        let line_start = text[..position]
            .rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);
        
        position - line_start
    }

    pub fn length(&self) -> usize {
        self.piece_table.length()
    }

    pub fn line_count(&self) -> usize {
        self.piece_table.line_count()
    }

    pub fn set_cursor_position(&mut self, position: usize) {
        self.cursor_position = position.min(self.piece_table.length());
        self.clear_selection();
    }

    pub fn get_cursor_position(&self) -> usize {
        self.cursor_position
    }

    pub fn move_cursor(&mut self, delta: i32) {
        let new_position = if delta < 0 {
            self.cursor_position.saturating_sub((-delta) as usize)
        } else {
            (self.cursor_position + delta as usize).min(self.piece_table.length())
        };
        
        self.set_cursor_position(new_position);
    }

    pub fn move_cursor_to_line_start(&mut self) {
        let line = self.get_line_at_position(self.cursor_position);
        if let Some(line_start) = self.piece_table.get_line_start_position(line) {
            self.set_cursor_position(line_start);
        }
    }

    pub fn move_cursor_to_line_end(&mut self) {
        let text = self.piece_table.get_text();
        let next_newline = text[self.cursor_position..]
            .find('\n')
            .map(|pos| self.cursor_position + pos)
            .unwrap_or(text.len());
        
        self.set_cursor_position(next_newline);
    }

    pub fn set_selection(&mut self, start: usize, end: usize) {
        let length = self.piece_table.length();
        let start = start.min(length);
        let end = end.min(length);
        
        if start == end {
            self.selection = None;
            self.cursor_position = start;
        } else {
            self.selection = Some(Selection::new(start, end));
            self.cursor_position = end;
        }
    }

    pub fn select_all(&mut self) {
        self.set_selection(0, self.piece_table.length());
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    pub fn has_selection(&self) -> bool {
        self.selection.is_some()
    }

    pub fn get_selection(&self) -> Option<Vec<usize>> {
        self.selection.as_ref().map(|sel| {
            let (start, end) = sel.normalize();
            vec![start, end]
        })
    }

    pub fn get_selected_text(&self) -> String {
        if let Some(sel) = self.get_selection() {
            if sel.len() >= 2 {
                self.get_text_range(sel[0], sel[1])
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    }

    #[wasm_bindgen(getter)]
    pub fn debug_info(&self) -> JsValue {
        let info = serde_json::json!({
            "cursor_position": self.cursor_position,
            "length": self.length(),
            "line_count": self.line_count(),
            "has_selection": self.has_selection(),
            "selection": self.get_selection(),
        });
        
        serde_wasm_bindgen::to_value(&info).unwrap_or(JsValue::NULL)
    }
}
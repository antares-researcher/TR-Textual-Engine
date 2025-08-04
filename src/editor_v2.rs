use crate::document_v2::DocumentV2;
use crate::layout_v2::LayoutEngineV2;
use crate::types::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct TreviaEditorV2 {
    document: DocumentV2,
    layout: LayoutEngineV2,
    needs_layout: bool,
}

#[wasm_bindgen]
impl TreviaEditorV2 {
    #[wasm_bindgen(constructor)]
    pub fn new(page_width: f64, page_height: f64) -> Self {
        let document = DocumentV2::new();
        let layout = LayoutEngineV2::new(page_width, page_height);
        
        let mut editor = Self {
            document,
            layout,
            needs_layout: true,
        };
        
        editor.update_layout();
        editor
    }

    pub fn from_text(text: String, page_width: f64, page_height: f64) -> Self {
        let document = DocumentV2::from_text(text);
        let layout = LayoutEngineV2::new(page_width, page_height);
        
        let mut editor = Self {
            document,
            layout,
            needs_layout: true,
        };
        
        editor.update_layout();
        editor
    }

    // === Operações de texto ===
    
    pub fn insert_text(&mut self, text: &str) {
        self.document.insert_text(text);
        self.needs_layout = true;
    }

    pub fn insert_text_at(&mut self, position: usize, text: &str) {
        self.document.insert_text_at(position, text);
        self.needs_layout = true;
    }

    pub fn delete_backward(&mut self, count: usize) {
        self.document.delete_backward(count);
        self.needs_layout = true;
    }

    pub fn delete_forward(&mut self, count: usize) {
        self.document.delete_forward(count);
        self.needs_layout = true;
    }

    pub fn delete_range(&mut self, start: usize, end: usize) {
        self.document.delete_range(start, end);
        self.needs_layout = true;
    }

    pub fn get_text(&self) -> String {
        self.document.get_text()
    }

    pub fn get_selected_text(&self) -> String {
        self.document.get_selected_text()
    }

    pub fn length(&self) -> usize {
        self.document.length()
    }

    // === Cursor e seleção ===

    pub fn set_cursor_position(&mut self, position: usize) {
        self.document.set_cursor_position(position);
    }

    pub fn get_cursor_position(&self) -> usize {
        self.document.get_cursor_position()
    }

    pub fn move_cursor(&mut self, delta: i32) {
        self.document.move_cursor(delta);
    }

    pub fn move_cursor_to_line_start(&mut self) {
        self.document.move_cursor_to_line_start();
    }

    pub fn move_cursor_to_line_end(&mut self) {
        self.document.move_cursor_to_line_end();
    }

    pub fn set_selection(&mut self, start: usize, end: usize) {
        self.document.set_selection(start, end);
    }

    pub fn select_all(&mut self) {
        self.document.select_all();
    }

    pub fn clear_selection(&mut self) {
        self.document.clear_selection();
    }

    pub fn has_selection(&self) -> bool {
        self.document.has_selection()
    }

    pub fn get_selection(&self) -> Option<Vec<usize>> {
        self.document.get_selection()
    }

    // === Layout e renderização ===

    pub fn update_layout(&mut self) {
        if self.needs_layout {
            self.layout.layout(&self.document);
            self.needs_layout = false;
        }
    }

    pub fn get_page_count(&self) -> usize {
        self.layout.get_page_count()
    }

    pub fn get_page(&self, index: usize) -> JsValue {
        self.layout.get_page(index)
    }

    pub fn get_cursor_coords(&self) -> Option<Vec<f64>> {
        let position = self.document.get_cursor_position();
        self.layout.get_position_coords(position)
    }

    pub fn handle_click(&mut self, page_index: usize, x: f64, y: f64) -> bool {
        if let Some(position) = self.layout.get_position_from_point(page_index, x, y) {
            self.document.set_cursor_position(position);
            true
        } else {
            false
        }
    }

    pub fn handle_drag(&mut self, page_index: usize, x: f64, y: f64) {
        if let Some(position) = self.layout.get_position_from_point(page_index, x, y) {
            let cursor_pos = self.document.get_cursor_position();
            
            if let Some(sel) = self.document.get_selection() {
                if sel.len() >= 2 {
                    // Estender seleção existente
                    self.document.set_selection(sel[0], position);
                } else {
                    // Criar nova seleção
                    self.document.set_selection(cursor_pos, position);
                }
            } else {
                // Criar nova seleção
                self.document.set_selection(cursor_pos, position);
            }
        }
    }

    // === Configurações ===

    pub fn set_page_size(&mut self, width: f64, height: f64) {
        self.layout = LayoutEngineV2::new(width, height);
        self.needs_layout = true;
        self.update_layout();
    }

    pub fn set_margins(&mut self, top: f64, right: f64, bottom: f64, left: f64) {
        self.layout.set_margins(top, right, bottom, left);
        self.needs_layout = true;
        self.update_layout();
    }

    pub fn set_line_height(&mut self, height: f64) {
        self.layout.set_line_height(height);
        self.needs_layout = true;
        self.update_layout();
    }

    pub fn set_char_width(&mut self, width: f64) {
        self.layout.set_char_width(width);
        self.needs_layout = true;
        self.update_layout();
    }

    // === Utilitários ===

    pub fn get_line_count(&self) -> usize {
        self.document.line_count()
    }

    pub fn get_line(&self, line_number: usize) -> Option<String> {
        self.document.get_line(line_number)
    }

    pub fn get_cursor_line(&self) -> usize {
        let position = self.document.get_cursor_position();
        self.document.get_line_at_position(position)
    }

    pub fn get_cursor_column(&self) -> usize {
        let position = self.document.get_cursor_position();
        self.document.get_column_at_position(position)
    }

    #[wasm_bindgen(getter)]
    pub fn debug_info(&self) -> JsValue {
        let info = serde_json::json!({
            "needs_layout": self.needs_layout,
            "cursor": self.document.get_cursor_position(),
            "length": self.document.length(),
            "pages": self.layout.get_page_count(),
        });

        serde_wasm_bindgen::to_value(&info).unwrap_or(JsValue::NULL)
    }

    // === Atalhos úteis ===

    pub fn handle_key(&mut self, key: &str, ctrl: bool, shift: bool, _alt: bool) -> bool {
        match key {
            "Backspace" => {
                self.delete_backward(1);
                true
            }
            "Delete" => {
                self.delete_forward(1);
                true
            }
            "ArrowLeft" => {
                if ctrl {
                    // Mover palavra
                } else if shift {
                    // Selecionar
                } else {
                    self.move_cursor(-1);
                }
                true
            }
            "ArrowRight" => {
                if ctrl {
                    // Mover palavra
                } else if shift {
                    // Selecionar
                } else {
                    self.move_cursor(1);
                }
                true
            }
            "Home" => {
                self.move_cursor_to_line_start();
                true
            }
            "End" => {
                self.move_cursor_to_line_end();
                true
            }
            "a" if ctrl => {
                self.select_all();
                true
            }
            _ => false
        }
    }
}
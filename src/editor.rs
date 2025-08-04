use crate::document::DocumentModel;
use crate::layout::LayoutEngine;
use crate::types::*;
use wasm_bindgen::prelude::*;
use std::collections::HashMap;

#[wasm_bindgen]
pub struct TreviaEditor {
    document: DocumentModel,
    layout_engine: LayoutEngine,
    cursor_position: CursorPosition,
    selection_start: Option<CursorPosition>,
    selection_end: Option<CursorPosition>,
    page_cache: HashMap<usize, String>,
    is_dirty: bool,
}

#[wasm_bindgen]
impl TreviaEditor {
    #[wasm_bindgen(constructor)]
    pub fn new(page_width: f64, page_height: f64) -> TreviaEditor {
        let mut editor = TreviaEditor {
            document: DocumentModel::new(),
            layout_engine: LayoutEngine::new(page_width, page_height),
            cursor_position: CursorPosition::new(0, 0),
            selection_start: None,
            selection_end: None,
            page_cache: HashMap::new(),
            is_dirty: true,
        };
        
        editor.recompute_layout();
        editor
    }

    pub fn insert_text(&mut self, text: &str) {
        let cursor_offset = self.cursor_position.offset;
        self.document.insert_text(cursor_offset, text);
        self.move_cursor_by(text.len() as i32);
        self.mark_dirty();
    }

    pub fn delete_text(&mut self, length: usize) {
        let cursor_offset = self.cursor_position.offset;
        if cursor_offset >= length {
            self.document.delete_text(cursor_offset - length, length);
            self.move_cursor_by(-(length as i32));
            self.mark_dirty();
        }
    }

    pub fn delete_selection(&mut self) -> bool {
        if let (Some(start), Some(end)) = (&self.selection_start, &self.selection_end) {
            let start_offset = start.offset.min(end.offset);
            let end_offset = start.offset.max(end.offset);
            let length = end_offset - start_offset;
            
            self.document.delete_text(start_offset, length);
            self.cursor_position = CursorPosition::new(start_offset, start.page);
            self.clear_selection();
            self.mark_dirty();
            true
        } else {
            false
        }
    }

    pub fn set_cursor_position(&mut self, offset: usize) {
        if offset <= self.document.get_text_length() {
            let page = self.find_page_for_offset(offset);
            self.cursor_position = CursorPosition::new(offset, page);
            self.clear_selection();
        }
    }

    pub fn move_cursor_by(&mut self, delta: i32) {
        let current_offset = self.cursor_position.offset as i32;
        let new_offset = (current_offset + delta).max(0) as usize;
        let max_offset = self.document.get_text_length();
        
        self.set_cursor_position(new_offset.min(max_offset));
    }

    pub fn set_selection(&mut self, start_offset: usize, end_offset: usize) {
        let max_offset = self.document.get_text_length();
        let start = start_offset.min(max_offset);
        let end = end_offset.min(max_offset);
        
        let start_page = self.find_page_for_offset(start);
        let end_page = self.find_page_for_offset(end);
        
        self.selection_start = Some(CursorPosition::new(start, start_page));
        self.selection_end = Some(CursorPosition::new(end, end_page));
        self.cursor_position = CursorPosition::new(end, end_page);
    }

    pub fn clear_selection(&mut self) {
        self.selection_start = None;
        self.selection_end = None;
    }

    pub fn has_selection(&self) -> bool {
        self.selection_start.is_some() && self.selection_end.is_some()
    }

    pub fn get_selected_text(&self) -> String {
        if let (Some(start), Some(end)) = (&self.selection_start, &self.selection_end) {
            let start_offset = start.offset.min(end.offset);
            let end_offset = start.offset.max(end.offset);
            let full_text = self.document.get_text();
            
            if let Some(text_slice) = full_text.get(start_offset..end_offset) {
                text_slice.to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    }

    pub fn get_document_text(&self) -> String {
        self.document.get_text()
    }

    pub fn get_document_length(&self) -> usize {
        self.document.get_text_length()
    }

    pub fn get_page_count(&self) -> usize {
        self.layout_engine.get_page_count()
    }

    pub fn get_page_layout(&self, page_index: usize) -> JsValue {
        if self.is_dirty {
            return JsValue::NULL;
        }
        
        self.layout_engine.get_page_layout(page_index)
    }

    pub fn get_cursor_position(&self) -> CursorPosition {
        self.cursor_position
    }

    pub fn get_cursor_screen_position(&self) -> Option<Position> {
        if self.is_dirty {
            return None;
        }
        
        self.layout_engine.get_cursor_position(self.cursor_position.offset)
    }

    pub fn handle_click(&mut self, page: usize, x: f64, y: f64) -> bool {
        if let Some(offset) = self.layout_engine.find_position_at_point(page, x, y) {
            self.set_cursor_position(offset);
            true
        } else {
            false
        }
    }

    pub fn set_page_size(&mut self, width: f64, height: f64) {
        self.layout_engine = LayoutEngine::new(width, height);
        self.mark_dirty();
    }

    pub fn set_margins(&mut self, top: f64, right: f64, bottom: f64, left: f64) {
        self.layout_engine.set_margins(top, right, bottom, left);
        self.mark_dirty();
    }

    pub fn recompute_layout(&mut self) {
        if self.is_dirty {
            self.layout_engine.compute_layout(&self.document);
            self.page_cache.clear();
            self.is_dirty = false;
            
            let page = self.find_page_for_offset(self.cursor_position.offset);
            self.cursor_position.page = page;
        }
    }

    pub fn get_layout_info(&self) -> JsValue {
        let info = LayoutInfo {
            page_count: self.get_page_count(),
            document_length: self.get_document_length(),
            cursor_offset: self.cursor_position.offset,
            cursor_page: self.cursor_position.page,
            has_selection: self.has_selection(),
            is_dirty: self.is_dirty,
        };
        
        serde_wasm_bindgen::to_value(&info).unwrap_or(JsValue::NULL)
    }
}

impl TreviaEditor {
    fn mark_dirty(&mut self) {
        self.is_dirty = true;
        self.page_cache.clear();
    }

    fn find_page_for_offset(&self, offset: usize) -> usize {
        if self.is_dirty {
            return 0;
        }
        
        let mut current_offset = 0;
        
        for page_index in 0..self.layout_engine.get_page_count() {
            let page_layout_js = self.layout_engine.get_page_layout(page_index);
            if !page_layout_js.is_null() {
                if let Ok(page_layout) = serde_wasm_bindgen::from_value::<PageLayoutData>(page_layout_js) {
                    let page_text_length: usize = page_layout.lines.iter()
                        .map(|line| line.text.chars().count())
                        .sum();
                    
                    if offset <= current_offset + page_text_length {
                        return page_index;
                    }
                    
                    current_offset += page_text_length;
                }
            }
        }
        
        self.layout_engine.get_page_count().saturating_sub(1)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct LayoutInfo {
    page_count: usize,
    document_length: usize,
    cursor_offset: usize,
    cursor_page: usize,
    has_selection: bool,
    is_dirty: bool,
}

#[derive(serde::Deserialize)]
struct PageLayoutData {
    lines: Vec<LineLayoutData>,
}

#[derive(serde::Deserialize)]
struct LineLayoutData {
    text: String,
}
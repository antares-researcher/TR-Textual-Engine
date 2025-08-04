use serde::{Deserialize, Serialize};
use std::cmp::min;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BufferType {
    Original,
    Add,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Piece {
    pub buffer_type: BufferType,
    pub start: usize,
    pub length: usize,
    pub line_breaks: Vec<usize>, // Posições relativas das quebras de linha
}

impl Piece {
    pub fn new(buffer_type: BufferType, start: usize, length: usize) -> Self {
        Self {
            buffer_type,
            start,
            length,
            line_breaks: Vec::new(),
        }
    }

    pub fn split_at(&self, offset: usize) -> (Piece, Piece) {
        let mut first = self.clone();
        first.length = offset;
        
        let mut second = self.clone();
        second.start += offset;
        second.length -= offset;
        
        // Ajustar line_breaks
        let mut first_breaks = Vec::new();
        let mut second_breaks = Vec::new();
        
        for &br in &self.line_breaks {
            if br < offset {
                first_breaks.push(br);
            } else {
                second_breaks.push(br - offset);
            }
        }
        
        first.line_breaks = first_breaks;
        second.line_breaks = second_breaks;
        
        (first, second)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PieceTable {
    original_buffer: String,
    add_buffer: String,
    pieces: Vec<Piece>,
    total_length: usize,
    line_count: usize,
}

impl PieceTable {
    pub fn new() -> Self {
        Self {
            original_buffer: String::new(),
            add_buffer: String::new(),
            pieces: Vec::new(),
            total_length: 0,
            line_count: 1,
        }
    }

    pub fn from_text(text: String) -> Self {
        let length = text.len();
        let line_breaks = Self::find_line_breaks(&text);
        let line_count = line_breaks.len() + 1;
        
        let mut piece = Piece::new(BufferType::Original, 0, length);
        piece.line_breaks = line_breaks;
        
        Self {
            original_buffer: text,
            add_buffer: String::new(),
            pieces: vec![piece],
            total_length: length,
            line_count,
        }
    }

    fn find_line_breaks(text: &str) -> Vec<usize> {
        text.char_indices()
            .filter(|(_, ch)| *ch == '\n')
            .map(|(idx, _)| idx)
            .collect()
    }

    pub fn insert(&mut self, position: usize, text: &str) {
        if text.is_empty() {
            return;
        }

        // Adicionar texto ao add_buffer
        let add_start = self.add_buffer.len();
        self.add_buffer.push_str(text);
        
        // Criar nova piece
        let mut new_piece = Piece::new(BufferType::Add, add_start, text.len());
        new_piece.line_breaks = Self::find_line_breaks(text);
        let line_breaks_count = new_piece.line_breaks.len();
        
        // Encontrar onde inserir
        let (piece_index, offset) = self.find_piece_and_offset(position);
        
        if piece_index >= self.pieces.len() {
            // Inserir no final
            self.pieces.push(new_piece);
        } else if offset == 0 {
            // Inserir antes da piece atual
            self.pieces.insert(piece_index, new_piece);
        } else if offset == self.pieces[piece_index].length {
            // Inserir depois da piece atual
            self.pieces.insert(piece_index + 1, new_piece);
        } else {
            // Dividir a piece atual
            let current = self.pieces[piece_index].clone();
            let (first, second) = current.split_at(offset);
            
            self.pieces[piece_index] = first;
            self.pieces.insert(piece_index + 1, new_piece);
            self.pieces.insert(piece_index + 2, second);
        }
        
        self.total_length += text.len();
        self.line_count += line_breaks_count;
    }

    pub fn delete(&mut self, position: usize, length: usize) {
        if length == 0 || position >= self.total_length {
            return;
        }

        let end_position = min(position + length, self.total_length);
        let actual_length = end_position - position;
        
        let (start_piece, start_offset) = self.find_piece_and_offset(position);
        let (end_piece, end_offset) = self.find_piece_and_offset(end_position);
        
        if start_piece == end_piece {
            // Deletar dentro de uma única piece
            let piece = &mut self.pieces[start_piece];
            
            if start_offset == 0 && end_offset == piece.length {
                // Deletar piece inteira
                self.pieces.remove(start_piece);
            } else if start_offset == 0 {
                // Deletar do início
                piece.start += end_offset;
                piece.length -= end_offset;
                // Ajustar line_breaks
                let removed_breaks = piece.line_breaks.iter()
                    .filter(|&&br| br < end_offset)
                    .count();
                piece.line_breaks.retain(|&br| br >= end_offset);
                piece.line_breaks.iter_mut().for_each(|br| *br -= end_offset);
                self.line_count -= removed_breaks;
            } else if end_offset == piece.length {
                // Deletar até o final
                piece.length = start_offset;
                let removed_breaks = piece.line_breaks.iter()
                    .filter(|&&br| br >= start_offset)
                    .count();
                piece.line_breaks.retain(|&br| br < start_offset);
                self.line_count -= removed_breaks;
            } else {
                // Deletar do meio - criar duas pieces
                let original = piece.clone();
                let (first, _) = original.split_at(start_offset);
                let (_, mut second) = original.split_at(end_offset);
                
                // Ajustar a segunda parte
                second.start = original.start + end_offset;
                second.length = original.length - end_offset;
                
                self.pieces[start_piece] = first;
                self.pieces.insert(start_piece + 1, second);
            }
        } else {
            // Deletar múltiplas pieces
            let mut pieces_to_remove = Vec::new();
            
            // Ajustar primeira piece
            if start_offset > 0 {
                self.pieces[start_piece].length = start_offset;
                let removed_breaks = self.pieces[start_piece].line_breaks.iter()
                    .filter(|&&br| br >= start_offset)
                    .count();
                self.pieces[start_piece].line_breaks.retain(|&br| br < start_offset);
                self.line_count -= removed_breaks;
            } else {
                pieces_to_remove.push(start_piece);
            }
            
            // Marcar pieces intermediárias para remoção
            for i in (start_piece + 1)..end_piece {
                pieces_to_remove.push(i);
                self.line_count -= self.pieces[i].line_breaks.len();
            }
            
            // Ajustar última piece
            if end_piece < self.pieces.len() {
                if end_offset < self.pieces[end_piece].length {
                    let removed_breaks = self.pieces[end_piece].line_breaks.iter()
                        .filter(|&&br| br < end_offset)
                        .count();
                    self.pieces[end_piece].start += end_offset;
                    self.pieces[end_piece].length -= end_offset;
                    self.pieces[end_piece].line_breaks.retain(|&br| br >= end_offset);
                    self.pieces[end_piece].line_breaks.iter_mut().for_each(|br| *br -= end_offset);
                    self.line_count -= removed_breaks;
                } else {
                    pieces_to_remove.push(end_piece);
                }
            }
            
            // Remover pieces marcadas (em ordem reversa)
            pieces_to_remove.sort_by(|a, b| b.cmp(a));
            for idx in pieces_to_remove {
                self.pieces.remove(idx);
            }
        }
        
        self.total_length -= actual_length;
    }

    pub fn get_text(&self) -> String {
        let mut result = String::with_capacity(self.total_length);
        
        for piece in &self.pieces {
            let buffer = match piece.buffer_type {
                BufferType::Original => &self.original_buffer,
                BufferType::Add => &self.add_buffer,
            };
            
            result.push_str(&buffer[piece.start..piece.start + piece.length]);
        }
        
        result
    }

    pub fn get_line(&self, line_number: usize) -> Option<String> {
        if line_number == 0 || line_number > self.line_count {
            return None;
        }

        let mut current_line = 1;
        let mut result = String::new();
        let mut in_target_line = false;

        for piece in &self.pieces {
            let buffer = match piece.buffer_type {
                BufferType::Original => &self.original_buffer,
                BufferType::Add => &self.add_buffer,
            };
            
            let piece_text = &buffer[piece.start..piece.start + piece.length];
            
            for ch in piece_text.chars() {
                if current_line == line_number {
                    in_target_line = true;
                    if ch == '\n' {
                        return Some(result);
                    }
                    result.push(ch);
                } else if ch == '\n' {
                    current_line += 1;
                    if current_line == line_number {
                        in_target_line = true;
                    }
                }
            }
        }

        if in_target_line {
            Some(result)
        } else {
            None
        }
    }

    pub fn length(&self) -> usize {
        self.total_length
    }

    pub fn line_count(&self) -> usize {
        self.line_count
    }

    fn find_piece_and_offset(&self, position: usize) -> (usize, usize) {
        let mut current_pos = 0;
        
        for (i, piece) in self.pieces.iter().enumerate() {
            if position <= current_pos + piece.length {
                return (i, position - current_pos);
            }
            current_pos += piece.length;
        }
        
        (self.pieces.len(), 0)
    }

    pub fn get_line_start_position(&self, line_number: usize) -> Option<usize> {
        if line_number == 0 || line_number > self.line_count {
            return None;
        }

        if line_number == 1 {
            return Some(0);
        }

        let mut current_line = 1;
        let mut position = 0;

        for piece in &self.pieces {
            if current_line >= line_number {
                return Some(position);
            }

            for &line_break in &piece.line_breaks {
                current_line += 1;
                if current_line == line_number {
                    return Some(position + line_break + 1);
                }
            }

            position += piece.length;
        }

        None
    }
}
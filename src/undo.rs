use std::collections::VecDeque;
const MAX_UNDO_STACK_SIZE: usize = 100;

use crate::Line;

#[derive(Debug, Clone)]
pub enum UndoAction {
    Erase(Line),
    Draw(Line),
}
pub struct UndoStack {
    stack: VecDeque<UndoAction>,
}
impl Default for UndoStack {
    fn default() -> Self {
        Self {
            stack: VecDeque::new(),
        }
    }
}
impl UndoStack {
    pub fn add_erase(&mut self, line: Line) {
        self.stack.push_back(UndoAction::Erase(line));
        if self.stack.len() > MAX_UNDO_STACK_SIZE {
            self.stack.pop_front();
        }
    }
    pub fn add_draw(&mut self, line: Line) {
        self.stack.push_back(UndoAction::Draw(line));
        if self.stack.len() > MAX_UNDO_STACK_SIZE {
            self.stack.pop_front();
        }
    }
    pub fn extend_erase(&mut self, erased: Vec<Line>) {
        self.stack.extend(erased.into_iter().map(UndoAction::Erase));
        if self.stack.len() > MAX_UNDO_STACK_SIZE {
            self.stack.pop_front();
        }
    }
    pub fn pop(&mut self) -> Option<UndoAction> {
        self.stack.pop_back()
    }
}

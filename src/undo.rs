use std::collections::VecDeque;
const MAX_UNDO_STACK_SIZE: usize = 100;

use crate::Line;

#[derive(Debug, Clone)]
pub enum UndoAction {
    Erase(Line),
    Draw(Line),
}
#[derive(Default)]
pub struct UndoStack {
    stack: VecDeque<UndoAction>,
}
impl UndoStack {
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

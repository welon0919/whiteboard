use std::collections::VecDeque;
const MAX_UNDO_STACK_SIZE: usize = 100;

use crate::Element;

#[derive(Debug, Clone)]
pub enum UndoAction {
    Erase(Vec<(usize, Element)>),
    Draw,
    Modify(Vec<(usize, Element)>),
}
#[derive(Default)]
pub struct UndoStack {
    stack: VecDeque<UndoAction>,
}
impl UndoStack {
    pub fn add_draw(&mut self) {
        self.stack.push_back(UndoAction::Draw);
        if self.stack.len() > MAX_UNDO_STACK_SIZE {
            self.stack.pop_front();
        }
    }
    pub fn add_erase(&mut self, erased: Vec<(usize, Element)>) {
        self.stack.push_back(UndoAction::Erase(erased));
        if self.stack.len() > MAX_UNDO_STACK_SIZE {
            self.stack.pop_front();
        }
    }
    pub fn add_modify(&mut self, modified: Vec<(usize, Element)>) {
        self.stack.push_back(UndoAction::Modify(modified));
        if self.stack.len() > MAX_UNDO_STACK_SIZE {
            self.stack.pop_front();
        }
    }
    pub fn pop(&mut self) -> Option<UndoAction> {
        self.stack.pop_back()
    }
}

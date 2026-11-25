use crate::canvas::Canvas;

pub struct History {
    undo: Vec<Canvas>,
    redo: Vec<Canvas>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    pub fn save(&mut self, canvas: &Canvas) {
        self.undo.push(canvas.clone());

        if self.undo.len() > 100 {
            self.undo.remove(0);
        }

        self.redo.clear();
    }

    pub fn undo(&mut self, current: &Canvas) -> Option<Canvas> {
        let previous = self.undo.pop()?;
        self.redo.push(current.clone());
        Some(previous)
    }

    pub fn redo(&mut self, current: &Canvas) -> Option<Canvas> {
        let next = self.redo.pop()?;
        self.undo.push(current.clone());
        Some(next)
    }
}

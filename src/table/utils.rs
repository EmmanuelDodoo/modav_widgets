use iced::{mouse, Point, Rectangle, Size, Vector};
use std::ops::Range;

#[derive(Debug, Clone, Copy)]
pub enum State {
    Index(usize),
    Selection { start: usize, end: usize },
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor {
            state: State::Index(0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    state: State,
}

impl Cursor {
    pub fn state(&self, value: &str) -> State {
        let len = value.len();
        match self.state {
            State::Index(idx) => State::Index(idx.min(len)),
            State::Selection { start, end } => {
                let start = start.min(len);
                let end = end.min(len);

                if start == end {
                    State::Index(start)
                } else {
                    State::Selection { start, end }
                }
            }
        }
    }

    pub fn selection(&self, value: &str) -> Option<(usize, usize)> {
        match self.state(value) {
            State::Selection { start, end } => Some((start.min(end), start.max(end))),
            State::Index(_) => None,
        }
    }

    pub fn move_to(&mut self, position: usize) {
        self.state = State::Index(position);
    }

    pub fn move_left(&mut self, value: &str) {
        match self.state(value) {
            State::Index(idx) if idx > 0 => self.move_to(idx - 1),
            State::Selection { start, end } => self.move_to(start.min(end)),
            State::Index(_) => self.move_to(0),
        }
    }

    pub fn move_right(&mut self, value: &str) {
        self.move_right_by_amount(value, 1)
    }

    pub fn move_right_by_amount(&mut self, value: &str, amount: usize) {
        match self.state(value) {
            State::Index(idx) => self.move_to(idx.saturating_add(amount).min(value.len())),
            State::Selection { start, end } => self.move_to(end.max(start)),
        }
    }

    pub fn select_range(&mut self, start: usize, end: usize) {
        if start == end {
            self.state = State::Index(start);
        } else {
            self.state = State::Selection {
                start: start.min(end),
                end: end.max(start),
            }
        }
    }

    pub fn select_all(&mut self, value: &str) {
        self.select_range(0, value.len())
    }

    pub fn select_left(&mut self, value: &str) {
        match self.state(value) {
            State::Index(index) if index > 0 => {
                self.select_range(index, index - 1);
            }
            State::Selection { start, end } if end > 0 => {
                self.select_range(start.saturating_sub(1), end);
            }
            _ => {}
        }
    }

    pub fn select_right(&mut self, value: &str) {
        match self.state(value) {
            State::Index(index) if index < value.len() => {
                self.select_range(index, index + 1);
            }
            State::Selection { start, end } if end < value.len() => {
                self.select_range(start, end + 1);
            }
            _ => {}
        }
    }

    pub fn start(&self, value: &str) -> usize {
        let start = match self.state {
            State::Index(idx) => idx,
            State::Selection { start, .. } => start,
        };

        start.min(value.len())
    }

    pub fn end(&self, value: &str) -> usize {
        let end = match self.state {
            State::Index(idx) => idx,
            State::Selection { end, .. } => end,
        };

        end.min(value.len())
    }

    pub fn left(&self, value: &str) -> usize {
        match self.state(value) {
            State::Index(idx) => idx,
            State::Selection { start, end } => start.min(end),
        }
    }

    pub fn right(&self, value: &str) -> usize {
        match self.state(value) {
            State::Index(idx) => idx,
            State::Selection { start, end } => start.max(end),
        }
    }
}

pub struct Editor<'a> {
    value: &'a mut String,
    cursor: &'a mut Cursor,
}

impl<'a> Editor<'a> {
    pub fn new(value: &'a mut String, cursor: &'a mut Cursor) -> Self {
        Self { value, cursor }
    }

    pub fn contents(&self) -> String {
        self.value.to_string()
    }

    pub fn insert(&mut self, character: char) {
        if let Some((left, right)) = self.cursor.selection(self.value) {
            self.cursor.move_left(self.value);
            self.value.replace_range(left..right, "");
        }

        self.value.insert(self.cursor.end(self.value), character);
        self.cursor.move_right(&self.value)
    }

    pub fn backspace(&mut self) {
        match self.cursor.selection(&self.value) {
            Some((start, end)) => {
                self.cursor.move_left(self.value);
                self.value.replace_range(start..end, "");
            }
            None => {
                let start = self.cursor.start(&self.value);

                if start > 0 {
                    self.cursor.move_left(&self.value);
                    self.value.remove(start - 1);
                }
            }
        }
    }

    pub fn delete(&mut self) {
        match self.cursor.selection(&self.value) {
            Some(_) => {
                self.backspace();
            }
            None => {
                let end = self.cursor.end(&self.value);

                if end < self.value.len() {
                    self.value.remove(end);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Selection {
    Block {
        rows: Range<usize>,
        columns: Range<usize>,
    },
    Scattered(Vec<usize>),
}

impl Selection {
    //fn new(row: usize, )
}

#[derive(Debug, Clone, Copy)]
enum Drag {
    Vertical,
    Horizontal,
    Diagonal,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Resizing {
    kind: Drag,
    cursor: Point,
    pub(crate) row: usize,
    pub(crate) column: usize,
}

impl Resizing {
    pub(crate) fn new(
        parent: Rectangle,
        child: Rectangle,
        cursor: mouse::Cursor,
        row: usize,
        column: usize,
    ) -> Option<Self> {
        let horizontal = {
            let height = parent.height;
            let width = parent.width - child.width;
            let position = parent.position() + Vector::new(child.width, 0.0);

            let bounds = Rectangle::new(position, Size::new(width, height));
            cursor.is_over(bounds)
        };

        let vertical = {
            let height = parent.height - child.height;
            let width = parent.width;
            let position = parent.position() + Vector::new(0.0, child.height);

            let bounds = Rectangle::new(position, Size::new(width, height));
            cursor.is_over(bounds)
        };

        let kind = if horizontal && vertical {
            Drag::Diagonal
        } else if horizontal {
            Drag::Horizontal
        } else if vertical {
            Drag::Vertical
        } else {
            return None;
        };

        let cursor = cursor.position_over(parent)?;

        Some(Self {
            kind,
            row,
            column,
            cursor,
        })
    }

    /// Returns the new minimum dimensions after a drag
    pub(crate) fn drag(&mut self, position: Point, width: f32, height: f32) -> (Size, Vector) {
        let diff = position - self.cursor;
        self.cursor = position;

        match self.kind {
            Drag::Vertical => {
                let size = Size::new(width, height + diff.y);
                let diff = Vector::new(0.0, diff.y);

                (size, diff)
            }
            Drag::Horizontal => {
                let size = Size::new(width + diff.x, height);
                let diff = Vector::new(-diff.x, 0.0);

                (size, diff)
            }
            Drag::Diagonal => (
                Size::new(width + diff.x, height + diff.y),
                Vector::new(-diff.x, diff.y),
            ),
        }
    }

    pub(crate) fn interaction(self) -> mouse::Interaction {
        match self.kind {
            Drag::Vertical => mouse::Interaction::ResizingVertically,
            Drag::Horizontal => mouse::Interaction::ResizingHorizontally,
            Drag::Diagonal => mouse::Interaction::ResizingDiagonallyDown,
        }
    }
}

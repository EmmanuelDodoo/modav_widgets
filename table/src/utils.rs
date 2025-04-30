use iced::{alignment::Horizontal, keyboard, mouse, Point, Rectangle, Size, Vector};
use std::collections::HashSet;

#[allow(unused_imports)]
use super::Table;
use std::ops::RangeInclusive;

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

    pub fn move_to_end(&mut self, value: &str) {
        self.state = State::Index(value.len());
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

    pub fn select_to_start(&mut self, value: &str) {
        match self.state(value) {
            State::Index(index) => self.select_range(0, index),
            State::Selection { end, .. } => self.select_range(0, end),
        }
    }

    pub fn select_to_end(&mut self, value: &str) {
        match self.state(value) {
            State::Index(index) => self.select_range(index, value.len()),
            State::Selection { start, .. } => self.select_range(start, value.len()),
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

    pub fn _left(&self, value: &str) -> usize {
        match self.state(value) {
            State::Index(idx) => idx,
            State::Selection { start, end } => start.min(end),
        }
    }

    pub fn _right(&self, value: &str) -> usize {
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

    pub fn _contents(&self) -> String {
        self.value.to_string()
    }

    pub fn insert(&mut self, character: char) {
        if let Some((left, right)) = self.cursor.selection(self.value) {
            self.cursor.move_left(self.value);
            self.value.replace_range(left..right, "");
        }

        self.value.insert(self.cursor.end(self.value), character);
        self.cursor.move_right(self.value)
    }

    pub fn backspace(&mut self) {
        match self.cursor.selection(self.value) {
            Some((start, end)) => {
                self.cursor.move_left(self.value);
                self.value.replace_range(start..end, "");
            }
            None => {
                let start = self.cursor.start(self.value);

                if start > 0 {
                    self.cursor.move_left(self.value);
                    self.value.remove(start - 1);
                }
            }
        }
    }

    pub fn delete(&mut self) {
        match self.cursor.selection(self.value) {
            Some(_) => {
                self.backspace();
            }
            None => {
                let end = self.cursor.end(self.value);

                if end < self.value.len() {
                    self.value.remove(end);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
/// A group of selected cells.
pub enum Selection {
    /// A continuous selection.
    Block {
        rows: RangeInclusive<usize>,
        columns: RangeInclusive<usize>,
    },
    /// A selection which is not necessarily continguous.
    Scattered {
        cells: HashSet<(usize, usize)>,
        last: (usize, usize),
    },
}

impl Selection {
    pub(super) fn new(row: usize, column: usize) -> Self {
        Self::Block {
            rows: row..=row,
            columns: column..=column,
        }
    }

    pub(super) fn row(row: usize, column_len: usize) -> Self {
        Self::Block {
            rows: row..=row,
            columns: 0..=column_len,
        }
    }

    pub(super) fn column(column: usize, limit: usize) -> Self {
        Self::Block {
            rows: 0..=limit,
            columns: column..=column,
        }
    }

    pub(super) fn block(&mut self, row: usize, column: usize) {
        match self {
            Self::Block { rows, columns } => {
                if !rows.contains(&row) {
                    let start = *rows.start().min(&row);
                    let end = *rows.end().max(&row);

                    *rows = start..=end
                }

                if !columns.contains(&column) {
                    let start = *columns.start().min(&column);
                    let end = *columns.end().max(&column);

                    *columns = start..=end
                }
            }
            Self::Scattered { cells, last } => {
                let rows = row.min(last.0)..=row.max(last.0);
                let columns = (column.min(last.1)..=column.max(last.1)).collect::<Vec<usize>>();
                *last = (row, column);

                for row in rows {
                    let set = columns.iter().map(|column| (row, *column));

                    cells.extend(set);
                }
            }
        }
    }

    pub(super) fn scattered(&mut self, row: usize, column: usize) {
        match self {
            Self::Block { rows, columns } => {
                let rows = rows.collect::<Vec<usize>>();
                let columns = columns.collect::<Vec<usize>>();
                let mut cells = HashSet::new();
                cells.insert((row, column));

                for row in rows {
                    let set = columns.iter().map(|column| (row, *column));

                    cells.extend(set)
                }

                *self = Self::Scattered {
                    cells,
                    last: (row, column),
                }
            }
            Self::Scattered { cells, last } => {
                cells.insert((row, column));
                *last = (row, column)
            }
        }
    }

    pub(super) fn contains(&self, row: usize, column: usize) -> bool {
        match self {
            Self::Block { rows, columns } => rows.contains(&row) && columns.contains(&column),
            Self::Scattered { cells, .. } => cells.contains(&(row, column)),
        }
    }

    pub(super) fn border(&self, row: usize, column: usize) -> u8 {
        match self {
            Self::Block { rows, columns } => {
                // bottom, right, top, left
                let mut out = 0;

                if !self.contains(row, column) {
                    return 0;
                }

                if *rows.start() == row {
                    // top
                    out |= 1 << 1;
                }

                if *rows.end() == row {
                    // bottom
                    out |= 1 << 3;
                }

                if *columns.start() == column {
                    // left
                    out |= 1 << 0;
                }

                if *columns.end() == column {
                    // right
                    out |= 1 << 2;
                }

                out
            }
            Self::Scattered { cells, .. } => {
                if cells.contains(&(row, column)) {
                    return 15;
                }

                0
            }
        }
    }

    pub(super) fn header(&self, column: usize) -> bool {
        match self {
            Self::Block { columns, .. } => columns.contains(&column),
            Self::Scattered { cells, .. } => cells.iter().any(|(_, col)| *col == column),
        }
    }

    pub(super) fn move_to(&mut self, row: usize, column: usize) {
        *self = Self::Block {
            rows: row..=row,
            columns: column..=column,
        };
    }

    pub(super) fn move_right(&mut self, column_limit: usize) {
        match self {
            Self::Block { columns, rows } => {
                let row = *rows.start();
                let column = (*columns.start() + 1).min(column_limit);

                self.move_to(row, column);
            }
            Self::Scattered { last, .. } => {
                let row = last.0;
                let column = (last.1 + 1).min(column_limit);
                self.move_to(row, column);
            }
        }
    }

    pub(super) fn move_left(&mut self) {
        match self {
            Self::Block { columns, rows } => {
                let row = *rows.start();
                let column = columns.start().saturating_sub(1);

                self.move_to(row, column);
            }
            Self::Scattered { last, .. } => {
                let row = last.0;
                let column = last.1.saturating_sub(1);
                self.move_to(row, column);
            }
        }
    }

    pub(super) fn move_down(&mut self, row_limit: usize) {
        match self {
            Self::Block { rows, columns } => {
                let column = *columns.start();
                let row = (*rows.start() + 1).min(row_limit);

                self.move_to(row, column);
            }
            Self::Scattered { last, .. } => {
                let column = last.1;
                let row = (last.0 + 1).min(row_limit);
                self.move_to(row, column)
            }
        }
    }

    pub(super) fn move_up(&mut self) {
        match self {
            Self::Block { rows, columns } => {
                let column = *columns.start();
                let row = rows.start().saturating_sub(1);

                self.move_to(row, column);
            }
            Self::Scattered { last, .. } => {
                let column = last.1;
                let row = last.0.saturating_sub(1);
                self.move_to(row, column)
            }
        }
    }

    pub(super) fn grow(
        &mut self,
        row_amt: usize,
        row_limit: usize,
        column_amt: usize,
        column_limit: usize,
    ) {
        if let Self::Block { rows, columns } = self {
            let end = (*rows.end() + row_amt).min(row_limit);
            *rows = *rows.start()..=end;

            let end = (*columns.end() + column_amt).min(column_limit);
            *columns = *columns.start()..=end;
        }
    }

    pub(super) fn shrink(&mut self, row_amt: usize, column_amt: usize) {
        if let Self::Block { rows, columns } = self {
            let end = *rows.end();
            if end == 0 {
                return;
            }
            let end = (end - row_amt).max(*rows.start());

            *rows = *rows.start()..=end;

            let end = *columns.end();
            if end == 0 {
                return;
            }
            let end = (end - column_amt).max(*columns.start());

            *columns = *columns.start()..=end;
        }
    }

    /// Returns the `(row, column)` indices for each unique cell in the [`Selection`].
    pub fn list(&self) -> HashSet<(usize, usize)> {
        match self {
            Self::Block { rows, columns } => {
                let mut cells = HashSet::new();
                let rows = rows.clone().collect::<Vec<usize>>();
                let columns = columns.clone().collect::<Vec<usize>>();

                for row in rows {
                    let set = columns.iter().map(|column| (row, *column));

                    cells.extend(set)
                }

                cells
            }
            Self::Scattered { cells, .. } => cells.clone(),
        }
    }
}

/// The direction in which a resize occurs
#[derive(Debug, Clone, Copy)]
pub enum ResizeDirection {
    /// A row resize affecting only the row height
    Vertical,
    /// A column resize affecting only the column width
    Horizontal,
    /// Both vertical and horizontal resizing
    Diagonal,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Resizing {
    kind: ResizeDirection,
    cursor: Point,
    pub(super) row: usize,
    pub(super) column: usize,
}

impl Resizing {
    pub(super) fn new(
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
            ResizeDirection::Diagonal
        } else if horizontal {
            ResizeDirection::Horizontal
        } else if vertical {
            ResizeDirection::Vertical
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
    pub(super) fn drag(&mut self, position: Point, width: f32, height: f32) -> (Size, Vector) {
        let diff = position - self.cursor;
        self.cursor = position;

        match self.kind {
            ResizeDirection::Vertical => {
                let size = Size::new(width, height + diff.y);
                let diff = Vector::new(0.0, diff.y);

                (size, diff)
            }
            ResizeDirection::Horizontal => {
                let size = Size::new(width + diff.x, height);
                let diff = Vector::new(-diff.x, 0.0);

                (size, diff)
            }
            ResizeDirection::Diagonal => (
                Size::new(width + diff.x, height + diff.y),
                Vector::new(-diff.x, diff.y),
            ),
        }
    }

    pub(super) fn interaction(self) -> mouse::Interaction {
        match self.kind {
            ResizeDirection::Vertical => mouse::Interaction::ResizingVertically,
            ResizeDirection::Horizontal => mouse::Interaction::ResizingHorizontally,
            ResizeDirection::Diagonal => mouse::Interaction::ResizingDiagonallyDown,
        }
    }

    pub(super) fn action(&self, size: Size) -> Action {
        Action::Resize {
            direction: self.kind,
            column: self.column.saturating_sub(1),
            row: self.row.saturating_sub(1),
            size,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// A key press.
pub struct KeyPress {
    /// The key pressed.
    pub key: keyboard::Key,
    /// The state of the keyboard modifiers.
    pub modifiers: keyboard::Modifiers,
    /// The text produced by the key press.
    pub text: Option<String>,
}

/// An interaction with a [`Table`].
#[derive(Debug, Clone)]
pub enum Action {
    /// A character insertion in a header
    HeaderInput { value: String, column: usize },
    /// A character insertion in a cell
    CellInput {
        value: String,
        column: usize,
        row: usize,
    },
    /// A header submission
    HeaderSubmit { value: String, column: usize },
    /// A cell submission
    CellSubmit {
        value: String,
        column: usize,
        row: usize,
    },
    /// A cell selection
    Selection(Selection),
    /// A page change
    PageChange { previous: usize, current: usize },
    /// A column and/or row resizing
    Resize {
        direction: ResizeDirection,
        size: Size,
        column: usize,
        row: usize,
    },
}

impl Action {
    pub(super) fn cell_input(value: String, column: usize, row: usize) -> Self {
        Self::CellInput { value, column, row }
    }

    pub(super) fn cell_submit(value: String, column: usize, row: usize) -> Self {
        Self::CellSubmit { value, column, row }
    }

    pub(super) fn header_input(value: String, column: usize) -> Self {
        Self::HeaderInput { value, column }
    }

    pub(super) fn header_submit(value: String, column: usize) -> Self {
        Self::HeaderSubmit { value, column }
    }

    pub(super) fn page(previous: usize, current: usize) -> Self {
        Self::PageChange {
            previous: previous + 1,
            current: current + 1,
        }
    }
}

/// The underlying data type for a [`Table`] widget.
pub trait RawTable {
    /// The type of values in a column
    type ColumnKind: std::fmt::Display;

    /// Returns the number of data rows (excluding header rows) in the [`RawTable`].
    fn height(&self) -> usize;

    /// Returns the number of data columns in the [`RawTable`].
    fn width(&self) -> usize;

    /// Returns the header for the column at `index` if it exists.
    fn column_header(&self, index: usize) -> Option<String>;

    /// Returns the `ColumnKind` for the column at `index` if it exists.
    fn column_kind(&self, index: usize) -> Option<Self::ColumnKind>;

    /// Returns the value at the specified row and column in the [RawTable],
    /// if it exists.
    fn cell(&self, row: usize, column: usize) -> Option<String>;

    /// Returns true if the [`RawTable`] has no cells.
    fn is_empty(&self) -> bool;

    /// Returns `true` if the `character` is accepted by the specified `ColumnKind`.
    fn column_filter(&self, kind: &Self::ColumnKind, character: char) -> bool;

    /// Returns the [`Horizontal`] column alignment for the specified `ColumnKind`.
    fn kind_alignment(&self, kind: &Self::ColumnKind) -> Horizontal;
}
